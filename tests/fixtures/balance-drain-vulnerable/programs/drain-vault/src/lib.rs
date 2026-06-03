// Balance-drain vulnerable fixture — tests the new balance rules:
//   - missing_balance_check: debit without guard
//   - lamports_drain: zero lamports without auth
//   - unchecked_balance_flow: conservation violations

use anchor_lang::prelude::*;

declare_id!("DrainVauLT1111111111111111111111111111111111");

#[program]
pub mod drain_vault {
    use super::*;

    // SAFE: has balance check before debit — should NOT trigger missing_balance_check.
    pub fn safe_withdraw(ctx: Context<SafeWithdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        require!(vault.lamports() >= amount, VaultError::InsufficientFunds);
        **vault.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.destination.try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    // VULN: debit without balance check — triggers missing_balance_check.
    pub fn unsafe_withdraw(ctx: Context<UnsafeWithdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        // No require! or checked_sub before this debit.
        **vault.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.destination.try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    // VULN: lamports zeroed without authorization — triggers lamports_drain.
    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        let account = &ctx.accounts.target_account;
        // No signer/authority check before zeroing.
        **account.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    // VULN: set_lamports(0) without auth — triggers lamports_drain.
    pub fn drain_close(ctx: Context<DrainClose>) -> Result<()> {
        let account = &ctx.accounts.target_account;
        account.set_lamports(0);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SafeWithdraw<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UnsafeWithdraw<'info> {
    /// VULN: not a signer, no has_one check.
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    /// VULN: not a signer — anyone can close.
    pub closer: AccountInfo<'info>,
    #[account(mut)]
    pub target_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DrainClose<'info> {
    /// VULN: not a signer.
    pub closer: AccountInfo<'info>,
    #[account(mut)]
    pub target_account: AccountInfo<'info>,
}

#[account]
pub struct VaultState {
    pub authority: Pubkey,
    pub total: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("insufficient funds")]
    InsufficientFunds,
}
