use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::fmt::Display;

use chrono::prelude::*;

use fxconv::TickDescription;

pub fn create(rx_producer: Receiver<Option<&str>>, tick: Vec<TickDescription>)  -> (thread::JoinHandle<()>, Receiver<Option<TickGroup>>) {
    let (tx_collector, rx_collector) = channel(); // TODO:: make buffered channel with configurable limit
    let grouper_thread = thread::spawn(move || {
        while let Some(line) = rx_producer.recv().expect("Unable to receive from channel") {
            let mut datetime: Option<DateTime<Utc>> = None;
            let mut ask: Option<f32> = None;
            let mut bid: Option<f32> = None;

            for (desc, elm) in tick.iter().zip(line.split(',')) {
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
                        let d_str = String::from(elm);
                        let year: i32 = extract_number(d_str.get(0..4), line_number, elm, "year");
                        let month: u32 = extract_number(d_str.get(4..6), line_number, elm, "month");
                        let day: u32 = extract_number(d_str.get(6..8), line_number, elm, "day");
                        let hour: u32 = extract_number(d_str.get(9..11), line_number, elm, "hour");
                        let minute: u32 = extract_number(d_str.get(12..14), line_number, elm, "minute");
                        let second: u32 = extract_number(d_str.get(15..17), line_number, elm, "second");
                        let millis: u32 = extract_number(d_str.get(18..), line_number, elm, "millis");
                        datetime = Some(Utc.ymd(year, month, day).and_hms_milli(hour, minute, second, millis));
                    },
                    TickDescription::Ask => {
                        ask = Some(elm.parse::<f32>().expect(&format!("Line {}, column {} not a number", line_number, elm)));
                    },
                    TickDescription::Bid => {
                        bid = Some(elm.parse::<f32>().expect(&format!("Line {}, column {} not a number", line_number, elm)));
                    },
                    TickDescription::Filler => {/* skip */}
                }
            }


            if datetime == None || ask == None || bid == None {
                panic!("Invalid line {}: '{}'", line_number + 1, line);
            }
            // errors should not occer
            let datetime: DateTime<Utc> = datetime.expect("Could not extract datetime, was None");
            let ask: f32 = ask.expect("Could not extract ask, was None");
            let bid: f32 = bid.expect("Could not extract bid, was None");
            let row = InputRow { datetime: datetime, ask: ask, bid: bid };
            tx_rows.send(Some(row)).expect("Could not send row data from the producer");
        }
        tx_rows.send(None).expect("Could not send None from the producer");
    });

}
