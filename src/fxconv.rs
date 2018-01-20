
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

pub enum Action {
    End,
    Continue
}
