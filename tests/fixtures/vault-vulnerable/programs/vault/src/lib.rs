// Vulnerable vault — used to drive AST-based rule detection.
//
// All accounts are passed as `AccountInfo` (the unsafe escape hatch) and
// the deposit/withdraw handlers use raw arithmetic without `checked_*`.
// Sentinel should pick up both classes of bug.

use anchor_lang::prelude::*;

declare_id!("VauLT1111111111111111111111111111111111111");

#[program]
pub mod vault_vulnerable {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let user = &ctx.accounts.user;
        // BUG: unchecked addition.
        let new_total = vault.lamports() + amount;
        **vault.try_borrow_mut_lamports()? = new_total;
        **user.try_borrow_mut_lamports()? -= amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let destination = &ctx.accounts.destination;
        // BUG: unchecked subtraction (underflow panic possible).
        let new_total = vault.lamports() - amount;
        **vault.try_borrow_mut_lamports()? = new_total;
        **destination.try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// BUG: should be `Signer` to authorize who is depositing.
    #[account(mut)]
    pub user: AccountInfo<'info>,

    /// BUG: should be `Signer` for an authority check.
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// BUG: should be `Signer` for an authority check.
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// BUG: should be `mut` because lamports are added.
    pub destination: AccountInfo<'info>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub total: u64,
}
