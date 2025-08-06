use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::state::constants::PRECISION;

/// Represents a single perpetuals market.
#[account(zero_copy)]
#[repr(C)]
#[derive(Default, Pod, Zeroable)]
pub struct Market {
    /// Index of the market.
    pub market_index: u16,

    /// Is the market initialized.
    pub initialized: bool,

    /// Is the market paused for trading.
    pub paused: bool,

    /// PDA bump.
    pub bump: u8,

    // vAMM state
    pub amm_base_asset_reserve: u128,
    pub amm_quote_asset_reserve: u128,
    pub amm_k_constant: u128,

    // Oracle
    pub oracle_price_feed: Pubkey,

    // Fees
    /// Fee rate on trades (scaled by 1_000_000).
    pub trade_fee_rate: u64,

    /// Fee for liquidators (scaled by 1_000_000).
    pub liquidation_fee_rate: u64,

    // Margin requirements
    /// Initial margin ratio (scaled by 1_000_000).
    pub initial_margin_ratio: u64,

    /// Maintenance margin ratio (scaled by 1_000_000).
    pub maintenance_margin_ratio: u64,

    // Funding rate
    pub last_funding_rate: i128,
    pub last_funding_ts: i64,
    pub funding_period: i64,

    // Open Interest
    /// Total open interest in base asset terms.
    pub open_interest_base: u128,

    /// Padding for future upgrades.
    pub _padding: [u8; 256],
}

impl Market {
    /// Computes the mark price of the market using the vAMM reserves.
    pub fn get_mark_price(&self) -> Result<u128> {
        if self.amm_base_asset_reserve == 0 {
            return Ok(0);
        }

        self.amm_quote_asset_reserve
            .checked_mul(PRECISION)
            .and_then(|n| n.checked_div(self.amm_base_asset_reserve))
            .ok_or(ErrorCode::InvalidCalculation.into())
    }
}

