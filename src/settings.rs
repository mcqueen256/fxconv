use std::fs::File;
use std::process::exit;
use std::path::Path;
use std::io;
use std::fs::OpenOptions;

use clap::ArgMatches;

use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;
use fxconv::AskBidOption;
use formatter::TickDescription;

pub const CHANNEL_BUFFER: usize = 1024*1024*16;

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
        let e = format!("Timeframe is incorrectly formatted: '{}'", tf);
        panic!(e);
    }
    // Set up time frame variables
    let length = tf[0..digits].to_string().parse::<usize>().unwrap();
    let unit = match unit {
        Some('s') => TimeUnit::Second,
        Some('m') => TimeUnit::Minute,
        Some('h') => TimeUnit::Hour,
        Some('d') => TimeUnit::Day,
        Some('w') => TimeUnit::Week,
        _ => panic!("Unit not valid, see ARGS/TIMEFRAME in --help")
    };
    // Set time_frame
    TimeFrame::new( length, unit )
}

pub fn output_file(matches: &ArgMatches) -> File {
    let name = matches.value_of("output").unwrap();
    let path = Path::new(name);
    //check if dir
    if path.file_name() == Option::None {
        panic!("Output file is not a regular file");
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
                        let e = format!("Not a valid option '{}' (use Yes or No)", answer.trim());
                        panic!(e);
                    }
                }
            }
        }
    }
    // write to file, overwrite if it already exists
    OpenOptions::new().create(true).write(true).open(name).expect(&format!("Could not open output file '{}'", name))
}

pub fn input_files(matches: &ArgMatches) -> Vec<File> {
    let input_names: Vec<String> = matches.values_of("inputs").unwrap().map(String::from).collect();
    let mut files: Vec<File> = Vec::new();
    for name in input_names.into_iter() {
        let file = File::open(name.as_str()).expect(&format!("Could not open output file '{}'", name));
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

pub fn tick(matches: &ArgMatches) -> Vec<TickDescription> {
    let mut description: Vec<TickDescription> = Vec::new();
    if let Some(tick) = matches.value_of("tick") {
        for c in tick.chars() {
            match c {
                'd' => {
                    if description.iter().any(|v| *v == TickDescription::DateTime) {
                        panic!("--tick option contains duplicat 'd' values");
                    }
                    description.push(TickDescription::DateTime);
                },
                'a' => {
                    if description.iter().any(|v| *v == TickDescription::Ask) {
                        panic!("--tick option contains duplicat 'a' values");
                    }
                    description.push(TickDescription::Ask);
                },
                'b' => {
                    if description.iter().any(|v| *v == TickDescription::Bid) {
                        panic!("--tick option contains duplicat 'b' values");
                    }
                    description.push(TickDescription::Bid);
                },
                'x' => {
                    description.push(TickDescription::Filler);
                },
                _ => {
                    panic!("--tick option contains invalid value: '{}'", c);
                }
            }
        }
    }
    if ! description.iter().any(|v| *v == TickDescription::DateTime) {
        panic!("--tick option does not contain 'd' value");
    }
    if ! description.iter().any(|v| *v == TickDescription::Ask) {
        panic!("--tick option does not contain 'a' value");
    }
    if ! description.iter().any(|v| *v == TickDescription::Bid) {
        panic!("--tick option does not contain 'b' value");
    }
    description
}
