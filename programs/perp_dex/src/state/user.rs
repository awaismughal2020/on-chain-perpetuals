use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::state::constants::{MAX_POSITIONS, PRECISION};
use crate::error::PerpError;

/// A user's position in a single market.
#[zero_copy]
#[repr(C)]
#[derive(Default, Pod, Zeroable)]
pub struct Position {
    /// Market index this position belongs to.
    pub market_index: u16,

    /// The amount of base asset. Positive for long, negative for short.
    pub base_asset_amount: i128,

    /// The quote asset amount used to acquire the position.
    pub quote_asset_amount: u128,

    /// Last cumulative funding rate settled.
    pub last_cumulative_funding_rate: i128,

    /// Last timestamp funding was settled.
    pub last_settled_funding_ts: i64,
}

impl Position {
    /// Calculates the unrealized PnL for the position.
    pub fn get_unrealized_pnl(&self, mark_price: u128) -> Result<i128> {
        if self.base_asset_amount == 0 {
            return Ok(0);
        }

        // Value of position at current mark price
        let current_value = self
            .base_asset_amount
            .abs()
            .checked_mul(mark_price)
            .ok_or(PerpError::MathOverflow)?;

        // Value of position at entry
        let entry_value = self
            .quote_asset_amount
            .checked_mul(PRECISION)
            .ok_or(PerpError::MathOverflow)?;

        let pnl = if self.base_asset_amount > 0 {
            // Long position
            current_value.checked_sub(entry_value)
        } else {
            // Short position
            entry_value.checked_sub(current_value)
        }
        .ok_or(PerpError::MathOverflow)?;

        Ok(pnl as i128)
    }
}

/// A user's account storing collateral and positions.
#[account(zero_copy)]
#[repr(C)]
pub struct User {
    /// The authority (owner) of this account.
    pub authority: Pubkey,

    /// The PDA bump.
    pub bump: u8,

    /// Is the user account initialized.
    pub initialized: bool,

    /// Prevents re-entrant CPIs during sensitive operations.
    pub operation_lock: bool,

    // Collateral
    /// Total collateral deposited (in collateral precision).
    pub collateral: u64,

    // Positions
    pub positions: [Position; MAX_POSITIONS],

    /// Padding for future upgrades.
    pub _padding: [u8; 256],
}

impl User {
    /// Finds a mutable reference to an existing position in a specific market.
    pub fn find_position_mut(&mut self, market_index: u16) -> Result<&mut Position> {
        self.positions
            .iter_mut()
            .find(|p| p.market_index == market_index && p.base_asset_amount != 0)
            .ok_or(PerpError::PositionNotFound.into())
    }

    /// Finds or creates a mutable reference to a position for the given market.
    pub fn find_or_create_position_mut(&mut self, market_index: u16) -> Result<&mut Position> {
        // Try to find an existing position
        if let Some(pos) = self.positions.iter_mut().find(|p| p.market_index == market_index) {
            return Ok(pos);
        }

        // Or find an empty slot to create a new one
        if let Some(pos) = self.positions.iter_mut().find(|p| p.base_asset_amount == 0) {
            pos.market_index = market_index;
            return Ok(pos);
        }

        Err(PerpError::InvalidMarketIndex.into())
    }
}
