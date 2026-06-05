// Clean reinit fixture — used to verify the `missing_reinit_guard`
// rule does NOT fire when the `init_if_needed` account is guarded.
//
// Two safe patterns are exercised:
//
//   1. `init_with_has_one` — `has_one = authority` on the same field.
//      Anchor's runtime verifies the account's stored `authority`
//      matches the `authority` account passed in, so a different
//      signer cannot reinit.
//   2. `init_with_constraint` — explicit `constraint = state.owner ==
//      user.key() @ ErrorCode::AlreadyInitialized` does the same job
//      by hand.

use anchor_lang::prelude::*;

declare_id!("ReinitC1ean1111111111111111111111111111111111");

#[program]
pub mod reinit_clean {
    use super::*;

    pub fn init_with_has_one(ctx: Context<InitWithHasOne>) -> Result<()> {
        // SAFE: `has_one = authority` is checked by Anchor.
        Ok(())
    }

    pub fn init_with_constraint(ctx: Context<InitWithConstraint>) -> Result<()> {
        // SAFE: explicit constraint compares the stored owner with
        // the signer, returning AlreadyInitialized on mismatch.
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitWithHasOne<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 32, has_one = authority)]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitWithConstraint<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32,
        constraint = state.owner == user.key() @ StateError::AlreadyInitialized,
    )]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub value: u64,
}

#[error_code]
pub enum StateError {
    #[msg("account already initialized")]
    AlreadyInitialized,
}
