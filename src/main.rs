extern crate clap;
extern crate chrono;
extern crate time;
#[macro_use(c)]
extern crate cute;

mod cliparser;
mod fxtickconv;
mod market;

use fxtickconv::FxTickConv;

fn main() {
    let mut app = FxTickConv::new();
    app.run();
}
