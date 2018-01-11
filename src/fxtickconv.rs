
use std::process::exit;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc;
use std::cmp::Ord;

use chrono::prelude::*;
use time::Duration;

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
    Open,
    High,
    Low,
    Close
}

#[derive(Debug)]
struct Column {
    desc: ColumnDescription,
    ask_bid: AskBid,
    data: VecDeque<f32>
}

impl Column {
    fn new(desc: ColumnDescription, ask_bid: AskBid,) -> Column {
        Column { desc: desc, ask_bid: ask_bid, data: VecDeque::new() }
    }

    fn name(&self) -> String {
        let mut name = String::new();
        name.push_str(match self.ask_bid {
            AskBid::Ask => "Ask ",
            AskBid::Bid => "Bid "
        });
        name.push_str(match self.desc {
            ColumnDescription::Open => "Open",
            ColumnDescription::High => "High",
            ColumnDescription::Low => "Low",
            ColumnDescription::Close => "Close"
        });
        name
    }

    fn data(&mut self) -> &mut VecDeque<f32> {
        &mut self.data
    }

    fn ask_bid(&mut self) -> &AskBid {
        &self.ask_bid
    }

    fn desc(&mut self) -> &ColumnDescription {
        &self.desc
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
    output_file: File,
    ask_bid: Option<AskBidOption>,
    headers: bool,
    precision: Option<u32>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    row_producer: thread::JoinHandle<()>,
    row_reciver: mpsc::Receiver<Option<InputRow>>
}

//self.input_files
//self.tick



fn input_row_producer(input_names: Vec<String>, tick: Vec<TickDescription>) -> (thread::JoinHandle<()>, mpsc::Receiver<Option<InputRow>>) {
    let (tx_rows, rx_rows) = channel();
    let t = thread::spawn(move || {
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

        for file in files.iter_mut() {
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
                            let year = match d_str.get(0..4) {
                                Some(y) => match y.parse::<i32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (year) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, y, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (year) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let month = match d_str.get(4..6) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (month) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (month) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let day = match d_str.get(6..8) {
                                Some(d) => match d.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (day) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, d, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (day) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let hour = match d_str.get(9..11) {
                                Some(h) => match h.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (hour) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, h, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (hour) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let minute = match d_str.get(12..14) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (minutes) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (minutes) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let second = match d_str.get(15..17) {
                                Some(s) => match s.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (seconds) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, s, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (seconds) incorrectly formatted (not found): {}", line_number, elm);
                                    exit(1);
                                }
                            };
                            let millis = match d_str.get(18..) {
                                Some(m) => match m.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(error) => {
                                        eprintln!("Error: Line {}, datetime (millis) incorrectly formatted:'{}' -> '{}', {}", line_number, elm, m, error);
                                        exit(1);
                                    }
                                },
                                None => {
                                    eprintln!("Error: Line {}, datetime (millis) incorrectly formatted (not found): {}", line_number, elm);
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
                tx_rows.send(Some(row)).unwrap();
            }
            tx_rows.send(None).unwrap();
            // Have all lines in the file converted in `rows` variables

        }
    });
    (t, rx_rows)
}

fn add_ask(col: &mut Vec<Column>) {
    col.push(Column::new(ColumnDescription::Open, AskBid::Ask));
    col.push(Column::new(ColumnDescription::High, AskBid::Ask));
    col.push(Column::new(ColumnDescription::Low, AskBid::Ask));
    col.push(Column::new(ColumnDescription::Close, AskBid::Ask));
}

fn add_bid(col: &mut Vec<Column>) {
    col.push(Column::new(ColumnDescription::Open, AskBid::Bid));
    col.push(Column::new(ColumnDescription::High, AskBid::Bid));
    col.push(Column::new(ColumnDescription::Low, AskBid::Bid));
    col.push(Column::new(ColumnDescription::Close, AskBid::Bid));
}

impl FxTickConv {
    pub fn new() -> FxTickConv {
        let matches = parse();
        let input_names = matches.values_of("inputs").unwrap().map(String::from).collect();
        let tick = {
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
        };
        let (row_producer, row_reciver) = input_row_producer(input_names, tick);

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
                    _ => {
                        println!("Error: Unit not valid, see ARGS/TIMEFRAME in --help");
                        exit(1);
                    }
                };
                // Set time_frame
                TimeFrame::new( length, unit )
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
            row_producer: row_producer,
            row_reciver: row_reciver
        }
    }

    pub fn run(self) {
        // build output columns
        let mut output_datetimes: VecDeque<DateTime<Utc>> = VecDeque::new();
        let mut output_columns: Vec<Column> = Vec::new();
        match self.ask_bid {
            Some(AskBidOption::AskOnly) => { add_ask(&mut output_columns); },
            Some(AskBidOption::BidOnly) => { add_bid(&mut output_columns); },
            Some(AskBidOption::BidFirst) => { add_bid(&mut output_columns); add_ask(&mut output_columns); },
            _ => { add_ask(&mut output_columns); add_bid(&mut output_columns); }
        };

        //
        let mut rows_datetime: Vec<DateTime<Utc>> = Vec::new();
        let mut rows_ask: Vec<f32> = Vec::new();
        let mut rows_bid: Vec<f32> = Vec::new();

        let mut reference_datetime: Option<DateTime<Utc>> = None;
        let mut previous_datetime: Option<DateTime<Utc>> = None;

        while let Some(row) = self.row_reciver.recv().expect("Unable to receive from channel") {

            let mut new_tf = false;
            if let Some(reference) = reference_datetime {
                // check if the current datetime is over the timeframe

                match  *self.time_frame.unit() {
                    TimeUnit::Second => {
                        if row.datetime.signed_duration_since(reference) >= Duration::seconds(self.time_frame.len() as i64) {
                            reference_datetime = Some(reference + Duration::seconds(self.time_frame.len() as i64));
                            new_tf = true;
                        }
                    },
                    TimeUnit::Minute => {
                        if row.datetime.signed_duration_since(reference) >= Duration::minutes(self.time_frame.len() as i64) {
                            reference_datetime = Some(reference + Duration::minutes(self.time_frame.len() as i64));
                            new_tf = true;
                        }
                    },
                    TimeUnit::Hour => {
                        if row.datetime.signed_duration_since(reference) >= Duration::hours(self.time_frame.len() as i64) {
                            reference_datetime = Some(reference + Duration::hours(self.time_frame.len() as i64));
                            new_tf = true;
                        }
                    },
                    TimeUnit::Day => {
                        if row.datetime.signed_duration_since(reference) >= Duration::days(self.time_frame.len() as i64) {
                            reference_datetime = Some(reference + Duration::days(self.time_frame.len() as i64));
                            new_tf = true;
                        }
                    },
                    TimeUnit::Week => {
                        if row.datetime.signed_duration_since(reference) >= Duration::weeks(self.time_frame.len() as i64) {
                            reference_datetime = Some(reference + Duration::weeks(self.time_frame.len() as i64));
                            new_tf = true;
                        }
                    }
                };
            }
            else {
                // initialise the reference_datetime
                reference_datetime = Some(row.datetime);
                previous_datetime = Some(row.datetime)
            }

            if new_tf {
                // process all data
                output_datetimes.push_front(previous_datetime.unwrap_or(reference_datetime.unwrap()));
                for col in output_columns.iter_mut() {
                    let datum: f32 = match *col.ask_bid() {
                        AskBid::Ask => {
                            match *col.desc() {
                                ColumnDescription::Open => rows_ask.iter().next().unwrap().clone(),
                                ColumnDescription::High => rows_ask.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                                ColumnDescription::Low => rows_ask.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                                ColumnDescription::Close => rows_ask.iter().rev().next().unwrap().clone(),
                            }
                        },
                        AskBid::Bid => {
                            match *col.desc() {
                                ColumnDescription::Open => rows_bid.iter().next().unwrap().clone(),
                                ColumnDescription::High => rows_bid.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                                ColumnDescription::Low => rows_bid.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                                ColumnDescription::Close => rows_bid.iter().rev().next().unwrap().clone(),
                            }
                        }
                    };
                    col.data().push_front(datum);

                }
                rows_datetime.clear();
                rows_ask.clear();
                rows_bid.clear();
                previous_datetime = reference_datetime;
            }
            rows_datetime.push(row.datetime);
            rows_ask.push(row.ask);
            rows_bid.push(row.bid);
        }

        {
            // process all data
            output_datetimes.push_front(previous_datetime.unwrap_or(reference_datetime.unwrap()));
            for col in output_columns.iter_mut() {
                let datum: f32 = match *col.ask_bid() {
                    AskBid::Ask => {
                        match *col.desc() {
                            ColumnDescription::Open => rows_ask.iter().next().unwrap().clone(),
                            ColumnDescription::High => rows_ask.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                            ColumnDescription::Low => rows_ask.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                            ColumnDescription::Close => rows_ask.iter().rev().next().unwrap().clone(),
                        }
                    },
                    AskBid::Bid => {
                        match *col.desc() {
                            ColumnDescription::Open => rows_bid.iter().next().unwrap().clone(),
                            ColumnDescription::High => rows_bid.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                            ColumnDescription::Low => rows_bid.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone(),
                            ColumnDescription::Close => rows_bid.iter().rev().next().unwrap().clone(),
                        }
                    }
                };
                col.data().push_front(datum);

            }
        }

        self.row_producer.join().unwrap();

    }




}
