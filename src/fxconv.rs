#[derive(Clone)]
pub enum AskBidOption {
    AskOnly,
    AskFirst,
    BidOnly,
    BidFirst
}

#[derive(Debug)]
pub enum AskBid {
    Ask,
    Bid
}
