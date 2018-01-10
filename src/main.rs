#[macro_use(c)]
extern crate cute;

extern crate clap;
extern crate assert_cli;
extern crate quickersort;
extern crate chrono;

mod cliparser;
mod market;
mod converter;

use converter::Converter;

// #[derive(Debug)]
// struct DataSet {
//     names: Vec<String>,
//     columns: Vec<Vec<f64>>,
//     width: usize,
//     precision: usize
// }
//
// impl DataSet {
//     fn new(names: Vec<&str>) -> DataSet {
//         let names = c![String::from(n), for n in names.into_iter()];
//         let columns = c![Vec::new(), for _n in names.iter()];
//         DataSet { names: names, columns: columns, width: 10, precision: 5 }
//         // TODO: Change the width to a cmd line variable, same with precision
//     }
//
//
// }
//
// use std::fmt;
//
// impl fmt::Display for DataSet {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // TODO: change the join to a variable argument --header-delimeter
//         let header: String = c![format!("{:<10}", n), for n in self.names.iter()].join(" ");
//         write!(f, "{}", header)
//     }
// }

fn main() {
    // let mut ds = DataSet::new(vec!["this", "that", "the", "other"]);
    // println!("ds:\n{}", ds);
    // cliparser::parse();

    let mut converter = Converter::new();
    converter.run();

    println!("ddd {}", {1});
}
