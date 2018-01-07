
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

pub struct Converter {
    time_frame: TimeFrame,
    input_files: Vec<File>,
    output_file: File
    //Option<>
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
            }
        }
    }
    pub fn run(&self) {

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
