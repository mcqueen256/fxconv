use std::fs::File;
use std::process::exit;
use std::path::Path;
use std::io;
use std::fs::OpenOptions;

use chrono::prelude::*;
use clap::ArgMatches;

use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;
use fxconv::AskBidOption;
use fxconv::TickDescription;

pub fn time_frame(matches: &ArgMatches) -> TimeFrame {
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
        _ => {
            eprintln!("Error: Unit not valid, see ARGS/TIMEFRAME in --help");
            exit(1);
        }
    };
    // Set time_frame
    TimeFrame::new( length, unit )
}

pub fn output_file(matches: &ArgMatches) -> File {
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
}

pub fn input_files(matches: &ArgMatches) -> Vec<File> {
    let input_names: Vec<String> = matches.values_of("inputs").unwrap().map(String::from).collect();
    let mut files: Vec<File> = Vec::new();
    for name in input_names.into_iter() {
        let file = match File::open(name.as_str()) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Error: Connot open file '{}': {}", name, error);
                exit(1);
            }
        };
        files.push(file);
    }
    files
}

pub fn ask_bid(matches: &ArgMatches) -> Option<AskBidOption> {
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
}

pub fn headers(matches: &ArgMatches) -> bool{
    matches.is_present("headers")
}

pub fn precision(matches: &ArgMatches) -> Option<u32> {
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

}

pub fn start(matches: &ArgMatches) -> Option<DateTime<Utc>> {
    if let Some(datetime) = matches.value_of("start") {
        match datetime.parse::<DateTime<Utc>>() {
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
}

pub fn end(matches: &ArgMatches) -> Option<DateTime<Utc>> {
    if let Some(datetime) = matches.value_of("end") {
        match datetime.parse::<DateTime<Utc>>() {
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

pub fn tick(matches: &ArgMatches) -> Vec<TickDescription> {
    let mut description: Vec<TickDescription> = Vec::new();
    if let Some(tick) = matches.value_of("tick") {
        for c in tick.chars() {
            match c {
                'd' => {
                    if description.iter().any(|v| *v == TickDescription::DateTime) {
                        eprintln!("Error: --tick option contains duplicat 'd' values");
                        exit(1);
                    }
                    description.push(TickDescription::DateTime);
                },
                'a' => {
                    if description.iter().any(|v| *v == TickDescription::Ask) {
                        eprintln!("Error: --tick option contains duplicat 'a' values");
                        exit(1);
                    }
                    description.push(TickDescription::Ask);
                },
                'b' => {
                    if description.iter().any(|v| *v == TickDescription::Bid) {
                        eprintln!("Error: --tick option contains duplicat 'b' values");
                        exit(1);
                    }
                    description.push(TickDescription::Bid);
                },
                'x' => {
                    description.push(TickDescription::Filler);
                },
                _ => {
                    eprintln!("Error: --tick option contains invalid value: '{}'", c);
                    exit(1);
                }
            }
        }
    }
    if ! description.iter().any(|v| *v == TickDescription::DateTime) {
        eprintln!("Error: --tick option does not contain 'd' value");
        exit(1);
    }
    if ! description.iter().any(|v| *v == TickDescription::Ask) {
        eprintln!("Error: --tick option does not contain 'a' value");
        exit(1);
    }
    if ! description.iter().any(|v| *v == TickDescription::Bid) {
        eprintln!("Error: --tick option does not contain 'b' value");
        exit(1);
    }
    description
}
