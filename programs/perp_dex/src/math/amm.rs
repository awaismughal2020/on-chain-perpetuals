use anchor_lang::prelude::*;
use crate::error::PerpError;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TradeDirection {
    Long,
    Short,
}

/// Calculates the output of a swap against the vAMM.
/// Returns (new_quote_asset_reserve, new_base_asset_reserve).
pub fn calculate_swap_output(
    base_asset_amount: u128,
    base_asset_reserve: u128,
    quote_asset_reserve: u128,
    direction: TradeDirection,
) -> Result<(u128, u128)> {
    if base_asset_reserve == 0 || quote_asset_reserve == 0 {
        return Err(PerpError::UnhealthyMarketState.into());
    }

    let k = base_asset_reserve
        .checked_mul(quote_asset_reserve)
        .ok_or(PerpError::MathOverflow)?;

    let new_base_asset_reserve = match direction {
        TradeDirection::Long => base_asset_reserve
            .checked_sub(base_asset_amount)
            .ok_or(PerpError::MathOverflow)?,
        TradeDirection::Short => base_asset_reserve
            .checked_add(base_asset_amount)
            .ok_or(PerpError::MathOverflow)?,
    };

    let new_quote_asset_reserve = k
        .checked_div(new_base_asset_reserve)
        .ok_or(PerpError::MathOverflow)?;

    Ok((new_quote_asset_reserve, new_base_asset_reserve))
}
