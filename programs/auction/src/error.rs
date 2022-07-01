use anchor_lang::prelude::*;

#[error_code]
pub enum AuctionError {
    #[msg("Auction is active!")]
    AuctionActive,
    #[msg("Auction is inactive!")]
    AuctionInactive,
    #[msg("Bidder already claimed his/her money!")]
    BidderAlreadyClaimed,
    #[msg("Auction already ended!")]
    AuctionEnded,
    #[msg("Auction has not ended yet!")]
    AuctionNotEnded,
}
