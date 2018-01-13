
pub enum AskBidOption {
    AskOnly,
    AskFirst,
    BidOnly,
    BidFirst
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum TickDescription {
    DateTime,
    Ask,
    Bid,
    Filler
}

#[derive(Debug)]
pub enum AskBid {
    Ask,
    Bid
}
