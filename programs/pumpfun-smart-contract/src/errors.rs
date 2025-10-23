use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Invalid Admin Address!")]
    InvalidAdminAccount,

    #[msg("Invalid Migration Authority!")]
    InvalidMigrationAuth,

    #[msg("Slippage Mismatch Error!")]
    SlippageExceeded,

    #[msg("Invalid Base Token Address!")]
    InvalidBaseToken,

    #[msg("Invaild Fee Wallet!")]
    InvalidFeeWallet,

    #[msg("Bonding Curve Is Completed!")]
    BondingCurveIsCompleted,

    #[msg("Bonding Curve Is Processing!")]
    BondingCurveIsNotCompleted,

    #[msg("Not enough base token!")]
    NotEnoughBaseToken,

    #[msg("Not enough quote token!")]
    NotEnoughQuoteToken,

    #[msg("DevBuy Amount is too small!")]
    DevBuyAmountIsTooSmall,

    #[msg("Not enough sol balance to create accounts!")]
    NotEnoughSolBalance,

    #[msg("Invalid initial token transfer percent!")]
    InvalidInitialTokenTransferPercent,

    #[msg("Overflow Estimate Out Quote!")]
    OverflowEstimateOutQuote,

    #[msg("Overflow Estimate Out Base!")]
    OverflowEstimateOutBase,

    #[msg("Math Overflow!")]
    MathOverflow,

    #[msg("Math Underflow!")]
    MathUnderflow,

    #[msg("Math Division By Zero!")]
    MathDivisionByZero,

    #[msg("Token mint constraint error!")]
    TokenConstraintError,

    #[msg("Invalid Open Time!")]
    InvalidOpenTime,
}
