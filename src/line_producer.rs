use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::io::prelude::*;
use std::thread;


/// From the input files, generates rows of the correct format
pub fn create(input_files: Vec<File>) -> (thread::JoinHandle<()>, Receiver<Option<(usize, String)>>) {
    let (tx_rows, rx_rows) = channel(); // TODO:: make buffered channel with configurable limit
    let t = thread::Builder::new().name("producer".to_string()).spawn(move || {
        line_producer(input_files, tx_rows);
    });
    (t.expect("Thread did not spawn correctly"), rx_rows)
}

fn line_producer(mut input_files: Vec<File>, tx_rows: Sender<Option<(usize, String)>>) {
    for file in input_files.iter_mut() {
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
    }
    tx_rows.send(None).expect("Could not send None from the producer");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use rand::random;
    //rand::thread_rng().gen_range(1, 101);

    // Helper function for creating files with designated content
    fn file_with(content: &str) -> File {
        let number = random::<u64>();
        let mut file_name_builder = String::from(number.to_string());
        file_name_builder.push_str(".test.tmp");
        let file_name = file_name_builder.as_str();

        {
            let mut file = OpenOptions::new().create(true).write(true).open(file_name).expect("Could not create test file");
            file.write(content.as_bytes()).expect("Failed write to file");
            file.flush().expect("Failed to flush");
        }

        OpenOptions::new().read(true).open(file_name).expect("Could not create test file")
    }

    #[test]
    fn no_line_one_file() {
        let empty_file = file_with("");
        let (tx, rx) = channel();
        line_producer(vec![empty_file], tx);
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn incomplete_line_one_file() {
        let empty_file = file_with("this line has no newline ending");
        let (tx, rx) = channel();
        line_producer(vec![empty_file], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("this line has no newline ending"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn no_line_two_file() {
        let (tx, rx) = channel();
        line_producer(vec![file_with(""), file_with("")], tx);
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn no_line_three_file() {
        let (tx, rx) = channel();
        line_producer(vec![file_with(""), file_with(""), file_with("")], tx);
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn one_line_one_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("this is a single line\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("this is a single line"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn one_line_two_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\n"),
            file_with("second\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn one_line_three_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\n"),
            file_with("second\n"),
            file_with("third\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("third"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn multiple_line_one_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\nsecond\nthird\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((3, String::from("third"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn multiple_line_two_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\nsecond\nthird\n"),
            file_with("fourth\nfifth\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((3, String::from("third"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("fourth"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("fifth"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn multiple_line_three_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\nsecond\nthird\n"),
            file_with("fourth\nfifth\n"),
            file_with("sixth\nseventh\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((3, String::from("third"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("fourth"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("fifth"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("sixth"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("seventh"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn lines_in_first_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with("first\nsecond\nthird\n"),
            file_with("")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("first"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("second"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((3, String::from("third"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

    #[test]
    fn lines_in_second_file() {
        let (tx, rx) = channel();
        line_producer(vec![
            file_with(""),
            file_with("fourth\nfifth\n")
        ], tx);
        assert_eq!(rx.recv().expect("recv failed"), Some((1, String::from("fourth"))));
        assert_eq!(rx.recv().expect("recv failed"), Some((2, String::from("fifth"))));
        assert_eq!(rx.recv().expect("recv failed"), None);
    }

}
