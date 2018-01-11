extern crate clap;
extern crate chrono;
extern crate time;

mod cliparser;
mod fxtickconv;
mod market;

use fxtickconv::FxTickConv;

fn main() {
    let mut app = FxTickConv::new();
    app.run();
}
