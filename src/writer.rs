use std::thread;
use std::fs::File;
use std::sync::mpsc::Receiver;
use std::io::prelude::*;

use converter::Row;

pub fn create(rx: Receiver<Option<Row>>, mut output_file: File)  -> thread::JoinHandle<()> {
    let t = thread::Builder::new().name("writer".to_string()).spawn(move || {
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
    });
    t.expect("Thread did not spawn correctly")
}
