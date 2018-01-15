extern crate clap;
extern crate chrono;
extern crate time;
#[macro_use(c)]
extern crate cute;
extern crate rand;


mod cliparser;
mod fxconv;
mod market;
mod line_producer;
// mod producer;
mod settings;
// mod converter;
// mod grouper;
// mod collector;

use std::fs::File;
use std::thread;
use std::process::exit;

use chrono::prelude::*;

use market::timeframe::TimeFrame;
use fxconv::AskBidOption;
use fxconv::TickDescription;
use cliparser::parse;

use std::io::prelude::*;

fn main() {
    // parse and extract application settings (see --help)
    let matches = parse();
    let time_frame: TimeFrame = settings::time_frame(&matches);
    let mut output_file: File = settings::output_file(&matches);
    let input_files: Vec<File> = settings::input_files(&matches);
    let ask_bid: Option<AskBidOption> = settings::ask_bid(&matches);
    let headers: bool = settings::headers(&matches);
    let precision: Option<u32> = settings::precision(&matches);
    let start: Option<DateTime<Utc>> = settings::start(&matches);
    let end: Option<DateTime<Utc>> = settings::end(&matches);
    let tick: Vec<TickDescription> = settings::tick(&matches);

    // start the file reader / input data producer
    // let (producer, rx)  = producer::create(input_files, tick);
    // let (grouper, rx)   = grouper::create(rx, time_frame);
    // let (converter, rx) = converter::create(rx, ask_bid);

    // while let Some(mut row) = rx.recv().unwrap() {
    //     let mut line: Vec<String> = Vec::new();
    //     line.push(row.datetime.to_string());
    //     for col in row.column_data.iter_mut() {
    //         line.push(col.to_string());
    //     }
    //     let line = line.join(",");
    //     let line = line.as_bytes();
    //     output_file.write(line).unwrap();
    //     output_file.write(b"\n").unwrap();
    // }
    // handle(producer);
    // handle(grouper);
    // handle(converter);
}

/// Handle thread joins
pub fn handle(t: thread::JoinHandle<()>) {
    let r: thread::Result<()> = t.join();
    match r {
        Ok(_) => {},
        Err(e) => {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                eprintln!("Error: {}", e);
            } else {
                eprintln!("Unkown Error: {:?}", e);
            }
            exit(1);
        }
    }
}
