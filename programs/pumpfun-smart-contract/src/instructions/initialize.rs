use anchor_lang::prelude::*;
use crate::state::{ InitializeConfiguration };
use crate::consts::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [CONFIG_SEED.as_bytes()],
        payer = admin,
        space = 8 + InitializeConfiguration::SIZE,
        bump
    )]
    pub global_configuration: Account<'info, InitializeConfiguration>,

    ///CHECK:
    pub fee_account: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    swap_fee: u64,
    bonding_curve_limitation: u64,
    initial_virtual_base_token: u64,
    initial_virtual_quote_token: u64,
    create_pool_fee_amount: u64,
    base_token_ca: Pubkey,
    fee_wallet: Pubkey,
    raydium_migration_fee: u64
) -> Result<()> {
    let config = &mut ctx.accounts.global_configuration;

    config.swap_fee = swap_fee;
    config.bonding_curve_limitation = bonding_curve_limitation;
    config.initial_virtual_base_token = initial_virtual_base_token;
    config.initial_virtual_quote_token = initial_virtual_quote_token;
    config.create_pool_fee_amount = create_pool_fee_amount;
    config.base_token_ca = base_token_ca;
    config.fee_wallet = fee_wallet;
    config.migration_authority = ctx.accounts.admin.key();
    config.admin = ctx.accounts.admin.key();
    config.raydium_migration_fee = raydium_migration_fee;

    Ok(())
}
