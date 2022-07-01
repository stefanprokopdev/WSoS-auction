use anchor_lang::prelude::*;

#[account]
pub struct Auction {
    pub seller: Pubkey,
    pub treasury: Pubkey,
    pub end_at: i64,
    pub highest_bid: Option<u64>,
    pub bidder: Option<Pubkey>,
    pub ended: bool,
}

#[account]
pub struct BidInfo {
    pub bump: u8,
    pub amount: u64,
}
