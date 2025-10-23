use anchor_lang::prelude::*;

pub mod state;
pub mod consts;
pub mod events;
pub mod errors;
pub mod instructions;

use instructions::*;
use crate::state::*;

declare_id!("77Pw9AmRgWD6oqjeufeV3enKnPfkavJia7Lq8RhVRTbu");

#[program]
pub mod pumpfun_smart_contract {
    use super::*;

    pub fn initialize(
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
        initialize::handler(
            ctx,
            swap_fee,
            bonding_curve_limitation,
            initial_virtual_base_token,
            initial_virtual_quote_token,
            create_pool_fee_amount,
            base_token_ca,
            fee_wallet,
            raydium_migration_fee
        )?;

        Ok(())
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        name: String,
        symbol: String,
        uri: String,
        dev_buy_amount: u64,
        initial_token_transfer_percent: u64
    ) -> Result<()> {
        create_pool::handler(
            ctx,
            name,
            symbol,
            uri,
            dev_buy_amount,
            initial_token_transfer_percent
        )?;
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, base_input_amount: u64, expected_amount: u64) -> Result<()> {
        buy::handler(ctx, base_input_amount, expected_amount)?;
        Ok(())
    }

    pub fn sell(ctx: Context<Sell>, quote_input_amount: u64, expected_amount: u64) -> Result<()> {
        sell::handler(ctx, quote_input_amount, expected_amount)?;
        Ok(())
    }

    pub fn proxy_initialize(ctx: Context<ProxyInitialize>) -> Result<()> {
        proxy_initialize::handler(ctx)?;
        Ok(())
    }

    pub fn set_swap_fee(ctx: Context<SetGlobalConfiguration>, new_swap_fee: u64) -> Result<()> {
        ctx.accounts.global_configuration.swap_fee = new_swap_fee;
        Ok(())
    }

    pub fn set_bonding_curve_limitaion(
        ctx: Context<SetGlobalConfiguration>,
        new_bonding_curve_limitaion: u64
    ) -> Result<()> {
        ctx.accounts.global_configuration.bonding_curve_limitation = new_bonding_curve_limitaion;
        Ok(())
    }
}
