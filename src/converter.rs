
use std::process::exit;
use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;

use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use cliparser;
use quickersort;

use chrono::prelude::*;

enum AskBidOption {
    AskOnly,
    AskFirst,
    BidOnly,
    BidFirst
}

enum Gaps {
    Skip,
    Continue,
    SkipWeekends,
    Stop
}


fn open(dat: Vec<f32>) -> f32 {
    dat.get(0).unwrap().clone()
}

fn high(dat: Vec<f32>) -> f32 {
    let mut highest: f32 = dat.get(0).unwrap().clone();
    for i in dat.into_iter() {
        if i > highest {
            highest = i;
        }
    }
    highest
}

fn low(dat: Vec<f32>) -> f32 {
    let mut lowest: f32 = dat.get(0).unwrap().clone();
    for i in dat.into_iter() {
        if i < lowest {
            lowest = i;
        }
    }
    lowest
}

fn close(dat: Vec<f32>) -> f32 {
    dat.get(dat.len() - 1).unwrap().clone()
}

fn mean(dat: Vec<f32>) -> f32 {
    let mut sum: f32 = 0.0;
    for i in dat.iter() {
        sum += i;
    }
    sum / dat.len() as f32
}

fn operate(dat: Vec<f32>, op: char) -> f32 {
    if dat.len() == 0 {
        panic!("Cannot operate on empty data array");
    }
    match op {
        'o' => open(dat),
        'h' => high(dat),
        'l' => low(dat),
        'c' => close(dat),
        'm' => mean(dat),
        _ => {
            eprintln!("Error: Invalid operator '{}'", op);
            exit(1);
        }
    }
}

pub struct Converter {
    time_frame: TimeFrame,
    input_files: Vec<File>,
    output_file: File,
    input_delimiter: Option<String>,
    output_delimiter: Option<String>,
    ask_bid: Option<AskBidOption>,
    tick: bool,
    headers: bool,
    precision: Option<u32>,
    formatter: usize,
    start: Option<DateTime<FixedOffset>>,
    end: Option<DateTime<FixedOffset>>
}

impl Converter {
    pub fn new() -> Converter {
        let matches = cliparser::parse();

        println!("{:#?}", cliparser::parse());
        Converter {
            time_frame: {
                let tf = matches.value_of("timeframe").unwrap();
                // count the digits
                let mut digits = 0;
                let mut unit: Option<char> = None;
                for c in tf.chars() {
                    if c.is_digit(10) {
                        digits += 1;
                    }
                    else {
                        unit = Some(c);
                        break;
                    }
                }
                // check the string only has one non-digit char
                if tf.len() != digits + 1 {
                    eprintln!("Error: Timeframe is incorrectly formatted: '{}'", tf);
                    exit(1);
                }
                // Set up time frame variables
                let length = tf[0..digits].to_string().parse::<usize>().unwrap();
                let unit = match unit {
                    Some('s') => TimeUnit::Second,
                    Some('m') => TimeUnit::Minute,
                    Some('h') => TimeUnit::Hour,
                    Some('d') => TimeUnit::Day,
                    Some('w') => TimeUnit::Week,
                    Some('n') => TimeUnit::Month,
                    Some('y') => TimeUnit::Year,
                    _ => {
                        println!("Error: Unit not valid, see ARGS/TIMEFRAME in --help");
                        exit(1);
                    }
                };
                // Set time_frame
                TimeFrame::new( length, unit )
            },

            input_files: {
                let mut files: Vec<File> = Vec::new();
                let names: Vec<&str> = matches.values_of("inputs").unwrap().collect();
                for name in names.into_iter() {
                    let file = match File::open(name) {
                        Ok(file) => file,
                        Err(error) => {
                            eprintln!("Error: Connot open file '{}': {}", name, error);
                            exit(1);
                        }
                    };
                    files.push(file);
                }
                files
            },

            output_file: {
                let name = matches.value_of("output").unwrap();
                let path = Path::new(name);
                //check if dir
                if path.file_name() == Option::None {
                    eprintln!("Error: Output file is not a regular file");
                    exit(1);
                }
                // check if the output already exists
                if path.exists() {
                    if matches.is_present("overwrite") {
                        // Skip verification
                    }
                    else if matches.is_present("no-overwrite") {
                        eprintln!("Error: Output file already exists and specified --no-overwrite");
                        exit(1);
                    }
                    else {
                        // Ask before deleting data
                        let mut answer = String::new();
                        let yes = || {};
                        let no = || {
                            println!("Stopped program by user input.");
                            exit(1);
                        };
                        // get human input
                        loop {
                            println!("The file '{}' already exists. Overwrite? (Yes/No)", name);
                            io::stdin().read_line(&mut answer).expect("Error: Failed to read line");
                            match answer.trim() {
                                "y" => { yes(); break; },
                                "Y" => { yes(); break; },
                                "yes" => { yes(); break; },
                                "Yes" => { yes(); break; },
                                "n" => { no(); break; },
                                "N" => { no(); break; },
                                "no" => { no(); break; },
                                "No" => { no(); break; },
                                _ => {
                                    println!("Not a valid option '{}' (use Yes or No)", answer.trim());
                                }
                            }
                        }
                    }
                }
                // write to file, overwrite if it already exists
                match OpenOptions::new().create(true).write(true).open(name) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("Error: Could not open output file '{}': {}", name, error);
                        exit(1);
                    }
                }
            },
            input_delimiter: {
                if let Some(delimiter) = matches.value_of("input-delimiter") {
                    Some(String::from(delimiter))
                }
                else {
                    None
                }
            },
            output_delimiter: {
                if let Some(delimiter) = matches.value_of("output-delimiter") {
                    Some(String::from(delimiter))
                }
                else {
                    None
                }
            },
            ask_bid: {
                if matches.is_present("ask-only") {
                    Some(AskBidOption::AskOnly)
                }
                else if matches.is_present("bid-only") {
                    Some(AskBidOption::BidOnly)
                }
                else if matches.is_present("ask-first") {
                    Some(AskBidOption::AskFirst)
                }
                else if matches.is_present("bid-first") {
                    Some(AskBidOption::BidFirst)
                } else {
                    None
                }
            },
            formatter: {
                0
            },
            tick: matches.is_present("tick"),
            headers: matches.is_present("headers"),
            precision: {
                if let Some(precision) = matches.value_of("precision") {
                    let precision = match precision.parse::<u32>() {
                        Ok(p) => p,
                        Err(error) => {
                            eprintln!("Error: Precision is not a number {}", error);
                            exit(1);
                        }
                    };
                    Some(precision)
                }
                else {
                    None
                }

            },
            start: {
                if let Some(datetime) = matches.value_of("start") {
                    match DateTime::parse_from_str(datetime, "%Y/%m/%d %H:%M:%S") {
                        Ok(dt) => Some(dt),
                        Err(error) => {
                            eprintln!("Error: Start date incorrectly formatted: {}", error);
                            exit(1);
                        }
                    }
                }
                else {
                    None
                }
            },
            end: {
                if let Some(datetime) = matches.value_of("end") {
                    match DateTime::parse_from_str(datetime, "%Y/%m/%d %H:%M:%S") {
                        Ok(dt) => Some(dt),
                        Err(error) => {
                            println!("Error: End date incorrectly formatted: {}", error);
                            exit(1);
                        }
                    }
                }
                else {
                    None
                }
            }
        }
    }
    pub fn run(&mut self) {
        self.output_file.write(b"20161101 22:30:03.617,0.76541,0.76562,0.76531,0.76558,0.76551,0.76572,0.76541,0.76559\n").expect("Failed to write line");
        // process input data

        for file in self.input_files.iter_mut() {
            println!("!start");
            let size = file.metadata().unwrap().len() as usize;
            println!("size {}", size);
            let mut text: Vec<u8> = vec![0; size];
            match file.read_exact(&mut text) {
                Ok(_) => {},
                Err(error) => {
                    eprintln!("Error: Could not read input file: {}", error);
                }
            }
            let s = String::from_utf8(text).unwrap();
            
            //println!("'{}'", String::from_utf8(text).unwrap());
            println!("!end");
        }
    }
    fn read_line(line: &str) {}


}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
    }
}
