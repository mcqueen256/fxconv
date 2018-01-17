use chrono::prelude::*;
use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;
use std::thread;
use std::sync::mpsc::channel;
use time::Duration;
use formatter::InputRow;
use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
#[derive(PartialEq)]
pub struct TickGroup {
    pub datetimes: Vec<DateTime<Utc>>,
    pub asks: Vec<f32>,
    pub bids: Vec<f32>
}

impl TickGroup {
    fn new() -> TickGroup {
        TickGroup {
            datetimes: Vec::new(),
            asks: Vec::new(),
            bids: Vec::new()
        }
    }

    /// Clears the current contents and returns that contents in a new group
    fn dump(&mut self) -> TickGroup {
        let group = TickGroup {
            datetimes: self.datetimes.clone(),
            asks: self.asks.clone(),
            bids: self.bids.clone()
        };
        self.datetimes.clear();
        self.asks.clear();
        self.bids.clear();
        group
    }

    fn push(&mut self, datetime: DateTime<Utc>, ask: f32, bid: f32) {
        self.datetimes.push(datetime);
        self.asks.push(ask);
        self.bids.push(bid);
    }

    fn len(&self) -> usize {
        self.datetimes.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn create(rx_formatter: Receiver<Option<InputRow>>, time_frame: TimeFrame)  -> (thread::JoinHandle<()>, Receiver<Option<TickGroup>>) {
    let (tx_grouper, rx_grouper) = channel(); // TODO:: make buffered channel with configurable limit
    let grouper_thread = thread::spawn(move || {
        grouper(tx_grouper, rx_formatter, time_frame);
    });
    (grouper_thread, rx_grouper)
}

fn grouper(tx_grouper: Sender<Option<TickGroup>>, rx_formatter: Receiver<Option<InputRow>>, time_frame: TimeFrame) {
    // Select unit measurment
    let duration_func = match  *time_frame.unit() {
        TimeUnit::Second => Duration::seconds,
        TimeUnit::Minute => Duration::minutes,
        TimeUnit::Hour => Duration::hours,
        TimeUnit::Day => Duration::days,
        TimeUnit::Week => Duration::weeks
    };

    let over_timeframe = |a: DateTime<Utc>,b: DateTime<Utc>| {
        a.signed_duration_since(b) >= duration_func(time_frame.len() as i64)
    };

    // to store the data in the frame
    let mut group = TickGroup::new();
    let mut first: Option<DateTime<Utc>> = None; // first datetime in timeframe

    while let Some(row) = rx_formatter.recv().expect("Unable to receive from channel") {
        // if not initialized, then init
        if first == None {
            first = Some(row.datetime);
        }

        if over_timeframe(row.datetime, first.unwrap()) {
            while over_timeframe(row.datetime, first.unwrap()) {
                first = Some(first.unwrap() + duration_func(time_frame.len() as i64));
            }
            tx_grouper.send(Some(group.dump())).unwrap();
            // reset the group
            group = TickGroup::new();
        }
        group.push(row.datetime, row.ask, row.bid);
    }
    if ! group.is_empty() {
        tx_grouper.send(Some(group)).unwrap();
    }
    tx_grouper.send(None).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_data() {
        let (txf, rxf) = channel();
        let (txg, rxg) = channel();
        txf.send(None).expect("Could not send None");
        grouper(txg, rxf, TimeFrame::new(1, TimeUnit::Day));
        assert_eq!(rxg.recv().expect("Failed to recieve"), None);
    }

    #[test]
    fn one() {
        let (txf, rxf) = channel();
        let (txg, rxg) = channel();
        txf.send(Some(InputRow {
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613),
            ask: 1.1234,
            bid: 1.1222
        })).expect("Could not send None");
        txf.send(None).expect("Could not send None");
        grouper(txg, rxf, TimeFrame::new(1, TimeUnit::Day));
        assert_eq!(rxg.recv().expect("Failed to recieve"), Some(TickGroup {
            datetimes: vec![Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613)],
            asks: vec![1.1234],
            bids: vec![1.1222]
        }));
        assert_eq!(rxg.recv().expect("Failed to recieve"), None);
    }

    #[test]
    fn two_same_time_frames() {
        let (txf, rxf) = channel();
        let (txg, rxg) = channel();
        txf.send(Some(InputRow {
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613),
            ask: 1.1234,
            bid: 1.1222
        })).expect("Could not send None");
        txf.send(Some(InputRow {
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(23, 25, 36, 923),
            ask: 1.1204,
            bid: 1.1195
        })).expect("Could not send None");
        txf.send(None).expect("Could not send None");
        grouper(txg, rxf, TimeFrame::new(1, TimeUnit::Day));
        assert_eq!(rxg.recv().expect("Failed to recieve"), Some(TickGroup {
            datetimes: vec![Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613), Utc.ymd(2016, 11, 1).and_hms_milli(23, 25, 36, 923)],
            asks: vec![1.1234, 1.1204],
            bids: vec![1.1222, 1.1195]
        }));
        assert_eq!(rxg.recv().expect("Failed to recieve"), None);
    }

    #[test]
    fn two_different_time_frames() {
        let (txf, rxf) = channel();
        let (txg, rxg) = channel();
        txf.send(Some(InputRow {
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613),
            ask: 1.1234,
            bid: 1.1222
        })).expect("Could not send None");
        txf.send(Some(InputRow {
            datetime: Utc.ymd(2016, 11, 2).and_hms_milli(23, 25, 36, 923),
            ask: 1.1204,
            bid: 1.1195
        })).expect("Could not send None");
        txf.send(None).expect("Could not send None");
        grouper(txg, rxf, TimeFrame::new(1, TimeUnit::Day));
        assert_eq!(rxg.recv().expect("Failed to recieve"), Some(TickGroup {
            datetimes: vec![Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 613)],
            asks: vec![1.1234],
            bids: vec![1.1222]
        }));
        assert_eq!(rxg.recv().expect("Failed to recieve"), Some(TickGroup {
            datetimes: vec![Utc.ymd(2016, 11, 2).and_hms_milli(23, 25, 36, 923)],
            asks: vec![1.1204],
            bids: vec![1.1195]
        }));
        assert_eq!(rxg.recv().expect("Failed to recieve"), None);
    }
}
