use anchor_lang::prelude::*;
use pyth_sdk_solana::PriceFeed;
use crate::state::constants::{FUNDING_PERIOD, MARKET_SEED};
use crate::state::market::Market;
use crate::state::state::State;
use crate::error::PerpError;

/// Context for creating a new perpetual market.
#[derive(Accounts)]
#[instruction(market_index: u16)]
pub struct CreateMarket<'info> {
    /// Admin account who initializes the market.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Program state (must match the admin).
    #[account(
        mut,
        has_one = admin,
    )]
    pub program_state: Account<'info, State>,

    /// The market account (PDA) to initialize.
    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [MARKET_SEED, &market_index.to_le_bytes()],
        bump
    )]
    pub market: AccountLoader<'info, Market>,

    /// Oracle account (Pyth price feed).
    /// CHECK: This is verified inside the handler.
    pub oracle_price_feed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

/// Handles creation of a new market with associated AMM and margin parameters.
pub fn handle(
    ctx: Context<CreateMarket>,
    market_index: u16,
    amm_base_asset_reserve: u128,
    amm_quote_asset_reserve: u128,
    trade_fee_rate: u64,
    liquidation_fee_rate: u64,
    initial_margin_ratio: u64,
    maintenance_margin_ratio: u64,
) -> Result<()> {
    // Sanity checks
    require_gt!(amm_base_asset_reserve, 0, PerpError::InvalidCalculation);
    require_gt!(amm_quote_asset_reserve, 0, PerpError::InvalidCalculation);
    require_gt!(initial_margin_ratio, maintenance_margin_ratio, PerpError::InvalidCalculation);

    // Validate oracle
    let price_feed_info = &ctx.accounts.oracle_price_feed;
    let price_feed: PriceFeed = pyth_sdk_solana::load_price_feed_from_account_info(price_feed_info)
        .map_err(|_| error!(PerpError::InvalidOraclePrice))?;
    let _current_price = price_feed.get_price().ok_or(PerpError::InvalidOraclePrice)?;

    // Initialize market
    let mut market = ctx.accounts.market.load_init()?;

    market.market_index = market_index;
    market.initialized = true;
    market.paused = false;
    market.bump = ctx.bumps.market;

    market.amm_base_asset_reserve = amm_base_asset_reserve;
    market.amm_quote_asset_reserve = amm_quote_asset_reserve;
    market.amm_k_constant = amm_base_asset_reserve
        .checked_mul(amm_quote_asset_reserve)
        .ok_or(PerpError::MathOverflow)?;

    market.oracle_price_feed = *price_feed_info.key;

    market.trade_fee_rate = trade_fee_rate;
    market.liquidation_fee_rate = liquidation_fee_rate;
    market.initial_margin_ratio = initial_margin_ratio;
    market.maintenance_margin_ratio = maintenance_margin_ratio;

    market.last_funding_ts = Clock::get()?.unix_timestamp;
    market.funding_period = FUNDING_PERIOD;

    // Increment global market count
    ctx.accounts.program_state.number_of_markets = ctx
        .accounts
        .program_state
        .number_of_markets
        .checked_add(1)
        .ok_or(PerpError::MathOverflow)?;

    Ok(())
}
