use anchor_lang::prelude::*;
use std::ops::{ Div, Mul };

use crate::consts::*;
use crate::errors::CustomError;

#[account]
pub struct InitializeConfiguration {
    pub swap_fee: u64,
    pub bonding_curve_limitation: u64,
    pub initial_virtual_base_token: u64,
    pub initial_virtual_quote_token: u64,
    pub create_pool_fee_amount: u64,
    pub base_token_ca: Pubkey,
    pub fee_wallet: Pubkey,
    pub migration_authority: Pubkey,
    pub admin: Pubkey,
    pub raydium_migration_fee: u64,
}

impl InitializeConfiguration {
    pub const SIZE: usize = 8 * 8 + 32 * 4;
}

#[account]
#[derive(Debug)]
pub struct BondingCurve {
    pub init_virtual_base_token: u64,
    pub init_virtual_quote_token: u64,
    pub quote_token_reserves: u64,
    pub base_token_reserves: u64,
    pub k_value: u128, // k = x * y
    pub is_completed: bool,
}

impl BondingCurve {
    pub const SIZE: usize = 49;

    pub fn get(&self) -> &BondingCurve {
        self
    }
}

#[derive(Accounts)]
pub struct SetGlobalConfiguration<'info> {
    #[account(constraint = admin.key() == global_configuration.admin.key() @ CustomError::InvalidAdminAccount)]
    pub admin: Signer<'info>, // ADMIN_C must be this signer

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()], 
        bump
    )]
    pub global_configuration: Account<'info, InitializeConfiguration>,
}
