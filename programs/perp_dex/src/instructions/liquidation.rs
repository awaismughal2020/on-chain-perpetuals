use anchor_lang::prelude::*;
use crate::state::constants::{MARKET_SEED, USER_SEED};
use crate::state::market::Market;
use crate::state::user::User;
use crate::error::PerpError;
use crate::math::margin::is_liquidatable;
use crate::validation::{validate_user_not_locked, validate_market_not_paused};

#[derive(Accounts)]
#[instruction(market_index: u16)]
pub struct Liquidate<'info> {
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_SEED, user_account.load()?.authority.as_ref()],
        bump = user_account.load()?.bump
    )]
    pub user_account: AccountLoader<'info, User>,

    #[account(
        mut,
        seeds = [MARKET_SEED, &market_index.to_le_bytes()],
        bump = market.load()?.bump
    )]
    pub market: AccountLoader<'info, Market>,
}

pub fn handle_liquidate(ctx: Context<Liquidate>, market_index: u16) -> Result<()> {
    let mut user = ctx.accounts.user_account.load_mut()?;
    let market = ctx.accounts.market.load()?;

    validate_user_not_locked(&user)?;
    validate_market_not_paused(&market)?;

    user.operation_lock = true;

    require!(
        is_liquidatable(&user, &market, &ctx.remaining_accounts)?,
        PerpError::PositionNotLiquidatable
    );

    let position_to_liquidate = user.find_position_mut(market_index)?.clone();

    let position_value = position_to_liquidate
        .base_asset_amount
        .abs()
        .checked_mul(market.get_mark_price()?)
        .ok_or(PerpError::MathOverflow)?;

    let liquidation_fee = (position_value as u64)
        .checked_mul(market.liquidation_fee_rate)
        .and_then(|f| f.checked_div(1_000_000))
        .ok_or(PerpError::MathOverflow)?;

    user.collateral = user
        .collateral
        .checked_sub(liquidation_fee)
        .ok_or(PerpError::InsufficientCollateral)?;

    let position = user.find_position_mut(market_index)?;
    *position = Default::default();
    position.market_index = market_index;

    user.operation_lock = false;

    Ok(())
}
