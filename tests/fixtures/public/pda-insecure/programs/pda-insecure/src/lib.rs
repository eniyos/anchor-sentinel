// In-tree fixture modeled after the well-known Sealevel Attacks "insecure
// PDA" pattern. The vault's PDA is derived with `seeds = [b"vault", owner]`
// and `bump` is taken from a *user-supplied* argument instead of the
// canonical bump — the classic "bump seed canonicalization" bug.
//
// Sentinel should pick this up via the upgraded pda_misconfig rule: the
// `bump = bump` constraint must be the canonical bump (i.e. without a
// runtime override from an instruction arg). We also use unchecked
// arithmetic on balance to fire unsafe_arithmetic.

use anchor_lang::prelude::*;

declare_id!("PDAInsec11111111111111111111111111111111111");

#[program]
pub mod pda_insecure {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>, bump: u8) -> Result<()> {
        ctx.accounts.vault.owner = ctx.accounts.user.key();
        ctx.accounts.vault.balance = 0;
        // (in real vulnerable code the bump is stored here)
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, _bump: u8, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require_keys_eq!(vault.owner, ctx.accounts.owner.key(), WithdrawError::Unauthorized);
        // BUG: unchecked subtraction.
        vault.balance = vault.balance - amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // VULN: seeds are present but the `bump` is overridden by a
    // user-supplied argument instead of the canonical bump.
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8,
        seeds = [b"vault", user.key().as_ref()],
        bump = bump,
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub owner: Signer<'info>,
    // VULN: seeds provided but no `bump` constraint at all — bump
    // canonicalization is left to runtime guesswork.
    #[account(mut, seeds = [b"vault", owner.key().as_ref()])]
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
}
