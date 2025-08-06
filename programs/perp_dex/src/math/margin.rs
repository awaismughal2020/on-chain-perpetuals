use anchor_lang::prelude::*;
use crate::state::user::{User, Position};
use crate::state::market::Market;
use crate::state::constants::{PRECISION, COLLATERAL_PRECISION};
use crate::validation::validate_oracle_price;
use crate::error::PerpError;

pub fn meets_initial_margin_requirement(
    user: &User,
    remaining_accounts: &[AccountInfo],
) -> Result<bool> {
    let market_info = remaining_accounts.get(0).ok_or(PerpError::InvalidMarketIndex)?;
    let oracle_info = remaining_accounts.get(1).ok_or(PerpError::InvalidOraclePrice)?;

    let market: AccountLoader<Market> = AccountLoader::try_from(market_info)?;
    let market = market.load()?;
    let oracle_price = validate_oracle_price(oracle_info, &Clock::get()?)?;

    let total_position_value = get_total_position_value(user, &market, oracle_price)?;
    let total_collateral_value =
        user.collateral as u128 * (PRECISION / COLLATERAL_PRECISION as u128);

    if total_position_value == 0 {
        return Ok(true);
    }

    let margin_ratio = total_collateral_value
        .checked_mul(PRECISION)
        .and_then(|n| n.checked_div(total_position_value))
        .ok_or(PerpError::MathOverflow)?;

    Ok(margin_ratio >= market.initial_margin_ratio as u128)
}

pub fn is_liquidatable(
    user: &User,
    market: &Market,
    remaining_accounts: &[AccountInfo],
) -> Result<bool> {
    let oracle_info = remaining_accounts.get(0).ok_or(PerpError::InvalidOraclePrice)?;
    let oracle_price = validate_oracle_price(oracle_info, &Clock::get()?)?;

    let total_position_value = get_total_position_value(user, market, oracle_price)?;
    let total_collateral_value =
        user.collateral as u128 * (PRECISION / COLLATERAL_PRECISION as u128);

    if total_position_value == 0 {
        return Ok(false);
    }

    let margin_ratio = total_collateral_value
        .checked_mul(PRECISION)
        .and_then(|n| n.checked_div(total_position_value))
        .ok_or(PerpError::MathOverflow)?;

    Ok(margin_ratio < market.maintenance_margin_ratio as u128)
}

fn get_total_position_value(user: &User, market: &Market, price: u128) -> Result<u128> {
    let mut total_value = 0u128;
    for position in user.positions.iter() {
        if position.market_index == market.market_index && position.base_asset_amount != 0 {
            let value = (position.base_asset_amount.abs() as u128)
                .checked_mul(price)
                .and_then(|n| n.checked_div(PRECISION))
                .ok_or(PerpError::MathOverflow)?;
            total_value = total_value
                .checked_add(value)
                .ok_or(PerpError::MathOverflow)?;
        }
    }
    Ok(total_value)
}

