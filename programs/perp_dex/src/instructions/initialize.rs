use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::constants::{PROGRAM_SEED, VAULT_SEED};
use crate::state::state::State;

/// Accounts required to initialize the Perpetual DEX program.
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The program admin who will fund the initialization.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The main program state account (PDA).
    #[account(
        init,
        payer = admin,
        space = State::LEN,
        seeds = [PROGRAM_SEED],
        bump
    )]
    pub program_state: Account<'info, State>,

    /// The mint for the collateral asset (e.g., USDC).
    pub usdc_mint: Account<'info, Mint>,

    /// The collateral vault that holds all user funds (PDA).
    #[account(
        init,
        payer = admin,
        seeds = [VAULT_SEED, usdc_mint.key().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = program_state,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Handles the initialization of the global program state.
pub fn handle(ctx: Context<Initialize>, usdc_mint: Pubkey) -> Result<()> {
    let state = &mut ctx.accounts.program_state;

    state.admin = *ctx.accounts.admin.key;
    state.usdc_mint = usdc_mint;
    state.bump = ctx.bumps.program_state;
    state.number_of_markets = 0;
    state.paused = false;

    Ok(())
}
