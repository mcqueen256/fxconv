
use std::process::exit;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::collections::VecDeque;

use chrono::prelude::*;

use cliparser::parse;
use market::timeframe::TimeFrame;
use market::timeframe::TimeUnit;

enum AskBidOption {
    AskOnly,
    AskFirst,
    BidOnly,
    BidFirst
}

#[derive(Debug)]
#[derive(PartialEq)]
enum TickDescription {
    DateTime,
    Ask,
    Bid,
    Filler
}

#[derive(Debug)]
enum AskBid {
    Ask,
    Bid
}

#[derive(Debug)]
enum ColumnDescription {
    DateTime,
    Open,
    High,
    Low,
    Close,
    Mean,
}

#[derive(Debug)]
struct Column<T> {
    desc: ColumnDescription,
    ask_bid: Option<AskBid>,
    data: VecDeque<T>
}

impl<T> Column<T> {
    fn new(desc: ColumnDescription, ask_bid: Option<AskBid>,) -> Column<T> {
        Column::<T> { desc: desc, ask_bid: ask_bid, data: VecDeque::new() }
    }

    fn name(&self) -> String {
        let mut name = String::new();
        name.push_str(match self.ask_bid {
            Some(AskBid::Ask) => "Ask ",
            Some(AskBid::Bid) => "Bid ",
            None => "",
        });
        name.push_str(match self.desc {
            ColumnDescription::DateTime => "DateTime",
            ColumnDescription::Open => "Open",
            ColumnDescription::High => "High",
            ColumnDescription::Low => "Low",
            ColumnDescription::Close => "Close",
            ColumnDescription::Mean => "Mean",
        });
        name
    }

}

#[derive(Debug)]
struct InputRow {
    datetime: DateTime<Utc>,
    ask: f32,
    bid: f32
}

pub struct FxTickConv {
    time_frame: TimeFrame,
    input_files: Vec<File>,
    output_file: File,
    ask_bid: Option<AskBidOption>,
    headers: bool,
    precision: Option<u32>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    tick: Vec<TickDescription>,
}



impl FxTickConv {
    pub fn new() -> FxTickConv {
        let matches = parse();
        FxTickConv {
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
            },
            end: {
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
            },
            tick: {
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
        }
    }

    pub fn run(&mut self) {
        let mut output_datetime_column: Column<DateTime<Utc>> =
            Column::new(ColumnDescription::DateTime, None);
        let mut output_columns: Vec<Column<f32>> = Vec::new();




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
            let contents = String::from_utf8(text).unwrap();
            let mut rows: VecDeque<InputRow> = VecDeque::new();
            for (line_number, line) in contents.split('\n').enumerate() {
                if line.trim().len() == 0 {
                    continue;
                }
                println!("{:?}", self.tick);
                println!("{:?}", line);
                let line = line.trim();
                let mut datetime: Option<DateTime<Utc>> = None;
                let mut ask: Option<f32> = None;
                let mut bid: Option<f32> = None;
                for (desc, elm) in self.tick.iter().zip(line.split(',')) {
                    println!("{:?}, '{}'", desc, elm);
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
                            let year = match d_str.get(0..4) {
                                Some(y) => match y.parse::<i32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (year) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, y, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (year) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let month = match d_str.get(4..6) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (month) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (month) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let day = match d_str.get(6..8) {
                                Some(d) => match d.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (day) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, d, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (day) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let hour = match d_str.get(9..11) {
                                Some(h) => match h.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (hour) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, h, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (hour) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let minute = match d_str.get(12..14) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (minutes) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (minutes) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let second = match d_str.get(15..17) {
                                Some(s) => match s.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (seconds) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, s, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (seconds) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let millis = match d_str.get(18..) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        println!("Error: Line {}, datetime (millis) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    println!("Error: Line {}, datetime (millis) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            Utc.ymd(2014, 7, 8).and_hms_milli(9, 10, 11, 12);
                            Utc.ymd(2014, 7, 8);

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
                rows.push_front(row);
            }

            // Have all lines in the file converted in `rows` variables
            println!("{:#?}", rows);

            println!("!end");
        }
    }


}
