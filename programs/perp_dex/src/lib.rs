//! On-chain perpetuals DEX with a virtual AMM (vAMM).

use anchor_lang::prelude::*;

// Module declarations for organized code structure
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;
pub mod validation;

// Make modules public for use in the program
use instructions::*;
use state::constants::PROGRAM_SEED;

declare_id!("perpFC8a13h45b2n3sUKG5aD5EwB2gXcnm5FL12h4m");

#[program]
pub mod perp_dex {
    use super::*;

    /// Initializes the global state of the protocol.
    /// Must be called once before any other instruction.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `usdc_mint` - The public key of the USDC mint to be used as collateral.
    pub fn initialize(ctx: Context<Initialize>, usdc_mint: Pubkey) -> Result<()> {
        instructions::initialize::handle(ctx, usdc_mint)
    }

    /// Creates a new perpetuals market.
    /// Only callable by the program admin.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `market_index` - A unique index for the new market.
    /// * `amm_base_asset_reserve` - Initial virtual base asset reserves for the vAMM.
    /// * `amm_quote_asset_reserve` - Initial virtual quote asset reserves for the vAMM.
    /// * `trade_fee_rate` - Fee rate for trades (e.g., 1000 for 0.1%).
    /// * `liquidation_fee_rate` - Fee for liquidators.
    /// * `initial_margin_ratio` - Initial margin ratio requirement.
    /// * `maintenance_margin_ratio` - Maintenance margin ratio requirement.
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_index: u16,
        amm_base_asset_reserve: u128,
        amm_quote_asset_reserve: u128,
        trade_fee_rate: u64,
        liquidation_fee_rate: u64,
        initial_margin_ratio: u64,
        maintenance_margin_ratio: u64,
    ) -> Result<()> {
        instructions::create_market::handle(
            ctx,
            market_index,
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            trade_fee_rate,
            liquidation_fee_rate,
            initial_margin_ratio,
            maintenance_margin_ratio,
        )
    }

    /// Creates a user account PDA to store their positions and collateral.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    pub fn create_user(ctx: Context<CreateUser>) -> Result<()> {
        instructions::user::handle_create_user(ctx)
    }

    /// Deposits collateral into the user's account.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `amount` - The amount of collateral to deposit.
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        instructions::user::handle_deposit_collateral(ctx, amount)
    }

    /// Withdraws collateral from the user's account.
    /// Fails if the withdrawal would push the user below the initial margin requirement.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `amount` - The amount of collateral to withdraw.
    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        instructions::user::handle_withdraw_collateral(ctx, amount)
    }

    /// Opens a new long or short position or modifies an existing one.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `base_asset_amount` - The amount of the base asset to trade. Positive for long, negative for short.
    /// * `limit_price` - The price limit for the trade. The trade will only execute if the resulting price is better.
    pub fn open_position(
        ctx: Context<OpenPosition>,
        base_asset_amount: i128,
        limit_price: u128,
    ) -> Result<()> {
        instructions::trade::handle_open_position(ctx, base_asset_amount, limit_price)
    }

    /// Closes an existing position.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `market_index` - The index of the market to close the position in.
    pub fn close_position(ctx: Context<ClosePosition>, market_index: u16) -> Result<()> {
        instructions::trade::handle_close_position(ctx, market_index)
    }

    /// Liquidates a user's position if their margin ratio is below the maintenance requirement.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `market_index` - The index of the market to liquidate the position in.
    pub fn liquidate(ctx: Context<Liquidate>, market_index: u16) -> Result<()> {
        instructions::liquidation::handle_liquidate(ctx, market_index)
    }

    /// Settles the funding rate payment for a user's position.
    ///
    /// # Arguments
    /// * `ctx` - The context for the instruction.
    /// * `market_index` - The index of the market to settle funding for.
    pub fn settle_funding(ctx: Context<SettleFunding>, market_index: u16) -> Result<()> {
        instructions::funding::handle_settle_funding(ctx, market_index)
    }
}
