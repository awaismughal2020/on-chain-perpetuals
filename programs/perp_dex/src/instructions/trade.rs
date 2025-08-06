use anchor_lang::prelude::*;
use crate::state::market::Market;
use crate::state::user::User;
use crate::state::constants::{MARKET_SEED, USER_SEED, PRECISION};
use crate::error::PerpError;
use crate::math::amm;
use crate::math::margin::meets_initial_margin_requirement;
use crate::validation::{validate_user_not_locked, validate_market_not_paused};

#[derive(Accounts)]
pub struct OpenPosition<'info> {
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
        seeds = [MARKET_SEED, &market.load()?.market_index.to_le_bytes()],
        bump = market.load()?.bump
    )]
    pub market: AccountLoader<'info, Market>,
}

pub fn handle_open_position(
    ctx: Context<OpenPosition>,
    base_asset_amount: i128,
    limit_price: u128,
) -> Result<()> {
    let mut user = ctx.accounts.user_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    validate_user_not_locked(&user)?;
    validate_market_not_paused(&market)?;

    let direction = if base_asset_amount > 0 {
        amm::TradeDirection::Long
    } else {
        amm::TradeDirection::Short
    };

    let (new_quote_asset_reserve, new_base_asset_reserve) = amm::calculate_swap_output(
        base_asset_amount.abs(),
        market.amm_base_asset_reserve,
        market.amm_quote_asset_reserve,
        direction,
    )?;

    let quote_asset_amount_acquired =
        market.amm_quote_asset_reserve.abs_diff(new_quote_asset_reserve);

    let entry_price = quote_asset_amount_acquired
        .checked_mul(PRECISION)
        .and_then(|n| n.checked_div(base_asset_amount.abs() as u128))
        .ok_or(PerpError::MathOverflow)?;

    match direction {
        amm::TradeDirection::Long => {
            require_gte!(limit_price, entry_price, PerpError::PriceSlippage)
        }
        amm::TradeDirection::Short => {
            require_lte!(limit_price, entry_price, PerpError::PriceSlippage)
        }
    }

    market.amm_base_asset_reserve = new_base_asset_reserve;
    market.amm_quote_asset_reserve = new_quote_asset_reserve;

    let position = user.find_or_create_position_mut(market.market_index)?;
    position.base_asset_amount = position
        .base_asset_amount
        .checked_add(base_asset_amount)
        .ok_or(PerpError::MathOverflow)?;
    position.quote_asset_amount = position
        .quote_asset_amount
        .checked_add(quote_asset_amount_acquired)
        .ok_or(PerpError::MathOverflow)?;

    require!(
        meets_initial_margin_requirement(&user, &ctx.remaining_accounts)?,
        PerpError::PositionCausesMarginCall
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(market_index: u16)]
pub struct ClosePosition<'info> {
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
}

pub fn handle_close_position(
    ctx: Context<ClosePosition>,
    market_index: u16,
) -> Result<()> {
    let mut user = ctx.accounts.user_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    validate_user_not_locked(&user)?;
    validate_market_not_paused(&market)?;

    let position_to_close = user.find_position_mut(market_index)?.clone();

    if position_to_close.base_asset_amount == 0 {
        return err!(PerpError::NoPositionToClose);
    }

    let base_asset_amount_to_close = -position_to_close.base_asset_amount;
    let direction = if base_asset_amount_to_close > 0 {
        amm::TradeDirection::Long
    } else {
        amm::TradeDirection::Short
    };

    let (new_quote_asset_reserve, new_base_asset_reserve) = amm::calculate_swap_output(
        base_asset_amount_to_close.abs(),
        market.amm_base_asset_reserve,
        market.amm_quote_asset_reserve,
        direction,
    )?;

    let quote_asset_returned = market.amm_quote_asset_reserve.abs_diff(new_quote_asset_reserve);

    let pnl = if position_to_close.base_asset_amount > 0 {
        quote_asset_returned as i128 - position_to_close.quote_asset_amount as i128
    } else {
        position_to_close.quote_asset_amount as i128 - quote_asset_returned as i128
    };

    if pnl > 0 {
        user.collateral = user
            .collateral
            .checked_add((pnl / (PRECISION as i128)) as u64)
            .ok_or(PerpError::MathOverflow)?;
    } else {
        user.collateral = user
            .collateral
            .checked_sub((pnl.abs() / (PRECISION as i128)) as u64)
            .ok_or(PerpError::MathOverflow)?;
    }

    market.amm_base_asset_reserve = new_base_asset_reserve;
    market.amm_quote_asset_reserve = new_quote_asset_reserve;

    let position = user.find_position_mut(market_index)?;
    *position = Default::default();
    position.market_index = market_index;

    Ok(())
}
