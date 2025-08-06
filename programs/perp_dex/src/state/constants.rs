//! Constants used throughout the program.

use anchor_lang::prelude::*;

/// Seed for the main program state PDA.
pub const PROGRAM_SEED: &[u8] = b"perp_dex_state";

/// Seed for the collateral vault PDA.
pub const VAULT_SEED: &[u8] = b"collateral_vault";

/// Seed for the market PDA.
pub const MARKET_SEED: &[u8] = b"market";

/// Seed for the user account PDA.
pub const USER_SEED: &[u8] = b"user";

/// Precision for prices and assets (10^9).
pub const PRECISION: u128 = 1_000_000_000;

/// Precision for collateral (USDC, 10^6).
pub const COLLATERAL_PRECISION: u64 = 1_000_000;

/// Maximum number of positions a user can hold.
pub const MAX_POSITIONS: usize = 8;

/// Oracle price validity duration in seconds (e.g., 60 seconds).
pub const ORACLE_STALENESS_THRESHOLD: i64 = 60;

/// Funding rate period in seconds (e.g., 1 hour).
pub const FUNDING_PERIOD: i64 = 3600;
