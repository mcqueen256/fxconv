use std::thread;
use std::sync::mpsc::{channel, Receiver};
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::fmt::Display;

use chrono::prelude::*;

use fxconv::TickDescription;

#[derive(Debug)]
pub struct InputRow {
    pub datetime: DateTime<Utc>,
    pub ask: f32,
    pub bid: f32
}

/// Extracts the number form the string, if there is an error, report it
fn extract_number<T>(number_str: Option<&str>, line_number: usize, elm: &str, unit: &str) -> T
    where T: FromStr + Display, <T as FromStr>::Err: Display {
    match number_str {
        Some(a) => match a.parse::<T>() {
            Ok(n) => n,
            Err(error) => {
                eprintln!("Error: Line {}, datetime ({}) incorrectly formatted:'{}' -> '{}', {}", line_number, unit, elm, a, error);
                exit(1);
            }
        },
        None => {
            eprintln!("Error: Line {}, datetime ({}) incorrectly formatted (not found): {}", line_number, unit, elm);
            exit(1);
        }
    }
}

/// From the input files, generates rows of the correct format
pub fn create(mut input_files: Vec<File>, tick: Vec<TickDescription>) -> (thread::JoinHandle<()>, Receiver<Option<InputRow>>) {
    let (tx_rows, rx_rows) = channel(); // TODO:: make buffered channel with configurable limit
    let t = thread::spawn(move || {
        for file in input_files.iter_mut() {
            let size = file.metadata().unwrap().len() as usize;
            let mut text: Vec<u8> = vec![0; size];
            match file.read_exact(&mut text) {
                Ok(_) => {},
                Err(error) => {
                    eprintln!("Error: Could not read input file: {}", error);
                }
            }
            let contents = String::from_utf8(text).unwrap();
            //let mut rows: VecDeque<InputRow> = VecDeque::new();
            for (line_number, line) in contents.split('\n').enumerate() {
                if line.trim().len() == 0 {
                    continue;
                }
                //TODO: check for lines like ",,,,"
                let line = line.trim();
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
                            ask = match elm.parse::<f32>() {
                                Ok(f) => Some(f),
                                Err(error) => {
                                    eprintln!("Error: Line {}, column {} not a number: {}", line_number, elm, error);
                                    exit(1);
                                }
                            }
                        },
                        TickDescription::Bid => {
                            bid = match elm.parse::<f32>() {
                                Ok(f) => Some(f),
                                Err(error) => {
                                    eprintln!("Error: Line {}, column {} not a number: {}", line_number, elm, error);
                                    exit(1);
                                }
                            }
                        },
                        TickDescription::Filler => {/* skip */}
                    }
                }

                let datetime: DateTime<Utc> = datetime.unwrap();
                let ask: f32 = ask.unwrap();
                let bid: f32 = bid.unwrap();
                let row = InputRow { datetime: datetime, ask: ask, bid: bid };
                tx_rows.send(Some(row)).unwrap();
            }
            tx_rows.send(None).unwrap();
            // Have all lines in the file converted in `rows` variables

        }
    });
    (t, rx_rows)
}
