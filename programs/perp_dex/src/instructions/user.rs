use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::state::constants::{USER_SEED, VAULT_SEED, COLLATERAL_PRECISION};
use crate::state::state::State;
use crate::state::user::User;
use crate::error::PerpError;
use crate::math::margin::meets_initial_margin_requirement;
use crate::validation::validate_user_not_locked;

/// Initializes a new user account.
#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<User>(),
        seeds = [USER_SEED, authority.key().as_ref()],
        bump
    )]
    pub user_account: AccountLoader<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_user(ctx: Context<CreateUser>) -> Result<()> {
    let mut user = ctx.accounts.user_account.load_init()?;
    user.authority = *ctx.accounts.authority.key;
    user.bump = ctx.bumps.user_account;
    user.initialized = true;
    Ok(())
}

/// User deposits collateral (e.g., USDC) into the protocol.
#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
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
        seeds = [VAULT_SEED, usdc_mint.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
    require_gt!(amount, 0, PerpError::InvalidAmount);
    let mut user = ctx.accounts.user_account.load_mut()?;
    validate_user_not_locked(&user)?;

    // Transfer collateral from user to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_collateral_account.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    user.collateral = user.collateral.checked_add(amount).ok_or(PerpError::MathOverflow)?;
    Ok(())
}

/// User withdraws collateral, checking margin safety.
#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority,
        seeds = [USER_SEED, authority.key().as_ref()],
        bump = user_account.load()?.bump
    )]
    pub user_account: AccountLoader<'info, User>,

    pub program_state: Account<'info, State>,

    #[account(
        mut,
        seeds = [VAULT_SEED, program_state.usdc_mint.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
    require_gt!(amount, 0, PerpError::InvalidAmount);
    let mut user = ctx.accounts.user_account.load_mut()?;
    validate_user_not_locked(&user)?;

    // Subtract collateral and check margin
    let free_collateral = user.collateral.checked_sub(amount).ok_or(PerpError::InsufficientCollateral)?;

    require!(
        meets_initial_margin_requirement(&user, &ctx.remaining_accounts)?,
        PerpError::WithdrawalCausesMarginCall
    );

    // Transfer funds using signer seeds
    let state_bump = ctx.accounts.program_state.bump;
    let signer_seeds = &[&crate::state::constants::PROGRAM_SEED[..], &[state_bump]];
    let signer = &[&signer_seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.user_collateral_account.to_account_info(),
        authority: ctx.accounts.program_state.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
    token::transfer(cpi_ctx, amount)?;

    // Save reduced balance
    let mut user = ctx.accounts.user_account.load_mut()?;
    user.collateral = free_collateral;

    Ok(())
}
