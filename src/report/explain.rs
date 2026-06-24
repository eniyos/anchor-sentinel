//! Detailed explanations for each security rule.
//!
//! These are used by `sentinel explain <rule>` to teach developers
//! why a pattern is dangerous, with vulnerable/safe examples and
//! real-world exploit references.

use crate::engine::Severity;

pub struct Explain {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: Severity,
    pub what: &'static str,
    pub why: &'static str,
    pub vulnerable_example: &'static str,
    pub safe_example: &'static str,
    pub exploit_ref: Option<&'static str>,
    pub see_also: Option<&'static [&'static str]>,
    pub detection_pattern: Option<&'static str>,
}

pub fn get_explanation(rule_id: &str) -> Option<Explain> {
    match rule_id {
        "cpi_signer_seed_validation" => Some(Explain {
            id: "cpi_signer_seed_validation",
            title: "Dynamic CPI Signer Seeds",
            severity: Severity::Critical,
            what: "An `invoke_signed` call uses signer seeds that cannot be verified at analysis time.",
            why: r#"PDA derivation uses `ctx.bumps` or `b"literal"` — safe. But if seeds contain function arguments or unresolvable expressions, an attacker can compute their own valid PDA and have the program sign for it.

This is the "fake PDA" exploit: the attacker calls the program with attacker-controlled seeds, and `invoke_signed` produces a signature for an address only the attacker knows."#,
            vulnerable_example: r#"// VULNERABLE: seeds include user-controlled `args.nonce`
invoke_signed(
    &ctx.accounts.target,
    &ctx.accounts.system_program,
    args.nonce,  // attacker controls this!
    &[
        b"account",
        ctx.accounts.user.to_account_info().key.as_ref(),
        args.nonce.to_le_bytes().as_ref(),
    ],
)"#,
            safe_example: r#"// SAFE: use canonical bump from ctx.bumps
invoke_signed(
    &ctx.accounts.target,
    &ctx.accounts.system_program,
    &[],
    &[
        b"account",
        ctx.accounts.user.to_account_info().key.as_ref(),
        &[ctx.bumps.target],
    ],
)"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#arbitrary-cpi"),
            see_also: Some(&["missing_signer", "pda_misconfig"]),
            detection_pattern: Some(r#"invoke_signed(..., seeds: [..., user_arg, ...])
// or
invoke_signed(..., seeds: [..., args.nonce, ...])"#),
        }),
        "missing_balance_check" => Some(Explain {
            id: "missing_balance_check",
            title: "Missing Balance Check",
            severity: Severity::Critical,
            what: "Lamports are debited from an account without a preceding balance check.",
            why: r#"If you subtract `amount` from an account without checking `account.lamports() >= amount`, two things can happen:

1. **Underflow panic** — The transaction fails, giving the attacker a DoS vector
2. **Wraparound drain** — If SOL prices fluctuate, the account wraps to max u64 and the attacker gets a windfall

This pattern has drained real vaults on mainnet."#,
            vulnerable_example: r#"// VULNERABLE: no balance check
pub fn withdraw(ctx: &Context<Withdraw>, amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    user.lamports -= amount;  // can underflow!
    ctx.accounts.vault.lamports += amount;
    Ok(())
}"#,
            safe_example: r#"// SAFE: check balance before debit
pub fn withdraw(ctx: &Context<Withdraw>, amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    require!(user.lamports() >= amount, MyError::InsufficientFunds);
    user.lamports -= amount;
    ctx.accounts.vault.lamports += amount;
    Ok(())
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#signer-authorization"),
            see_also: Some(&["missing_balance_check", "missing_ownership"]),
            detection_pattern: Some(r#"user.lamports -= amount;
// No preceding: require!(user.lamports() >= amount, ...)"#),
        }),
        "missing_signer" => Some(Explain {
            id: "missing_signer",
            title: "Missing Signer Check",
            severity: Severity::Critical,
            what: "An account is typed as `AccountInfo` but not marked as a signer in the IDL.",
            why: r#"`AccountInfo` is the unsafe escape hatch — there's no runtime signer check unless you opt in. If the field name suggests authority (user, authority, admin, owner), it should be a `Signer` type.

Attackers can pass any pubkey for this field and bypass authorization."#,
            vulnerable_example: r#"// IDL: signer: false (or missing)
// Rust: AccountInfo instead of Signer
#[derive(Accounts)]
pub struct WithdrawArgs<'info> {
    pub user: AccountInfo<'info>,  // not marked as signer!
    pub vault: Account<'info, Vault>,
}

// In handler:
pub fn withdraw(ctx: &Context<WithdrawArgs>, amount: u64) -> Result<()> {
    ctx.accounts.vault.amount -= amount;
    // No signer check! Anyone can withdraw.
}"#,
            safe_example: r#"// SAFE: use Signer type
#[derive(Accounts)]
pub struct WithdrawArgs<'info> {
    #[account(mut)]
    pub user: Signer<'info>,  // runtime enforces signer
    #[account(mut, has_one = user)]
    pub vault: Account<'info, Vault>,
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#signer-authorization"),
            see_also: Some(&["missing_balance_check", "missing_ownership", "pda_misconfig"]),
            detection_pattern: Some(r#"pub user: AccountInfo<'info>  // not Signer<'info>
// IDL: signer: false or absent"#),
        }),
        "missing_bump_seed_canonicalization" => Some(Explain {
            id: "missing_bump_seed_canonicalization",
            title: "Non-Canonical PDA Bump",
            severity: Severity::High,
            what: "A PDA account uses a non-canonical bump (hardcoded or from error).",
            why: r#"PDAs are derived from `find_program_address(seeds, program_id)`. The bump is the value that makes the derivation succeed. There's exactly ONE canonical bump — the highest one found.

If you hardcode `bump = authority` or `bump = first_8_bytes` instead of using `ctx.bumps.target`, an attacker can:
1. Find the canonical bump
2. Derive a different PDA with the same seeds but the canonical bump
3. Access both accounts independently

This breaks account uniqueness guarantees."#,
            vulnerable_example: r#"// VULNERABLE: non-canonical bump
#[derive(Accounts)]
pub struct InitializeArgs<'info> {
    #[account(
        init,
        payer = user,
        space = Vault::SIZE,
        seeds = [b"vault", user.key().as_ref()],
        bump = authority,  // can be anything that works!
    )]
    pub vault: Account<'info, Vault>,
    pub user: Signer<'info>,
}"#,
            safe_example: r#"// SAFE: use canonical bump
#[derive(Accounts)]
pub struct InitializeArgs<'info> {
    #[account(
        init,
        payer = user,
        space = Vault::SIZE,
        seeds = [b"vault", user.key().as_ref()],
        bump,  // uses ctx.bumps.vault (canonical)
    )]
    pub vault: Account<'info, Vault>,
    pub user: Signer<'info>,
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#bump-seed-canonicalization"),
            see_also: Some(&["pda_misconfig", "missing_signer"]),
            detection_pattern: Some(r#"seeds = [...], bump = user_bump
// or
seeds = [...], bump = args.nonce"#),
        }),
        "duplicate_mutable_accounts" => Some(Explain {
            id: "duplicate_mutable_accounts",
            title: "Duplicate Mutable Accounts",
            severity: Severity::High,
            what: "Two or more mutable `AccountInfo` accounts could reference the same pubkey.",
            why: r#"If an instruction accepts `account_a` and `account_b` both as `AccountInfo` (mutable), an attacker can pass the same pubkey for both. Operations on "different" accounts actually modify the same account.

Example: debit from `account_a`, credit to `account_b` — if they're the same, the attacker drains the account twice."#,
            vulnerable_example: r#"// VULNERABLE: both are AccountInfo
#[derive(Accounts)]
pub struct TransferArgs<'info> {
    pub from: AccountInfo<'info>,   // attacker can set = to
    pub to: AccountInfo<'info>,     // and pass same pubkey
    pub authority: Signer<'info>,
}

pub fn transfer(ctx: &Context<TransferArgs>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    from.lamports -= amount;  // subtracts from same account
    to.lamports += amount;   // adds to same account
    // Net effect: no change, but account debited!
}"#,
            safe_example: r#"// SAFE: use typed accounts with anchor constraint
#[derive(Accounts)]
pub struct TransferArgs<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut, constraint = from.owner == authority.key())]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}
// Anchor's Account<'info, T> type-checks uniqueness
// for both accounts. Can't pass the same pubkey."#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#duplicate-mutable-accounts"),
            see_also: Some(&["missing_signer", "missing_mut"]),
            detection_pattern: Some(r#"pub a: AccountInfo<'info>,  // two or more
pub b: AccountInfo<'info>,  // of these"#),
        }),
        "missing_ownership" => Some(Explain {
            id: "missing_ownership",
            title: "Missing Ownership Check",
            severity: Severity::High,
            what: "A mutable account lacks an owner constraint, allowing arbitrary data deserialization.",
            why: r#"When an account is typed as `AccountInfo` (not `Account<'info, T>`), the runtime doesn't enforce type safety. An attacker can pass a malicious program as the account, deserialize their own data, and bypass security checks.

Without `owner = my_program_id`, any account (even a TOKEN_PROGRAM account) can be passed and deserialized as your type."#,
            vulnerable_example: r#"// VULNERABLE: no owner constraint
#[derive(Accounts)]
pub struct DepositArgs<'info> {
    pub vault: AccountInfo<'info>,  // no owner check!
    pub user: Signer<'info>,
    pub system_program: AccountInfo<'info>,
}

// In handler:
pub fn deposit(ctx: &Context<DepositArgs>, amount: u64) -> Result<()> {
    let vault = Vault::try_from(&ctx.accounts.vault)?;
    // Can deserialize attacker-controlled account as Vault!
}"#,
            safe_example: r#"// SAFE: enforce owner
#[derive(Accounts)]
pub struct DepositArgs<'info> {
    #[account(mut, owner = system_program::ID)]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
// Only SystemAccount (owned by system_program) accepted
// Attacker can't pass a fake account."#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#account-data-matching"),
            see_also: Some(&["missing_signer", "pda_misconfig"]),
            detection_pattern: Some(r#"pub vault: AccountInfo<'info>
// No: owner = Program::ID or System"#),
        }),
        "missing_reinit_guard" => Some(Explain {
            id: "missing_reinit_guard",
            title: "Reinitialization Risk",
            severity: Severity::High,
            what: "`init_if_needed` without a reinit discriminator guard allows account reinitialization.",
            why: r#"`init_if_needed` skips initialization if the account already exists. But an attacker can:
1. Initialize an account with their data
2. Call your instruction with `init_if_needed`
3. Since the account exists, initialization is skipped... but only sometimes!

This creates unpredictable state. Some deployments reinit (if account is zeroed), others don't."#,
            vulnerable_example: r#"// VULNERABLE: init_if_needed without reinit guard
#[derive(Accounts)]
pub struct InitTokenArgs<'info> {
    #[account(init_if_needed, payer = user, space = Token::SIZE)]
    pub token: Account<'info, Token>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
// If attacker zeroes the account, reinitializes with their data
// If account has existing data, initialization skipped
// Inconsistent behavior!"#,
            safe_example: r#"// SAFE: add discriminator check
#[derive(Accounts)]
pub struct InitTokenArgs<'info> {
    #[account(mut, payer = user)]
    pub token: Account<'info, Token>,
    pub user: Signer<'info>,
}

pub fn init_token(ctx: &Context<InitTokenArgs>) -> Result<()> {
    // Check discriminator instead of relying on init_if_needed
    require!(
        ctx.accounts.token.discriminator == 0,
        TokenError::AlreadyInitialized
    );
    ctx.accounts.token.set_inner(Token::new());
    Ok(())
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#reinitialization"),
            see_also: Some(&["missing_close_authority", "missing_signer"]),
            detection_pattern: Some(r#"#[account(init_if_needed, ...)]
pub state: Account<'info, State>
// No: has_one = authority or constraint = ..."#),
        }),
        "lamports_drain" => Some(Explain {
            id: "lamports_drain",
            title: "Unauthorized Lamports Drain",
            severity: Severity::High,
            what: "Lamports are zeroed from an account without verifying caller authorization.",
            why: r#"Setting an account's lamports to 0 (via `set_lamports(0)` or `**account.try_borrow_mut_lamports()? = 0`) transfers the rent-exempt deposit to... nowhere.

If this is done without verifying the caller is authorized (e.g., a `close` function), anyone can drain the account's SOL."#,
            vulnerable_example: r#"// VULNERABLE: no authority check
pub fn close_token(ctx: &Context<CloseToken>) -> Result<()> {
    let dest = ctx.accounts.destination.to_account_info();
    let source = ctx.accounts.token.to_account_info();

    // Transfers lamports to destination
    **dest.lamports.borrow_mut() += source.lamports();
    **source.lamports.borrow_mut() = 0;
    // No authority check! Anyone can close.
}"#,
            safe_example: r#"// SAFE: check authority before closing
#[derive(Accounts)]
pub struct CloseToken<'info> {
    #[account(mut, close = authority)]
    pub token: Account<'info, Token>,
    pub authority: Signer<'info>,  // must be signer
}

// In handler:
pub fn close_token(ctx: &Context<CloseToken>) -> Result<()> {
    // Anchor verified authority signed this tx
    ctx.accounts.token.close(ctx.accounts.authority.to_account_info());
    Ok(())
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#closing-accounts"),
            see_also: Some(&["missing_close_authority"]),
            detection_pattern: Some(r#"**dest.lamports.borrow_mut() += source.lamports();
**source.lamports.borrow_mut() = 0;
// No authority verification before drain"#),
        }),
        "missing_close_authority" => Some(Explain {
            id: "missing_close_authority",
            title: "Missing Close Authority",
            severity: Severity::High,
            what: "Account close constraint lacks authorization check.",
            why: r#"`close = target` sends the rent-exempt lamports to `target` when the account closes. If `close` is set to a plain `AccountInfo` (not a `Signer`), anyone can provide that pubkey and claim the lamports.

This is a griefing/draining vector for accounts with significant SOL deposits."#,
            vulnerable_example: r#"// VULNERABLE: close to non-signer
#[derive(Accounts)]
pub struct CreateDataArgs<'info> {
    #[account(init, payer = user, space = 100)]
    pub data: Account<'info, Data>,
    pub user: Signer<'info>,
    #[account(close = attacker)]  // attacker controls this!
    pub data2: Account<'info, Data>,
    pub attacker: AccountInfo<'info>,  // not a signer!
    pub system_program: Program<'info, System>,
}"#,
            safe_example: r#"// SAFE: close to authority (who must sign)
#[derive(Accounts)]
pub struct CreateDataArgs<'info> {
    #[account(init, payer = user, space = 100)]
    pub data: Account<'info, Data>,
    pub user: Signer<'info>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseDataArgs<'info> {
    #[account(mut, close = authority)]
    pub data: Account<'info, Data>,
    pub authority: Signer<'info>,  // must sign
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#closing-accounts"),
            see_also: Some(&["lamports_drain", "missing_signer"]),
            detection_pattern: Some(r#"#[account(close = target)]
pub data: Account<'info, Data>
// target is AccountInfo, not Signer"#),
        }),
        "pda_misconfig" => Some(Explain {
            id: "pda_misconfig",
            title: "PDA Misconfiguration",
            severity: Severity::High,
            what: "PDA seeds are missing or use a non-canonical bump.",
            why: r#"A PDA without seeds or with a hardcoded bump can be derived by anyone. Combined with missing `mut` or `signer` constraints, this allows attackers to manipulate account state.

Without proper seeds, the program can't verify the account was created through the correct derivation path."#,
            vulnerable_example: r#"// VULNERABLE: missing bump constraint
#[derive(Accounts)]
pub struct CreateVaultArgs<'info> {
    #[account(init, payer = user, space = Vault::SIZE)]
    pub vault: Account<'info, Vault>,
    // Missing: seeds = [...], bump = ...
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}"#,
            safe_example: r#"// SAFE: proper seeds and bump
#[derive(Accounts)]
pub struct CreateVaultArgs<'info> {
    #[account(
        init,
        payer = user,
        space = Vault::SIZE,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}"#,
            exploit_ref: Some("https://github.com/coral-xyz/sealevel-attacks#bump-seed-canonicalization"),
            see_also: Some(&["missing_bump_seed_canonicalization", "missing_signer"]),
            detection_pattern: Some(r#"PDA account without seeds = [...]
// or with init but missing bump"#),
        }),
        "unsafe_arithmetic" => Some(Explain {
            id: "unsafe_arithmetic",
            title: "Unsafe Arithmetic",
            severity: Severity::Medium,
            what: "Integer arithmetic without overflow checking in a program instruction.",
            why: r#"On-chain, arithmetic overflow/underflow PANICS in release builds. This creates:
- **Denial of Service**: transactions fail, blocking the program
- **Logic bugs**: wraparound can bypass checks

While panics prevent silent data corruption, they enable DoS attacks."#,
            vulnerable_example: r#"// VULNERABLE: unchecked arithmetic
pub fn transfer(ctx: &Context<Transfer>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;

    from.amount -= amount;  // PANIC if underflow!
    to.amount += amount;   // PANIC if overflow!

    Ok(())
}"#,
            safe_example: r#"// SAFE: checked arithmetic
pub fn transfer(ctx: &Context<Transfer>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;

    from.amount = from.amount.checked_sub(amount)
        .ok_or(MyError::InsufficientFunds)?;
    to.amount = to.amount.checked_add(amount)
        .ok_or(MyError::Overflow)?;

    Ok(())
}"#,
            exploit_ref: None,
            see_also: Some(&["missing_ownership"]),
            detection_pattern: Some(r#"from.amount -= amount;  // unchecked -, +, *, /
to.amount += amount;     // no .checked_*()"#),
        }),
        "missing_mut" => Some(Explain {
            id: "missing_mut",
            title: "Missing Mut Constraint",
            severity: Severity::Medium,
            what: "An account that should be mutable is not declared with `#[account(mut)]`.",
            why: r#"Without `#[account(mut)]`, the runtime rejects transactions that modify the account. The error is confusing (`transaction failed`) rather than a clear security signal, making debugging harder.

It's also a code smell — if the handler modifies an account but the IDL doesn't show it as mutable, there's a mismatch."#,
            vulnerable_example: r#"// IDL says: writable: false
// But handler tries to modify:
pub fn update(ctx: &Context<Update>, new_value: u64) -> Result<()> {
    ctx.accounts.data.value = new_value;
    // Runtime REJECTS this tx!
    // Error: "transaction failed" (confusing)
}"#,
            safe_example: r#"// SAFE: declare mut in IDL
// IDL: accounts: [{ name: "data", writable: true }]
#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]  // explicitly declare mutation
    pub data: Account<'info, MyData>,
}

pub fn update(ctx: &Context<Update>, new_value: u64) -> Result<()> {
    ctx.accounts.data.value = new_value;
    Ok(())
}"#,
            exploit_ref: None,
            see_also: Some(&["missing_balance_check", "lamports_drain"]),
            detection_pattern: Some(r#"#[account(mut)]  // in IDL but missing
// or IDL: writable: false but modified"#),
        }),
        "unchecked_balance_flow" => Some(Explain {
            id: "unchecked_balance_flow",
            title: "Unchecked Balance Flow",
            severity: Severity::Medium,
            what: "An account is debited without a matching credit or CPI call.",
            why: r#"Lamports conservation: if an account loses lamports in an instruction, something else should gain them (or the account should be rent-exempt closing).

Unchecked debits often indicate a logic bug: you're removing funds but not accounting for where they go."#,
            vulnerable_example: r#"// Debits user but never credits anyone else
pub fn withdraw_all(ctx: &Context<Withdraw>) -> Result<()> {
    let user = &mut ctx.accounts.user;

    user.lamports -= user.lamports();  // all lamports removed
    // Where did they go? Bug!

    Ok(())
}"#,
            safe_example: r#"// Credit the destination explicitly
pub fn withdraw_all(ctx: &Context<Withdraw>) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let dest = &mut ctx.accounts.destination;

    let amount = user.lamports();
    **user.lamports.borrow_mut() = 0;
    **dest.lamports.borrow_mut() += amount;

    Ok(())
}"#,
            exploit_ref: None,
            see_also: Some(&["unsafe_arithmetic", "missing_balance_check"]),
            detection_pattern: Some(r#"user.lamports -= user.lamports();
// No credit to another account or CPI"#),
        }),
        "integer_cast_truncation" => Some(Explain {
            id: "integer_cast_truncation",
            title: "Integer Cast Truncation",
            severity: Severity::Medium,
            what: "An integer cast narrows the value, silently dropping high bits.",
            why: r#"`as` casts don't check for overflow. Casting `u64` to `u32` silently drops the high 32 bits. On-chain, this can:
- Bypass quantity limits
- Allow unauthorized transfers
- Create phantom balances

The bug is silent — no panic, just lost data."#,
            vulnerable_example: r#"// VULNERABLE: silent truncation
pub fn transfer_amount(ctx: &Context<Transfer>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;

    let small_amount = amount as u32;  // truncate!
    // If amount > u32::MAX, silently loses high bits
    // Attacker can transfer more than intended!

    from.balance -= small_amount;
    to.balance += small_amount;
    Ok(())
}"#,
            safe_example: r#"// SAFE: explicit checked conversion
pub fn transfer_amount(ctx: &Context<Transfer>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;

    // Check bounds before cast
    require!(amount <= u32::MAX as u64, MyError::AmountTooLarge);
    let small_amount = amount as u32;

    from.balance -= small_amount;
    to.balance += small_amount;
    Ok(())
}"#,
            exploit_ref: None,
            see_also: Some(&["unsafe_arithmetic", "missing_balance_check"]),
            detection_pattern: Some(r#"let val = amount as u32;  // u64 -> u32 cast
// High bits silently dropped"#),
        }),
        _ => None,
    }
}
