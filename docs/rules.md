# Rules catalog

14 rules in total, organized by severity. Each rule has:

- **What it checks** — the exact pattern.
- **Layer** — `IDL` (IDL only), `AST` (Rust source only), or
  `IDL+AST` (both).
- **False positives** — when to expect noise, and how to silence it.

For programmatic access, `sentinel scan . --format json` returns
findings with a stable `rule` field matching these names.

---

## Critical

### `cpi_signer_seed_validation`  (AST)

Flags `invoke_signed` calls whose signer seeds cannot be verified at
analysis time. The runtime derives the PDA from the seeds the program
passes in — if those seeds come from user input (a function arg, a
locally-bound variable, an unresolvable expression), the attacker can
compute their own valid PDA and have the program sign for it. This
is the Sealevel-Attacks "fake PDA" / "withdraw without signing"
pattern.

**Safe seed forms (not flagged):**
- `b"literal"` byte string literal
- `ctx.bumps.<ident>` — the canonical bump Anchor manages for the PDA
- `<expr>.key().as_ref()` on a known account field

**False positives:** none in practice. If you have a custom signer
derivation scheme that uses something other than the three forms
above, the rule will fire — that's by design.

---

### `missing_balance_check`  (AST)

Flags lamports debited from an account (`-=`, `try_borrow_mut_lamports`)
without a preceding `require!` / `require_gte!` / explicit `>=` guard.
An attacker can pass `amount > account.lamports()` and either cause
an underflow panic or drain via wraparound.

**False positives:** rare. If your program has a custom guard
(`if amount > account.lamports() { return Err(...) }`), the rule
may still fire — those don't match the heuristic. Use
`--ignore missing_balance_check` to silence.

---

### `missing_signer`  (IDL+AST)

Flags account fields that are named or used like an authority
(`authority`, `admin`, `owner`, `user`, etc.) but are typed as
`AccountInfo<'info>` and not marked `signer: true` in the IDL.
`AccountInfo` is the unsafe escape hatch — there's no runtime
signer check unless you opt in.

**False positives:**
- Naming heuristic: a field called `user` that's intentionally
  non-signing (a counterparty, a recipient) will fire. The fix is
  to rename it (`counterparty`, `recipient`) or type it as the
  appropriate non-signer type (e.g. `UncheckedAccount`).
- If you don't run with `anchor build` first, the IDL may be stale
  and disagree with the source. Re-run `anchor build` to refresh.

---

## High

### `duplicate_mutable_accounts`  (IDL+AST)

Flags instructions that pass two or more mutable `AccountInfo`
arguments. An attacker can pass the same pubkey for both, and a
debit to one debits the other. The "duplicate mutable accounts"
vulnerability.

**False positives:** if you have two accounts of the same type
that you genuinely need both mutable (e.g. a swap of two user
tokens), type them as `Account<'info, T>` (which Anchor
type-checks for duplicates) instead of `AccountInfo`.

---

### `lamports_drain`  (AST)

Flags lamports explicitly zeroed (`**account.try_borrow_mut_lamports()? = 0`,
`account.set_lamports(0)`) without an authorization check. The
zeroing transfers the rent-exempt deposit to no one in particular,
or worse, to a location the attacker controls.

**False positives:** if you have a manual auth check that the AST
heuristic doesn't recognize (e.g. `if ctx.accounts.authority.key() !=
crate::id() { return Err(...) }`), the rule may still fire. Use
`--ignore lamports_drain` if you have a manual gate.

---

### `missing_bump_seed_canonicalization`  (AST)

Flags PDA accounts whose `bump` constraint is set to anything other
than `ctx.bumps.<ident>` (the canonical framework-managed bump).
Using a user-supplied or args-derived bump is the classic
canonicalization bypass.

**False positives:** very rare. If you're using Anchor 0.30+'s
implicit `bump` (no value) or `bump = ctx.bumps.foo`, the rule
won't fire.

---

### `missing_close_authority`  (AST)

Flags `close = <ident>` constraints where the close target isn't
enforced as a signer or authority. If the target is a plain
`AccountInfo`, anyone can pass their own pubkey and claim the
rent-exempt deposit on close.

**False positives:** if your close target is bound by a constraint
expression the AST can't parse (rare), the rule may fire. Verify
manually.

---

### `missing_ownership`  (IDL+AST)

Flags mutable `AccountInfo` fields on accounts named like
`vault`/`pool`/`state`/`config` that lack an `#[account(owner = ...)]`
constraint. Without an owner check, runtime code can deserialize
arbitrary data into the account.

**False positives:** rare. If you genuinely have a non-owned
account (e.g. a PDA-derived token account that holds SOL), use a
different name or `UncheckedAccount`.

---

### `pda_misconfig`  (IDL+AST)

Flags PDA accounts whose `seeds` constraint is missing a `bump`, or
whose `bump` is set to a plain identifier (`bump = bump`).
Combining the Sealevel-Attacks PDA traps with the canonicalization
gap.

**False positives:** none in practice.

---

### `missing_reinit_guard`  (AST)

Flags accounts declared with `init_if_needed` that lack a
reinitialization guard. Without `has_one = <authority>` or
`constraint = <field> == <signer>.key()` on the struct field, any
signer who pays can reinitialize the account and overwrite its
state — a silent state clobber.

**False positives:** rare. If you have a custom authority-binding
pattern that the AST heuristic doesn't recognize, the rule may fire.
Use `--ignore missing_reinit_guard` to suppress.

---

## Medium

### `integer_cast_truncation`  (AST)

Flags `as` casts between integer types where the source is wider
than the destination. Silent truncation; on-chain the value just
"loses" its high bits.

**False positives:** variable names like `amount` are assumed to
be `u64` even when typed differently. The rule skips
`usize`/`isize` and unknown types. Use `--ignore integer_cast_truncation`
if you have legitimate narrowing casts that the AST can't verify.

---

### `missing_mut`  (IDL+AST)

Flags `destination`/`recipient`/`to`/`target` accounts not declared
`#[account(mut)]`. Without `mut`, the runtime rejects the transaction
when the program tries to write to the account, but the failure
mode is harder to debug than catching it at scan time.

**False positives:** if your naming is non-standard (e.g. `dst` for
destination), the heuristic won't catch it. Either rename or use
`--ignore missing_mut`.

---

### `unchecked_balance_flow`  (IDL+AST)

Flags writable accounts debited without a matching credit or CPI
call. The lamports-conservation heuristic — if `account` is
debited in the handler but no `+=` or `invoke` to a corresponding
account is found in the same handler, something's off.

**False positives:** medium-high. The heuristic is per-handler and
doesn't trace CPI. If your handler legitimately debits one account
and credits another inside a CPI call, this may fire. Use
`--ignore unchecked_balance_flow`.

---

### `unsafe_arithmetic`  (AST)

Flags raw `+ - * / %` on integer types in `#[program]` handlers.
On overflow these panic in release builds. Wrap in `checked_*` /
`saturating_*` / `overflowing_*` to make the conversion explicit.

**False positives:** rare. If the source name is `isize`/`usize` or
unrecognized, the rule skips. Decimal literals (`1_000_000u64`)
are skipped because they can't overflow.

---

## Summary

| Rule | Layer | Severity | False-positive rate |
| ---- | ----- | -------- | ------------------- |
| `cpi_signer_seed_validation` | AST | Critical | Very low |
| `missing_balance_check` | AST | Critical | Low |
| `missing_signer` | IDL+AST | Critical | Medium (naming heuristic) |
| `duplicate_mutable_accounts` | IDL+AST | High | Low |
| `lamports_drain` | AST | High | Low |
| `missing_bump_seed_canonicalization` | AST | High | Very low |
| `missing_close_authority` | AST | High | Low |
| `missing_ownership` | IDL+AST | High | Low |
| `pda_misconfig` | IDL+AST | High | Very low |
| `missing_reinit_guard` | AST | High | Low |
| `integer_cast_truncation` | AST | Medium | Medium (variable-name heuristic) |
| `missing_mut` | IDL+AST | Medium | Medium (naming heuristic) |
| `unchecked_balance_flow` | IDL+AST | Medium | Medium-High (per-handler) |
| `unsafe_arithmetic` | AST | Medium | Low |
