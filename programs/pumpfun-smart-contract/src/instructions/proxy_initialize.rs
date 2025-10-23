use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ Token, Mint },
    token_interface::{ Mint as InterfaceMint, TokenAccount as TokenInterAccount, TokenInterface },
};
use raydium_cp_swap::{ cpi, program::RaydiumCpSwap, states::{ AmmConfig } };
use anchor_lang::solana_program::system_instruction::transfer;
use crate::errors::CustomError;
use crate::consts::*;
use crate::state::{ InitializeConfiguration, BondingCurve };
use crate::events::MigrationEvent;

#[derive(Accounts)]
pub struct ProxyInitialize<'info> {
    #[account(mut, seeds = [CONFIG_SEED.as_bytes()], bump)]
    pub global_configuration: Account<'info, InitializeConfiguration>,

    #[account(
      mut,
      seeds = [ &mint_address.key().to_bytes(), CURVE_SEED.as_bytes()],
      bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    pub mint_address: Account<'info, Mint>,

    ///CHECK:
    #[account(
      mut,
      seeds = [&mint_address.key().to_bytes(), POOL_SEED.as_bytes()],
      bump
    )]
    pub pool: AccountInfo<'info>,

    /// CHECK:
    #[account(
        constraint = migration_authority.key() == global_configuration.migration_authority.key() @ CustomError::InvalidMigrationAuth
    )]
    pub migration_authority: Signer<'info>,

    pub cp_swap_program: Program<'info, RaydiumCpSwap>,

    /// CHECK: Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub creator: AccountInfo<'info>,

    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: pool vault and lp mint authority
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(mut)]
    pub pool_state: UncheckedAccount<'info>,

    /// Token_0 mint, the key must smaller then token_1 mint.
    #[account(constraint = token_0_mint.key() < token_1_mint.key() @ CustomError::TokenConstraintError)]
    pub token_0_mint: Account<'info, Mint>,

    /// Token_1 mint, the key must grater then token_0 mint.
    #[account(mut)]
    pub token_1_mint: Account<'info, Mint>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// payer token0 account
    #[account(mut)]
    pub creator_token_0: Box<InterfaceAccount<'info, TokenInterAccount>>,

    /// creator token1 account
    #[account(mut)]
    pub creator_token_1: Box<InterfaceAccount<'info, TokenInterAccount>>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,

    /// CHECK: Token_0 vault for the pool, init by cp-swap
    #[account(mut)]
    pub token_0_vault: UncheckedAccount<'info>,

    /// CHECK: Token_1 vault for the pool, init by cp-swap
    #[account(mut)]
    pub token_1_vault: UncheckedAccount<'info>,

    /// create pool fee account
    #[account(mut)]
    pub create_pool_fee: Box<InterfaceAccount<'info, TokenInterAccount>>,

    /// CHECK: an account to store oracle observations, init by cp-swap
    #[account(mut)]
    pub observation_state: UncheckedAccount<'info>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,

    /// Spl token program or token program
    pub token_0_program: Interface<'info, TokenInterface>,

    /// Spl token program or token program
    pub token_1_program: Interface<'info, TokenInterface>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,

    /// To create a new program account
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ProxyInitialize>) -> Result<()> {
    require!(
        ctx.accounts.bonding_curve.is_completed == true,
        CustomError::BondingCurveIsNotCompleted
    );

    let fee: u64 = ctx.accounts.global_configuration.raydium_migration_fee;

    let lamports_required = fee + tx_confirm_fee;

    require!(
        ctx.accounts.migration_authority.lamports() > lamports_required,
        CustomError::NotEnoughSolBalance
    );

    let transfer_ix = transfer(
        &ctx.accounts.migration_authority.key(),
        &ctx.accounts.creator.key(),
        lamports_required
    );

    anchor_lang::solana_program::program::invoke(
        &transfer_ix,
        &[
            ctx.accounts.migration_authority.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ]
    )?;

    let (init_amount_0, init_amount_1) = if
        ctx.accounts.token_0_mint.key() == ctx.accounts.global_configuration.base_token_ca
    {
        (
            ctx.accounts.bonding_curve.base_token_reserves,
            ctx.accounts.bonding_curve.quote_token_reserves,
        )
    } else {
        (
            ctx.accounts.bonding_curve.quote_token_reserves,
            ctx.accounts.bonding_curve.base_token_reserves,
        )
    };

    // CPI to Raydium CP-Swap program to initialize the pool

    emit!(MigrationEvent {
        mint_address: ctx.accounts.mint_address.key(),
        raydium_pool: ctx.accounts.pool_state.key(),
        contract: ctx.accounts.cp_swap_program.key(),
    });

    Ok(())
}
