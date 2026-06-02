// Clean vault — used as the control fixture. Every account is properly
// typed (`Signer`, `mut`), all arithmetic is checked, and authority checks
// exist. Sentinel should report zero findings on this fixture once both
// IDL and AST layers are wired.

use anchor_lang::prelude::*;

declare_id!("VauLT2222222222222222222222222222222222222");

#[program]
pub mod vault_clean {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.payer.key();
        vault.total = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // SAFE: checked math.
        vault.total = vault
            .total
            .checked_add(amount)
            .ok_or(error!(VaultError::Overflow))?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // SAFE: checked math.
        vault.total = vault
            .total
            .checked_sub(amount)
            .ok_or(error!(VaultError::Overflow))?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = 8 + 32 + 8)]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub user: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub destination: SystemAccount<'info>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub total: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("arithmetic overflow")]
    Overflow,
}
