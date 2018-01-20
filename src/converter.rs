use chrono::prelude::*;
use fxconv::AskBidOption;
use std::thread;
use std::sync::mpsc::channel;
use fxconv::AskBid;
use grouper::TickGroup;
use std::sync::mpsc::{Receiver};

pub struct Row {
    pub datetime: DateTime<Utc>,
    pub column_data: Vec<f32>
}

// Return the first value of the vector
fn open(column: & Vec<f32>) -> f32 {
    column.iter().next().unwrap().clone()
}

// Return the highest value of the vector
fn high(column: & Vec<f32>) -> f32 {
    column.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone()
}

//  Return the lowest value fo the vector
fn low(column: & Vec<f32>) -> f32 {
    column.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().clone()
}

// Return the last value of the vector
fn close(column: & Vec<f32>) -> f32 {
    column.iter().rev().next().unwrap().clone()
}

// Create an output row from the input data
fn process(column_structure: &[AskBid], rows_ask: & Vec<f32>, rows_bid: & Vec<f32>) -> Vec<f32> {
    let mut row: Vec<f32> = Vec::new();
    for group in column_structure {
        match *group {
            AskBid::Ask => {
                row.push(open(rows_ask));
                row.push(high(rows_ask));
                row.push(low(rows_ask));
                row.push(close(rows_ask));
            },
            AskBid::Bid => {
                row.push(open(rows_bid));
                row.push(high(rows_bid));
                row.push(low(rows_bid));
                row.push(close(rows_bid));
            }
        };
    }
    row
}

// Create the converter
pub fn create(rx_grouper: Receiver<Option<TickGroup>>, ask_bid: Option<AskBidOption>)  -> (thread::JoinHandle<()>, Receiver<Option<Row>>) {
    // Build the conversion structure
    let column_structure: &[AskBid] = match ask_bid {
        Some(AskBidOption::AskOnly) => &[AskBid::Ask],
        Some(AskBidOption::BidOnly) => &[AskBid::Bid],
        Some(AskBidOption::BidFirst) => &[AskBid::Bid, AskBid::Ask],
        _ => &[AskBid::Ask, AskBid::Bid]
    };

    let (tx_converter, rx_converter) = channel();
    let converter_thread = thread::Builder::new().name("converter".to_string()).spawn(move || {

        while let Some(group) = rx_grouper.recv().expect("Unable to receive from channel") {

            let row = Row {
                datetime: *group.datetimes.iter().next().unwrap(),
                column_data: process(column_structure, & group.asks, & group.bids)
            };
            tx_converter.send(Some(row)).unwrap();
        }
        tx_converter.send(None).unwrap();
    });

    (converter_thread.expect("Thread did not spawn correctly"), rx_converter)
}
