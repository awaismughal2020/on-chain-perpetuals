use anchor_lang::prelude::*;

#[error_code]
pub enum PerpError {
    #[msg("Invalid calculation")]
    InvalidCalculation,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid Oracle Price")]
    InvalidOraclePrice,

    #[msg("Oracle price is stale")]
    StaleOraclePrice,

    #[msg("AMM price is outside the limit price")]
    PriceSlippage,

    #[msg("Invalid trade direction")]
    InvalidTradeDirection,

    #[msg("Market is paused")]
    MarketPaused,

    #[msg("Position not found for the given market")]
    PositionNotFound,

    #[msg("No position to close")]
    NoPositionToClose,

    #[msg("Position is not liquidatable")]
    PositionNotLiquidatable,

    #[msg("Insufficient collateral for withdrawal or trade")]
    InsufficientCollateral,

    #[msg("Cannot withdraw funds, it would cause a margin call")]
    WithdrawalCausesMarginCall,

    #[msg("Cannot open position, it would cause an immediate margin call")]
    PositionCausesMarginCall,

    #[msg("Invalid amount for deposit or withdrawal")]
    InvalidAmount,

    #[msg("The market index provided is invalid or out of bounds")]
    InvalidMarketIndex,

    #[msg("Market is in an unhealthy state")]
    UnhealthyMarketState,

    #[msg("Re-entrancy guard is active")]
    ReentrancyGuardActive,

    #[msg("Funding was already settled for the period")]
    FundingAlreadySettled,
}
