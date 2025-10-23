use anchor_lang::prelude::*;

#[event]
pub struct BondingCurveCompleted {
    pub mint_address: Pubkey,
    pub user_quote_token_ata: Pubkey,
    pub pool: Pubkey,
    pub quote_token_pool: Pubkey,
}

#[event]
pub struct TransactionEvent {
    pub operation: String,
    pub creator: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub platform_fee: u64,
    pub base_token_mint: Pubkey,
    pub mint_address: Pubkey,
    pub fee_wallet: Pubkey,
}

#[event]
pub struct MigrationEvent {
    pub mint_address: Pubkey,
    pub raydium_pool: Pubkey,
    pub contract: Pubkey,
}
