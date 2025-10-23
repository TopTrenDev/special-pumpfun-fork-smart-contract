use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    system_instruction,
    program::invoke,
    entrypoint::ProgramResult,
    program_pack::Pack,
};
use anchor_spl::{
    associated_token::{
        spl_associated_token_account::instruction::create_associated_token_account,
        AssociatedToken,
    },
    token::{
        self,
        initialize_mint,
        mint_to,
        spl_token,
        InitializeMint,
        Token,
        Mint,
        MintTo,
        TokenAccount,
        Transfer,
    },
    metadata::{ mpl_token_metadata, create_metadata_accounts_v3, Metadata },
};
use mpl_token_metadata::types::DataV2;
use anchor_spl::metadata::CreateMetadataAccountsV3;

use crate::state::{ InitializeConfiguration, BondingCurve };
use crate::consts::*;
use crate::errors::CustomError;
use crate::events::*;

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut, seeds = [CONFIG_SEED.as_bytes()], bump)]
    pub global_configuration: Account<'info, InitializeConfiguration>,

    #[account(
        init,
        payer = payer,
        seeds = [&mint_address.key().to_bytes(), CURVE_SEED.as_bytes()],
        space = 8 + BondingCurve::SIZE,
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// CHECK:
    #[account(mut)]
    pub mint_address: Signer<'info>,

    pub base_token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = base_token_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program
    )]
    pub user_base_token_ata: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub user_quote_token_ata: AccountInfo<'info>,

    /// CHECK:
    #[account(mut, seeds = [&mint_address.key().to_bytes(), POOL_SEED.as_bytes()], bump)]
    pub pool: AccountInfo<'info>,

    /// CHECK:
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = base_token_mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub base_token_pool: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub quote_token_pool: AccountInfo<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_address.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = fee_account,
        associated_token::token_program = token_program
    )]
    pub fee_base_token_ata: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub fee_quote_token_ata: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Define the create_pool function for the CreatePool instruction

pub fn handler(
    ctx: Context<CreatePool>,
    name: String,
    symbol: String,
    uri: String,
    dev_buy_amount: u64,
    initial_token_transfer_percent: u64
) -> Result<()> {
    require!(
        ctx.accounts.base_token_mint.key() == ctx.accounts.global_configuration.base_token_ca,
        CustomError::InvalidBaseToken
    );
    require!(
        ctx.accounts.fee_account.key() == ctx.accounts.global_configuration.fee_wallet,
        CustomError::InvalidFeeWallet
    );
    require!(
        ctx.accounts.user_base_token_ata.amount >
            dev_buy_amount + ctx.accounts.global_configuration.create_pool_fee_amount,
        CustomError::NotEnoughBaseToken
    );
    require!(
        initial_token_transfer_percent >= 0 && initial_token_transfer_percent <= 10000,
        CustomError::InvalidInitialTokenTransferPercent
    );

    let space = spl_token::state::Mint::LEN as u64;
    let metadata_space = 250;

    let lamports_required = Rent::get()?.minimum_balance((space + metadata_space) as usize);

    require!(ctx.accounts.payer.lamports() > lamports_required, CustomError::NotEnoughSolBalance);

    let ix = system_instruction::create_account(
        &ctx.accounts.payer.key(),
        &ctx.accounts.mint_address.key(),
        lamports_required,
        space,
        &spl_token::id()
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.mint_address.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ]
    )?;

    initialize_mint(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), InitializeMint {
            mint: ctx.accounts.mint_address.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        }),
        6,
        &ctx.accounts.payer.key(),
        Some(&ctx.accounts.payer.key())
    )?;

    ctx.accounts.initialize_token_metadata(name.clone(), symbol.clone(), uri.clone())?;

    //create user quote token ata

    // create quote token ata of the pool

    //create fee wallet quote token ata
    invoke(
        &create_associated_token_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.fee_account.key(),
            &ctx.accounts.mint_address.key(),
            &ctx.accounts.token_program.key
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.fee_quote_token_ata.to_account_info(),
            ctx.accounts.fee_account.to_account_info(),
            ctx.accounts.mint_address.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
        ]
    )?;

    let token_supply = ctx.accounts.global_configuration.initial_virtual_quote_token;

    ctx.accounts.transfer_fee_to_fee_account()?;
    ctx.accounts.mint_tokens(token_supply)?;

    let base = u128::from(ctx.accounts.global_configuration.initial_virtual_base_token);
    let quote = u128::from(ctx.accounts.global_configuration.initial_virtual_quote_token);

    let k_value = base.checked_mul(quote).ok_or(CustomError::MathOverflow)?;

    let create_pool_fee = ctx.accounts.global_configuration.create_pool_fee_amount;
    let dev_buy_fee = (dev_buy_amount * ctx.accounts.global_configuration.swap_fee.clone()) / 10000;

    let initial_base_token = u128::from(
        ctx.accounts.global_configuration.initial_virtual_base_token
    );
    let quote_token = u128::from(ctx.accounts.global_configuration.initial_virtual_quote_token);
    let dev_buy_amount = u128::from(dev_buy_amount);
    let dev_buy_fee = u128::from(dev_buy_fee);

    let base_sum = initial_base_token
        .checked_add(dev_buy_amount)
        .and_then(|v| v.checked_sub(dev_buy_fee))
        .ok_or(CustomError::MathUnderflow)?;

    let division = k_value.checked_div(base_sum).ok_or(CustomError::MathDivisionByZero)?;

    let quote_amount_u128 = quote.checked_sub(division).ok_or(CustomError::MathUnderflow)?;

    let quote_amount = u64::try_from(quote_amount_u128).map_err(|_| CustomError::MathOverflow)?;

    let initial_token_transfer_amount = quote_amount
        .checked_mul(initial_token_transfer_percent)
        .and_then(|v| v.checked_div(10_000))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(CustomError::MathOverflow)?;

    // Dev buy Instruction

    emit!(TransactionEvent {
        operation: "Dev Buy".to_string(),
        creator: ctx.accounts.payer.key(),
        input_amount: dev_buy_amount as u64,
        output_amount: quote_amount,
        platform_fee: create_pool_fee + (dev_buy_fee as u64),
        base_token_mint: ctx.accounts.base_token_mint.key(),
        mint_address: ctx.accounts.mint_address.key(),
        fee_wallet: ctx.accounts.fee_account.key(),
    });

    // authority
    ctx.accounts.set_freeze_authority()?;
    ctx.accounts.set_mint_authority()?;

    // Clone the needed values before mutably borrowing bonding_curve
    let initial_virtual_base_token = ctx.accounts.global_configuration.initial_virtual_base_token;
    let initial_virtual_quote_token = ctx.accounts.global_configuration.initial_virtual_quote_token;

    let bonding_curve = &mut ctx.accounts.bonding_curve;

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
