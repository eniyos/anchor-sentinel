// Vulnerable reinit fixture — used to verify the `missing_reinit_guard`
// rule fires on every `init_if_needed` account that lacks a guard.
//
// Three vulnerable patterns are exercised:
//
//   1. `init_no_guard` — bare `init_if_needed, payer = user, space = …`
//      with no `has_one` and no `constraint`. Any signer that pays can
//      overwrite the existing state.
//   2. `init_pda_no_guard` — same pattern on a PDA-derived account.
//      The PDA address is deterministic, but a different signer calling
//      the same instruction can still overwrite the stored data.
//   3. `init_mutable_no_guard` — adds `mut` for good measure; the
//      mutability does nothing to prevent reinit.

use anchor_lang::prelude::*;

declare_id!("ReinitVuln1111111111111111111111111111111111");

#[program]
pub mod reinit_vulnerable {
    use super::*;

    pub fn init_no_guard(ctx: Context<InitNoGuard>) -> Result<()> {
        // VULNERABLE: no `has_one` or `constraint` binding the original
        // initializer. Anyone paying `user` can re-clobber the account.
        Ok(())
    }

    pub fn init_pda_no_guard(ctx: Context<InitPdaNoGuard>) -> Result<()> {
        // VULNERABLE: same problem on a PDA — the PDA address is fixed
        // but the data inside is not.
        Ok(())
    }

    pub fn init_mutable_no_guard(ctx: Context<InitMutableNoGuard>) -> Result<()> {
        // VULNERABLE: `mut` is required for reinit to work, but it's not
        // a guard.
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitNoGuard<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 32)]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPdaNoGuard<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32,
        seeds = [b"state"],
        bump,
    )]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitMutableNoGuard<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 32)]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub value: u64,
}
