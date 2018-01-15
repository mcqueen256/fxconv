use chrono::prelude::*;
use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;
use std::thread;
use std::sync::mpsc::channel;
use time::Duration;
use producer::InputRow;
use std::sync::mpsc::{Receiver};

pub struct TickGroup {
    pub datetimes: Vec<DateTime<Utc>>,
    pub asks: Vec<f32>,
    pub bids: Vec<f32>
}

pub fn create(rx_producer: Receiver<Option<InputRow>>, time_frame: TimeFrame)  -> (thread::JoinHandle<()>, Receiver<Option<TickGroup>>) {
    let (tx_grouper, rx_grouper) = channel(); // TODO:: make buffered channel with configurable limit
    let grouper_thread = thread::spawn(move || {

        // Select unit measurment
        let duration_func = match  *time_frame.unit() {
            TimeUnit::Second => Duration::seconds,
            TimeUnit::Minute => Duration::minutes,
            TimeUnit::Hour => Duration::hours,
            TimeUnit::Day => Duration::days,
            TimeUnit::Week => Duration::weeks
        };

        // to store the data in the frame
        let mut group = TickGroup {
            datetimes: Vec::new(),
            asks: Vec::new(),
            bids: Vec::new()
        };
        let mut first: Option<DateTime<Utc>> = None; // first datetime in timeframe

        while let Some(row) = rx_producer.recv().expect("Unable to receive from channel") {
            if first == None {
                first = Some(row.datetime);
            }
            let over_timeframe = |a: DateTime<Utc>,b: DateTime<Utc>| {
                a.signed_duration_since(b) >= duration_func(time_frame.len() as i64)
            };
            if over_timeframe(row.datetime, first.unwrap()) {
                while over_timeframe(row.datetime, first.unwrap()) {
                    first = Some(first.unwrap() + duration_func(time_frame.len() as i64));
                }
                tx_grouper.send(Some(group)).unwrap();
                // reset the group
                group = TickGroup {
                    datetimes: Vec::new(),
                    asks: Vec::new(),
                    bids: Vec::new()
                };
            }
            group.datetimes.push(row.datetime);
            group.asks.push(row.ask);
            group.bids.push(row.bid);
        }
        tx_grouper.send(Some(group)).unwrap();
        tx_grouper.send(None).unwrap();
    });
    (grouper_thread, rx_grouper)
}
