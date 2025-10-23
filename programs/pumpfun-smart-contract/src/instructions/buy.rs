use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ self, Token, Mint, TokenAccount, TransferChecked, transfer_checked },
};
use crate::{ consts::*, events::{ BondingCurveCompleted, TransactionEvent } };
use crate::state::{ BondingCurve, InitializeConfiguration };
use crate::errors::CustomError;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut, seeds = [CONFIG_SEED.as_bytes()], bump)]
    pub global_configuration: Account<'info, InitializeConfiguration>,

    #[account(
      mut,
      seeds = [ &mint_address.key().to_bytes(), CURVE_SEED.as_bytes()],
      bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    pub mint_address: Account<'info, Mint>,

    pub base_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub user_base_token_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_address,
        associated_token::authority = payer,
        associated_token::token_program = token_program
    )]
    pub user_quote_token_ata: Account<'info, TokenAccount>,

    ///CHECK:
    #[account(
      mut,
      seeds = [&mint_address.key().to_bytes(), POOL_SEED.as_bytes()],
      bump
    )]
    pub pool: AccountInfo<'info>,

    ///CHECK:
    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub base_token_pool: Account<'info, TokenAccount>,

    ///CHECK:
    #[account(
      mut, 
      associated_token::mint = mint_address,
      associated_token::authority = pool,
      associated_token::token_program = token_program
    )]
    pub quote_token_pool: Account<'info, TokenAccount>,

    ///CHECK:
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = fee_account,
        associated_token::token_program = token_program
    )]
    pub fee_base_token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Buy>, base_input_amount: u64, expected_amount: u64) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let k_value = bonding_curve.k_value;
    let platform_fee =
        (base_input_amount * ctx.accounts.global_configuration.swap_fee.clone()) / 10000;
    let denominator = (bonding_curve.init_virtual_base_token +
        bonding_curve.base_token_reserves +
        (base_input_amount - platform_fee)) as u128;
    let maybe_quote = (bonding_curve.quote_token_reserves as u128).checked_sub(
        k_value / denominator
    );
    let estimated_out_quote = maybe_quote.ok_or_else(||
        error!(CustomError::OverflowEstimateOutQuote)
    )? as u64;

    // Transfer fee to the fee account

    // Transfer base tokens from user to pool

    bonding_curve.base_token_reserves += base_input_amount - platform_fee;
    bonding_curve.quote_token_reserves -= estimated_out_quote;

    emit!(TransactionEvent {
        operation: "Buy".to_string(),
        creator: ctx.accounts.payer.key(),
        input_amount: base_input_amount,
        output_amount: estimated_out_quote,
        platform_fee: platform_fee,
        base_token_mint: ctx.accounts.base_token_mint.key(),
        mint_address: ctx.accounts.mint_address.key(),
        fee_wallet: ctx.accounts.fee_account.key(),
    });

    if
        bonding_curve.base_token_reserves + bonding_curve.init_virtual_base_token >=
        ctx.accounts.global_configuration.bonding_curve_limitation
    {
        emit!(BondingCurveCompleted {
            mint_address: ctx.accounts.mint_address.key(),
            user_quote_token_ata: ctx.accounts.user_quote_token_ata.key(),
            pool: ctx.accounts.pool.key(),
            quote_token_pool: ctx.accounts.quote_token_pool.key(),
        });

        bonding_curve.is_completed = true;
    }
    Ok(())
}
