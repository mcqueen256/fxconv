use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::io::prelude::*;
use std::thread;


/// From the input files, generates lines from file
pub fn create(input_file: File) -> (thread::JoinHandle<()>, Receiver<Option<(usize, String)>>) {
    let (tx_rows, rx_rows) = channel();
    let t = thread::Builder::new().name("producer".to_string()).spawn(move || {
        line_producer(input_file, tx_rows);
    });
    (t.expect("Thread did not spawn correctly"), rx_rows)
}

fn line_producer(mut file: File, tx_rows: Sender<Option<(usize, String)>>) {
    let size = file.metadata().expect("Cannot find file metadata").len() as usize;
    let mut text: Vec<u8> = vec![0; size];
    file.read_exact(&mut text).expect(&format!("Could not read input file"));
    let contents = String::from_utf8(text).expect("Could not convert bytes to string");

    for (line_number, line) in contents.split('\n').enumerate() {
        let line = line.trim();
            // skip empty line
            if line.len() == 0 {
                continue;
            }
        tx_rows.send(Some((line_number + 1, String::from(line)))).expect("Could not send row data from the producer");
    }
    tx_rows.send(None).expect("Could not send None from the producer");
}
