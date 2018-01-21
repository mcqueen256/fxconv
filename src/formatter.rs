use std::sync::mpsc::{channel, Receiver, Sender};
use std::str::FromStr;
use std::fmt::Display;
use std::thread;
use chrono::prelude::*;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TickDescription {
    DateTime,
    Ask,
    Bid,
    Filler
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct InputRow {
    pub datetime: DateTime<Utc>,
    pub ask: f32,
    pub bid: f32
}

/// From the input lines, generates tick data
pub fn create(rx_producer: Receiver<Option<(usize, String)>>, tick: Vec<TickDescription>) -> (thread::JoinHandle<()>, Receiver<Option<InputRow>>) {
    let (tx_ticks, rx_ticks) = channel();
    let t = thread::Builder::new().name("formatter".to_string()).spawn(move || {
        formatter(tx_ticks, rx_producer, tick);
    });
    (t.expect("Thread did not spawn correctly"), rx_ticks)
}

/// Extracts the number form the string, if there is an error, report it
fn extract<T>(number_str: Option<&str>, line_number: usize, elm: &str, unit: &str) -> T
    where T: FromStr + Display, <T as FromStr>::Err: Display {
    match number_str {
        Some(a) => match a.parse::<T>() {
            Ok(n) => n,
            Err(_) => {
                panic!("Line {}, {} data incorrectly formatted:'{}' -> '{}'", line_number, unit, elm, a);
            }
        },
        None => {
            let msg = format!("Line {}, {} data incorrectly formatted (not found): {}", line_number, unit, elm);
            panic!(msg);
        }
    }
}

/// Invarent: line must not be empty
fn formatter(tx_formatter: Sender<Option<InputRow>>, rx_producer: Receiver<Option<(usize, String)>>, tick: Vec<TickDescription>) {
    while let Some((line_number, line)) = rx_producer.recv().expect("Unable to receive from channel") {
        let mut datetime: Option<DateTime<Utc>> = None;
        let mut ask: Option<f32> = None;
        let mut bid: Option<f32> = None;

        let cols = line.split(',');
        if cols.clone().collect::<Vec<_>>().len() != tick.len() {
            panic!("Invalid line {}: '{}'", line_number, line)
        }
        for (desc, elm) in tick.iter().zip(cols) {
            match *desc {
                TickDescription::DateTime => {
                    // "20161101 22:30:03.617"
                    //  ____ [0..5] year
                    //      __ [5..7] month
                    //        __[7..9] day
                    //           __ [10..12] hour
                    //              __ [13..15] minute
                    //                 __ [16..18] second
                    //                    __ [19..] millis
                    let year: i32 = extract(elm.get(0..4), line_number, elm, "year");
                    let month: u32 = extract(elm.get(4..6), line_number, elm, "month");
                    let day: u32 = extract(elm.get(6..8), line_number, elm, "day");
                    let hour: u32 = extract(elm.get(9..11), line_number, elm, "hour");
                    let minute: u32 = extract(elm.get(12..14), line_number, elm, "minute");
                    let second: u32 = extract(elm.get(15..17), line_number, elm, "second");
                    let millis: u32 = extract(elm.get(18..), line_number, elm, "millis");
                    datetime = Some(Utc.ymd(year, month, day).and_hms_milli(hour, minute, second, millis));
                },
                TickDescription::Ask => {
                    ask = Some(elm.parse::<f32>().expect(&format!("Line {}, column {} not a number", line_number, elm)));
                },
                TickDescription::Bid => {
                    bid = Some(elm.parse::<f32>().expect(&format!("Line {}, column {} not a number", line_number, elm)));
                },
                TickDescription::Filler => { /* skip */ }
            }
        }


        if datetime == None || ask == None || bid == None {
            panic!("Invalid line {}: '{}'", line_number, line);
        }
        // errors should not occer
        let datetime: DateTime<Utc> = datetime.unwrap();
        let ask: f32 = ask.unwrap();
        let bid: f32 = bid.unwrap();
        let row = InputRow { datetime: datetime, ask: ask, bid: bid };
        tx_formatter.send(Some(row)).expect("Could not send row data from the producer");
    }
    tx_formatter.send(None).expect("Cannot send None");
}

#[cfg(test)]
mod test {
    use super::*;

    // helper method to generate a filter for the input line data
    fn gen_td() -> Vec<TickDescription> {
        vec![
            TickDescription::Filler,
            TickDescription::DateTime,
            TickDescription::Ask,
            TickDescription::Bid
        ]
    }

    #[test]
    fn normal_use_one_line() {
        let (tx, rx) = channel();
        let (txf, rxf) = channel();
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:05.632,0.76551,0.76541")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
        assert_eq!(rxf.recv().unwrap(), Some(InputRow{
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 632),
            ask: 0.76551,
            bid: 0.76541
        }));
        assert_eq!(rxf.recv().unwrap(), None);
    }

    #[test]
    fn normal_use_more_lines() {
        let (tx, rx) = channel();
        let (txf, rxf) = channel();
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:05.632,0.76551,0.76541")))).expect("Could not send line");
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:06.473,0.76555,0.76545")))).expect("Could not send line");
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:06.890,0.76549,0.76538")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
        assert_eq!(rxf.recv().unwrap(), Some(InputRow{
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 5, 632),
            ask: 0.76551,
            bid: 0.76541
        }));
        assert_eq!(rxf.recv().unwrap(), Some(InputRow{
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 6, 473),
            ask: 0.76555,
            bid: 0.76545
        }));
        assert_eq!(rxf.recv().unwrap(), Some(InputRow{
            datetime: Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 6, 890),
            ask: 0.76549,
            bid: 0.76538
        }));
        assert_eq!(rxf.recv().unwrap(), None);
    }

    #[test]
    #[should_panic(expected = "Line 1, year data incorrectly formatted (not found): ")]
    fn missing_datetime() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("AUD/USD,,0.76551,0.76541")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Invalid line 1: 'AUD/USD,0.76551,0.76541'")]
    fn missing_element() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("AUD/USD,0.76551,0.76541")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Invalid line 1: 'Line 1, column  not a number: ParseFloatError { kind: Empty }'")]
    fn missing_ask() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("Line 1, column  not a number: ParseFloatError { kind: Empty }")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:05.632,0.76551'")]
    fn missing_bid() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:05.632,0.76551")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Line 1, minute data incorrectly formatted:'20161101 230:05.632' -> ':0'")]
    fn faulty_datetime() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("AUD/USD,20161101 230:05.632,0.76551,0.76541")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:05.632,0.76551,0.76541,0.7364,0.9347'")]
    fn more_cols() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("AUD/USD,20161101 22:30:05.632,0.76551,0.76541,0.7364,0.9347")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Invalid line 1: '20161101 22:30:05.632,0.76551,0.76541'")]
    fn less_cols() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from("20161101 22:30:05.632,0.76551,0.76541")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }

    #[test]
    #[should_panic(expected = "Line 1, year data incorrectly formatted (not found): ")]
    fn empty_columns() {
        let (tx, rx) = channel();
        let (txf, _) = channel();
        tx.send(Some((1, String::from(",,,")))).expect("Could not send line");
        tx.send(None).expect("Cannot send None");
        formatter(txf, rx, gen_td());
    }
}
