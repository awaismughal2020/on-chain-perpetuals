use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
use crate::state::constants::{ORACLE_STALENESS_THRESHOLD, PRECISION};
use crate::state::market::Market;
use crate::state::user::User;
use crate::error::PerpError;

pub fn validate_oracle_price(oracle_info: &AccountInfo, clock: &Clock) -> Result<u128> {
    let price_feed = load_price_feed_from_account_info(oracle_info)
        .map_err(|_| error!(PerpError::InvalidOraclePrice))?;

    let price = price_feed
        .get_price_no_older_than(clock.unix_timestamp, ORACLE_STALENESS_THRESHOLD)
        .ok_or(PerpError::StaleOraclePrice)?;

    if price.expo > 0 {
        return Err(PerpError::InvalidOraclePrice.into());
    }

    let scale_factor = 10u128.pow(price.expo.abs() as u32);
    let oracle_price_u128 = (price.price as u128)
        .checked_mul(PRECISION)
        .and_then(|p| p.checked_div(scale_factor))
        .ok_or(PerpError::MathOverflow)?;

    Ok(oracle_price_u128)
}

pub fn validate_user_not_locked(user: &User) -> Result<()> {
    require!(!user.operation_lock, PerpError::ReentrancyGuardActive);
    Ok(())
}

pub fn validate_market_not_paused(market: &Market) -> Result<()> {
    require!(!market.paused, PerpError::MarketPaused);
    Ok(())
}
