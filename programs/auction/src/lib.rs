use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use error::AuctionError;
use state::{Auction, BidInfo};
use validation::{validate_auction_active, validate_auction_inactive};

pub mod error;
pub mod state;
pub mod validation;

declare_id!("4oGFTts1oTHgUQZt1DMcQN7BRiJiR2KunLdTuDNZyv1L");

#[program]
pub mod auction {
    use super::*;

    /// Creates and initialize a new state of our program
    pub fn initialize(ctx: Context<AuctionInitialize>, auction_duration: i64) -> Result<()> {
        validate_auction_active(auction_duration)?;
        let state = &mut ctx.accounts.state;
        state.end_at = auction_duration;
        state.highest_bid = None;
        state.bidder = None;
        state.ended = false;
        state.seller = ctx.accounts.seller.key();
        state.treasury = ctx.accounts.treasury.key();
        Ok(())
    }
    /// Bid
    pub fn bid(ctx: Context<AuctionBid>, amount: u64) -> Result<()> {
        validate_auction_active(ctx.accounts.state.end_at)?;
        transfer(ctx.accounts.into_treasury_transfer_context(), amount)?;
        let state = &mut ctx.accounts.state;
        let bid = &mut ctx.accounts.bid_info;
        bid.amount = amount;
        bid.bump = *ctx.bumps.get("bid_info").unwrap();
        if state.highest_bid.is_none()
            || (state.highest_bid.is_some() && state.highest_bid.unwrap() < amount)
        {
            state.highest_bid = Some(amount);
            state.bidder = Some(ctx.accounts.bidder.key());
        }
        Ok(())
    }
    /// After an auction ends (determined by `auction_duration`), a seller can claim the
    /// highest bid by calling this instruction
    pub fn end_auction(ctx: Context<AuctionEnd>) -> Result<()> {
        validate_auction_inactive(ctx.accounts.state.end_at)?;
        if ctx.accounts.state.ended {
            return Err(error!(AuctionError::AuctionEnded));
        }
        transfer(
            ctx.accounts.into_seller_transfer_context(),
            ctx.accounts.state.highest_bid.unwrap(),
        )?;
        let state = &mut ctx.accounts.state;
        state.ended = true;
        Ok(())
    }
    /// After an auction ends (the initializer/seller already received the winning bid),
    /// the unsuccessful bidders can claim their money back by calling this instruction
    pub fn refund(ctx: Context<AuctionRefund>) -> Result<()> {
        let state = &ctx.accounts.state;
        validate_auction_inactive(state.end_at)?;
        if !state.ended {
            return Err(error!(AuctionError::AuctionNotEnded));
        }
        transfer(
            ctx.accounts.into_bidder_transfer_context(),
            ctx.accounts.bid_info.amount,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AuctionInitialize<'info> {
    /// State of auction program
    #[account(
        init,
        payer = seller,
        // discriminator + (seller + treasury + end_at + ended + highest_bid + bidder)
        space = 8 + (32 + 32 + 8 + 1 + std::mem::size_of::<Option<u64>>() + std::mem::size_of::<Option<Pubkey>>())
    )]
    pub state: Account<'info, Auction>,
    /// Account which holds tokens bidded by bidders
    /// CHECK:
    pub treasury: AccountInfo<'info>,
    /// Seller
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuctionBid<'info> {
    #[account(mut)]
    pub state: Account<'info, Auction>,
    /// CHECK:
    #[account(mut, constraint = *treasury.key == state.treasury)]
    pub treasury: AccountInfo<'info>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(
        init,
        payer = bidder,
        // discriminator + (bump + amount)
        space = 8 + (1 + 8),
        seeds = [b"bid-info", bidder.key().as_ref()],
        bump
    )]
    pub bid_info: Account<'info, BidInfo>,
    pub system_program: Program<'info, System>,
}

impl<'info> AuctionBid<'info> {
    pub fn into_treasury_transfer_context(
        &self,
    ) -> CpiContext<'info, 'info, 'info, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.bidder.to_account_info(),
            to: self.treasury.clone(),
        };
        CpiContext::new(self.system_program.to_account_info(), accounts)
    }
}

#[derive(Accounts)]
pub struct AuctionEnd<'info> {
    #[account(mut)]
    pub state: Account<'info, Auction>,
    #[account(mut, constraint = treasury.to_account_info().key() == state.treasury)]
    pub treasury: Signer<'info>,
    #[account(mut, constraint = seller.to_account_info().key() == state.seller)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> AuctionEnd<'info> {
    pub fn into_seller_transfer_context(
        &self,
    ) -> CpiContext<'info, 'info, 'info, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.seller.to_account_info(),
        };
        CpiContext::new(self.system_program.to_account_info(), accounts)
    }
}

#[derive(Accounts)]
pub struct AuctionRefund<'info> {
    #[account()]
    pub state: Account<'info, Auction>,
    #[account(mut, constraint = treasury.to_account_info().key() == state.treasury)]
    pub treasury: Signer<'info>,
    #[account(mut, constraint = bidder.to_account_info().key() != state.bidder.unwrap())]
    pub bidder: Signer<'info>,
    #[account(mut, seeds = [b"bid-info", bidder.key().as_ref()], bump = bid_info.bump)]
    pub bid_info: Account<'info, BidInfo>,
    pub system_program: Program<'info, System>,
}

impl<'info> AuctionRefund<'info> {
    pub fn into_bidder_transfer_context(
        &self,
    ) -> CpiContext<'info, 'info, 'info, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.bidder.to_account_info(),
        };
        CpiContext::new(self.system_program.to_account_info(), accounts)
    }
}
