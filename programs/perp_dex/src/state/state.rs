use anchor_lang::prelude::*;

/// Global state for the perpetuals DEX.
#[account]
#[derive(Default)]
pub struct State {
    /// The administrator of the program.
    pub admin: Pubkey,

    /// The mint for the collateral asset (e.g., USDC).
    pub usdc_mint: Pubkey,

    /// PDA bump for the state account.
    pub bump: u8,

    /// Number of markets created.
    pub number_of_markets: u16,

    /// Is the program paused (e.g., for upgrades).
    pub paused: bool,
}

impl State {
    /// Total size of the account, including padding.
    pub const LEN: usize = 8    // discriminator
        + 32                    // admin
        + 32                    // usdc_mint
        + 1                     // bump
        + 2                     // number_of_markets
        + 1                     // paused
        + 200;                  // padding for future upgrades
}
