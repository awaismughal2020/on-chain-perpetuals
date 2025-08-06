use anchor_lang::prelude::*;
use crate::state::constants::{MARKET_SEED, USER_SEED, PRECISION};
use crate::state::market::Market;
use crate::state::user::User;
use crate::error::PerpError;
use crate::validation::{validate_user_not_locked, validate_oracle_price};

#[derive(Accounts)]
#[instruction(market_index: u16)]
pub struct SettleFunding<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority,
        seeds = [USER_SEED, authority.key().as_ref()],
        bump = user_account.load()?.bump
    )]
    pub user_account: AccountLoader<'info, User>,

    #[account(
        mut,
        seeds = [MARKET_SEED, &market_index.to_le_bytes()],
        bump = market.load()?.bump
    )]
    pub market: AccountLoader<'info, Market>,

    /// CHECK: Oracle account, validated in handler
    pub oracle_price_feed: AccountInfo<'info>,
}

pub fn handle_settle_funding(ctx: Context<SettleFunding>, market_index: u16) -> Result<()> {
    let mut user = ctx.accounts.user_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;
    validate_user_not_locked(&user)?;

    let position = user.find_position_mut(market_index)?;
    require_keys_eq!(market.oracle_price_feed, ctx.accounts.oracle_price_feed.key());

    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    let time_since_last_settle = now
        .checked_sub(position.last_settled_funding_ts)
        .ok_or(PerpError::MathOverflow)?;

    if time_since_last_settle < market.funding_period {
        return Ok(());
    }

    let time_since_last_update = now
        .checked_sub(market.last_funding_ts)
        .ok_or(PerpError::MathOverflow)?;

    if time_since_last_update >= market.funding_period {
        let oracle_price = validate_oracle_price(&ctx.accounts.oracle_price_feed, &clock)?;
        let mark_price = market.get_mark_price()?;

        let premium = (mark_price as i128)
            .checked_sub(oracle_price as i128)
            .ok_or(PerpError::MathOverflow)?;

        let funding_rate = premium
            .checked_mul(PRECISION as i128)
            .and_then(|p| p.checked_div(24))
            .ok_or(PerpError::MathOverflow)?;

        market.last_funding_rate = funding_rate;
        market.last_funding_ts = now;
    }

    let funding_payment = position.base_asset_amount
        .checked_mul(market.last_funding_rate)
        .and_then(|p| p.checked_div(PRECISION as i128))
        .ok_or(PerpError::MathOverflow)?;

    let collateral_change = -funding_payment;

    if collateral_change > 0 {
        user.collateral = user
            .collateral
            .checked_add(collateral_change as u64)
            .ok_or(PerpError::MathOverflow)?;
    } else {
        user.collateral = user
            .collateral
            .checked_sub(collateral_change.abs() as u64)
            .ok_or(PerpError::MathOverflow)?;
    }

    position.last_settled_funding_ts = now;

    Ok(())
}
