//! `missing_reinit_guard` — flags accounts declared with
//! `init_if_needed` that lack a reinitialization guard.
//!
//! In Anchor, `#[account(init_if_needed, payer = user, space = ...)]`
//! creates the account on the first call and **reuses** it on
//! subsequent calls. Without an accompanying `has_one` or
//! `constraint` expression that proves the caller is the same
//! authority that previously initialized the account, **any signer
//! that pays can clobber the existing state**. They bump the
//! account's lamports above rent-exempt, set the discriminator,
//! and write whatever data they like — overwriting whatever the
//! legitimate initializer wrote.
//!
//! Severity: High — silent, repeatable, often leads to fund loss
//! when the reinitialized account is a vault, a registry entry, or
//! a config struct that gates downstream actions.
//!
//! Safe patterns (not flagged):
//!   - `has_one = <authority>` on the same field, where `<authority>`
//!     is a Signer field — Anchor's runtime check verifies the
//!     account's stored authority matches the signer.
//!   - `constraint = <field>.<authority> == <signer>.key() @ Error`
//!     — explicit same-authority check in the constraint expression.
//!   - The struct uses `#[account(init, ...)]` (not `init_if_needed`) —
//!     the account is then guaranteed to be created exactly once,
//!     at the same time as the first transaction, and reinit is
//!     impossible by construction.

use anyhow::Result;
use std::collections::HashSet;

use crate::engine::{AnalysisContext, AstHintKind, AstHint, Finding, Layer, Rule, Severity};

pub struct MissingReinitGuard;

impl Rule for MissingReinitGuard {
    fn id(&self) -> &'static str {
        "missing_reinit_guard"
    }
    fn description(&self) -> &'static str {
        "Detects init_if_needed accounts without a reinitialization guard"
    }
    fn severity(&self) -> Severity {
        Severity::High
    }
    fn layer(&self) -> Layer {
        Layer::Ast
    }

    fn check(&self, ctx: &AnalysisContext) -> Result<Vec<Finding>> {
        let mut out = Vec::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();

        for hint in &ctx.ast_hints {
            if let AstHintKind::AccountsField {
                struct_name,
                field_name,
                constraints,
                ..
            } = &hint.kind
            {
                if !has_init_if_needed(constraints) {
                    continue;
                }
                let key = (struct_name.clone(), field_name.clone());
                if !seen.insert(key) {
                    continue;
                }

                if has_reinit_guard(constraints, &ctx.ast_hints, struct_name) {
                    continue;
                }

                let mut b = Finding::builder(
                    self.id(),
                    self.severity(),
                    format!(
                        "Account `{field_name}` uses init_if_needed without a reinitialization guard — an attacker can reinitialize this account and overwrite its state."
                    ),
                )
                .program(&ctx.ir.name)
                .account(field_name)
                .hint(
                    "Add `constraint = <account>.<authority_field> == <signer>.key() @ ErrorCode::AlreadyInitialized` (or `has_one = <authority>` where `<authority>` is a Signer) to prevent reinitialization by a different caller.",
                );
                b = hint.location().stamp(b);
                out.push(b.build());
            }
        }

        Ok(out)
    }
}

/// True if the field's `#[account(...)]` attrs include `init_if_needed`.
///
/// We substring-search the joined constraints rather than parsing
/// each one — `init_if_needed` is always a bare identifier token
/// (no `= value`), so a word-boundary match is sufficient. An
/// account is *only* reinitializable if it actually carries the
/// `init_if_needed` keyword; bare `init` does NOT trigger.
fn has_init_if_needed(constraints: &[String]) -> bool {
    for c in constraints {
        // The stringified form is something like
        // "init_if_needed, payer=user, space=8+32". We tokenize by
        // comma/space and look for an exact match.
        for tok in c.split(|ch: char| ch == ',' || ch.is_whitespace()) {
            if tok == "init_if_needed" {
                return true;
            }
        }
    }
    false
}

/// True if the field's `#[account(...)]` attrs include either:
///   - `has_one = <authority>` on the **same field**, where `<authority>`
///     resolves to a `Signer` type in the same struct, OR
///   - `constraint = <expr>` (must have '=') — bare `constraint` without
///     '=' is a no-op in Anchor and does NOT guard against reinit.
///
/// `realloc` alone is NOT a guard — it only controls whether data is zeroed
/// on reallocation. It must be paired with a `constraint = ...` expression
/// that verifies the caller's authority.
fn has_reinit_guard(
    constraints: &[String],
    ast_hints: &[AstHint],
    struct_name: &str,
) -> bool {
    for c in constraints {
        let tokens: Vec<&str> = c.split(|ch: char| ch == ',' || ch.is_whitespace()).collect();

        if let Some(has_one_idx) = tokens.iter().position(|tok| *tok == "has_one") {
            if let Some(next) = tokens.get(has_one_idx + 1) {
                if *next == "=" {
                    if let Some(target) = tokens.get(has_one_idx + 2) {
                        if is_signer_field(ast_hints, struct_name, target) {
                            return true;
                        }
                    }
                }
            }
        }

        if let Some(constraint_idx) = tokens.iter().position(|tok| *tok == "constraint") {
            if let Some(next) = tokens.get(constraint_idx + 1) {
                if *next == "=" {
                    return true;
                }
            }
        }

    }
    false
}

/// Returns true if `field_name` is declared as a `Signer<'info>` type
/// in the given struct. Used to verify that a `has_one` guard actually
/// references a signer (Bug 5 fix).
fn is_signer_field(
    ast_hints: &[AstHint],
    struct_name: &str,
    field_name: &str,
) -> bool {
    for hint in ast_hints {
        if let AstHintKind::AccountsField {
            struct_name: sn,
            field_name: fn_,
            ty,
            ..
        } = &hint.kind
        {
            if sn == struct_name && fn_ == field_name {
                // Match common Signer type patterns.
                let ty_lower = ty.to_lowercase();
                return ty_lower.contains("signer")
                    || ty_lower.contains("signer<'info>")
                    || ty_lower.contains("account<signer")
                    || ty_lower == "signer";
            }
        }
    }
    false
}
