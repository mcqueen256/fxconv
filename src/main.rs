extern crate clap;
extern crate chrono;

mod cliparser;
mod fxtickconv;
mod market;

use fxtickconv::FxTickConv;

fn main() {
    let mut app = FxTickConv::new();
    app.run();
}
