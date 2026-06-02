// Clean version of the PDA vault: canonical bump, checked math, properly
// typed accounts. Sentinel should produce zero findings.

use anchor_lang::prelude::*;

declare_id!("PDASecur11111111111111111111111111111111111");

#[program]
pub mod pda_secure {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require_keys_eq!(vault.owner, ctx.accounts.owner.key(), WithdrawError::Unauthorized);
        vault.balance = vault
            .balance
            .checked_sub(amount)
            .ok_or(error!(WithdrawError::Overflow))?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8,
        // SAFE: bump is the canonical, runtime-resolved bump.
        seeds = [b"vault", user.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub recipient: SystemAccount<'info>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

#[error_code]
pub enum WithdrawError {
    #[msg("unauthorized")]
    Unauthorized,
    #[msg("overflow")]
    Overflow,
}
