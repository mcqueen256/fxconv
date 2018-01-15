use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::fmt::Display;

use chrono::prelude::*;

use fxconv::TickDescription;

#[derive(Debug)]
#[derive(PartialEq)]
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
                eprintln!("Line {}, datetime ({}) incorrectly formatted:'{}' -> '{}', {}", line_number, unit, elm, a, error);
                exit(1);
            }
        },
        None => {
            let msg = format!("Line {}, datetime ({}) incorrectly formatted (not found): {}", line_number, unit, elm);
            panic!(msg);
        }
    }
}

/// From the input files, generates rows of the correct format
pub fn producer(mut input_files: Vec<File>, tick: Vec<TickDescription>, tx_rows: Sender<Option<InputRow>>) {
    for file in input_files.iter_mut() {
        let size = file.metadata().expect("Cannot find file metadata").len() as usize;
        let mut text: Vec<u8> = vec![0; size];
        file.read_exact(&mut text).expect(&format!("Could not read input file"));
        let contents = String::from_utf8(text).expect("Could not convert bytes to string");

        for (line_number, line) in contents.split('\n').enumerate() {
            let line = line.trim();
            // Skip empty lines
            if line.len() == 0 {
                continue;
            }

            ///////
            tx_rows.send(Some(row)).expect("Could not send row data from the producer");
        }
        tx_rows.send(None).expect("Could not send None from the producer");
        // Have all lines in the file converted in `rows` variables

    }
}

/// From the input files, generates rows of the correct format
pub fn create(input_files: Vec<File>, tick: Vec<TickDescription>) -> (thread::JoinHandle<()>, Receiver<Option<InputRow>>) {
    let (tx_rows, rx_rows) = channel(); // TODO:: make buffered channel with configurable limit
    let t = thread::Builder::new().name("producer".to_string()).spawn(move || {
        producer(input_files, tick, tx_rows);
    });
    (t.expect("Thread did not spawn correctly"), rx_rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_content(content: &str, file_name: &str) -> Receiver<Option<InputRow>> {
        use std::fs::OpenOptions;
        use std::fs::remove_file;
        let rx: Receiver<Option<InputRow>>;
        {
            let mut f = OpenOptions::new().create(true).write(true).open(file_name).expect("Unable to create file for testing");
            f.write(content.as_bytes()).expect("Failed write to file");
            f.flush().expect("Failed flush");
        }
        {
            let f = OpenOptions::new().read(true).open(file_name).expect("Unable to open file for testing");
            let (tx_rows, rx_rows) = channel();
            producer(vec![f], vec![], tx_rows);
            rx = rx_rows;
        }
        remove_file(file_name).expect("Unable to delete the file 'foo.txt'");
        rx
    }

    #[test]
    #[should_panic(expected = "like this")]
    fn test_panic() {
    }

    // #[test]
    // fn create_process_none() {
    //     let rx = with_content("", "create_process_none");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }

    // #[test]
    // fn create_process_one() {
    //     let rx = with_content("AUD/USD,20161101 22:30:03.617,0.76542,0.76532\n", "create_process_one");
    //     let row: InputRow = rx.recv().expect("Failed recv").expect("Is None");
    //     let dt = Utc.ymd(2016, 11, 1).and_hms_milli(22, 30, 3, 617);
    //     let ask = 0.76542;
    //     let bid = 0.76532;
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    //     assert_eq!(row.datetime, dt);
    //     assert_eq!(row.ask, ask);
    //     assert_eq!(row.bid, bid);
    //     use std::fs::remove_file;
    //     remove_file("create_process_one").expect("Unable to delete the file 'foo.txt'");
    // }

    // #[test]
    // fn crate_skip_empty_line() {
    //     let rx = with_content("\n", "crate_skip_empty_line");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: ',,,'")]
    // fn create_no_data_date() {
    //     with_content(",,,\n", "create_no_data_date");
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:03.617,,0.76543'")]
    // fn create_no_data_ask() {
    //     with_content("AUD/USD,20161101 22:30:03.617,,0.76543\n", "create_no_data_ask");
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:03.617,0.76543,'")]
    // fn create_no_data_bid() {
    //     with_content("AUD/USD,20161101 22:30:03.617,0.76543,\n", "create_no_data_bid");
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: ',,'")]
    // fn create_no_data_and_less_columns() {
    //     let rx = with_content(",,\n", "create_no_data_and_less_columns");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: ',,,,,'")]
    // fn create_no_data_and_more_columns() {
    //     let rx = with_content(",,,,,\n", "create_no_data_and_more_columns");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:03.617,0.76543,0.76542,this,that'")]
    // fn create_more_columns() {
    //     let rx = with_content("AUD/USD,20161101 22:30:03.617,0.76543,0.76542,this,that\n", "create_more_columns");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }
    //
    // #[test]
    // #[should_panic(expected = "Invalid line 1: 'AUD/USD,20161101 22:30:03.617,0.76542'")]
    // fn create_missing_column() {
    //     let rx = with_content("AUD/USD,20161101 22:30:03.617,0.76542\n", "create_missing_column");
    //     assert_eq!(rx.recv().expect("Failed recv"), None);
    // }
}
