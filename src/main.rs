extern crate clap;
extern crate chrono;
extern crate time;
extern crate rand;
extern crate thread_id;
extern crate nix;


mod cliparser;
mod fxconv;
mod market;
mod line_producer;
mod formatter;
mod settings;
mod converter;
mod grouper;

use std::fs::File;
use std::thread;
use std::process::exit;
use std::panic;

use market::timeframe::TimeFrame;
use fxconv::AskBidOption;
use formatter::TickDescription;
use cliparser::parse;

use std::io::prelude::*;

fn main() {
    // setup error handling
    panic::set_hook(Box::new(|_info| { /* do nothing */ }));

    // run entire program in a thread to catch panics
    let phantom = thread::Builder::new().name("phantom_main".to_string()).spawn(move || {
        // parse and extract application settings (see --help)
        let matches = parse();
        let time_frame: TimeFrame = settings::time_frame(&matches);
        let mut output_file: File = settings::output_file(&matches);
        let input_files: Vec<File> = settings::input_files(&matches);
        let ask_bid: Option<AskBidOption> = settings::ask_bid(&matches);
        let headers: bool = settings::headers(&matches);
        let tick: Vec<TickDescription> = settings::tick(&matches);


                if headers {
                    let ask = "ask,ask,ask,ask";
                    let bid = "bid,bid,bid,bid";
                    let ohlc = "open,high,low,close";
                    let mut top = String::from(",");
                    let mut bottom = String::from("datetime,");
                    match ask_bid {
                        Some(AskBidOption::AskOnly) => {
                            top.push_str(ask);
                            bottom.push_str(ohlc);
                        },
                        Some(AskBidOption::BidOnly) => {
                            top.push_str(ask);
                            bottom.push_str(ohlc);
                        },
                        Some(AskBidOption::BidFirst) => {
                            top.push_str(bid);
                            top.push_str(",");
                            top.push_str(ask);
                            bottom.push_str(ohlc);
                            bottom.push_str(",");
                            bottom.push_str(ohlc);
                        },
                        _ => {
                            top.push_str(ask);
                            top.push_str(",");
                            top.push_str(bid);
                            bottom.push_str(ohlc);
                            bottom.push_str(",");
                            bottom.push_str(ohlc);
                        },
                    }
                    output_file.write(top.as_bytes()).expect("Cannot write to output");
                    output_file.write(b"\n").expect("Cannot write to output");
                    output_file.write(bottom.as_bytes()).expect("Cannot write to output");
                    output_file.write(b"\n").expect("Cannot write to output");
                }

        // start the file reader / input data producer
        for file in input_files.into_iter() {
            let (line_producer, rx) = line_producer::create(file);
            let (formatter, rx) = formatter::create(rx, tick.clone());
            let (grouper, rx)   = grouper::create(rx, time_frame.clone());
            let (converter, rx) = converter::create(rx, ask_bid.clone());

            while let Some(mut row) = rx.recv().unwrap() {
                let mut line: Vec<String> = Vec::new();
                line.push(row.datetime.to_string());
                for col in row.column_data.iter_mut() {
                    line.push(col.to_string());
                }
                let line = line.join(",");
                let line = line.as_bytes();
                output_file.write(line).expect("Could not write to file");
                output_file.write(b"\n").expect("Could not write to file");
            }

            handle(line_producer);
            handle(formatter);
            handle(grouper);
            handle(converter);
        }
    });
    handle(phantom.expect("Thread did not spawn correctly"));
}

fn handle(t: thread::JoinHandle<()>) {
    match sub_handle(t) {
        Some(text) => {
            eprintln!("{}", text);
            exit(1);
        },
        None => {}
    };
}

/// Handle thread joins
fn sub_handle(t: thread::JoinHandle<()>) -> Option<String> {
    let r: thread::Result<()> = t.join();
    match r {
        Ok(_) => None,
        Err(e) => {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                Some(String::from(format!("Error: {}", e)))
            } else {
                Some(String::from(format!("Unkown Error: {:?}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_thread() {
        panic::set_hook(Box::new(|_info| {}));
        let t = thread::spawn(move || {
            panic!("oops! I slipped..");
        });
        assert_eq!(sub_handle(t), Some(String::from("Error: oops! I slipped..")));
    }
}
