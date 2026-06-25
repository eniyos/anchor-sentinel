//! Visitor that finds `#[program]` impl blocks, the `pub fn` handlers inside
//! them, and emits:
//!   - `AstHintKind::InstructionHandler` (struct + fn name)
//!   - `AstHintKind::UnsafeArithmetic` for raw `+ - * / %` on integer types
//!   - `AstHintKind::LamportsDebit` / `LamportsCredit` for lamports +=/ -=
//!   - `AstHintKind::BalanceCheck` for `>=`, `require!`, etc. guards
//!   - `AstHintKind::LamportsZero` for `lamports = 0` / `set_lamports(0)`
//!   - `AstHintKind::CpiTransfer` for `invoke` / `invoke_signed` calls
//!   - `AstHintKind::CpiInvokeSigned` for `invoke_signed` calls with the
//!     signer-seeds safety classification that drives the
//!     `cpi_signer_seed_validation` rule.

use quote::ToTokens;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    BinOp, Expr, ExprArray, ExprBinary, ExprCall, ExprCast, ExprReference, ImplItem, ImplItemFn,
    Item, Type,
};

use crate::engine::{AstHintKind, SignerSeedClass};

/// A single raw hint waiting for span resolution.
#[derive(Debug)]
pub struct RawHint {
    pub kind: AstHintKind,
    pub start: proc_macro2::LineColumn,
}

#[derive(Debug, Default)]
pub struct FileFns {
    pub hints: Vec<RawHint>,
}

pub struct InstructionFnVisitor<'a> {
    /// File path the visitor is attached to. Carried for future
    /// diagnostics — current visitor body doesn't read it.
    #[allow(dead_code)]
    pub file: &'a str,
    pub out: &'a mut FileFns,
}

impl<'a> InstructionFnVisitor<'a> {
    pub fn new(file: &'a str, out: &'a mut FileFns) -> Self {
        Self { file, out }
    }
}

impl<'ast, 'a> Visit<'ast> for InstructionFnVisitor<'a> {
    fn visit_item(&mut self, i: &'ast Item) {
        if let Item::Mod(m) = i {
            if has_program_attr(&m.attrs) {
                let mod_name = m.ident.to_string();
                if let Some((_, items)) = &m.content {
                    for item in items {
                        if let syn::Item::Fn(f) = item {
                            visit_handler_fn(f, &mod_name, &mut self.out.hints);
                        }
                    }
                }
            }
        }
        if let Item::Impl(imp) = i {
            if has_program_attr(&imp.attrs) {
                let struct_name = match &*imp.self_ty {
                    Type::Path(p) => last_path_segment(&p.path).to_string(),
                    _ => String::new(),
                };
                for item in &imp.items {
                    if let ImplItem::Fn(f) = item {
                        visit_handler_fn_impl(f, &struct_name, &mut self.out.hints);
                    }
                }
            }
        }
        visit::visit_item(self, i);
    }
}

fn has_program_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("program"))
}

fn last_path_segment(path: &syn::Path) -> String {
    path.segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

fn visit_handler_fn(f: &syn::ItemFn, mod_name: &str, hints: &mut Vec<RawHint>) {
    let fn_name = f.sig.ident.to_string();
    let start = f.sig.ident.span().start();
    hints.push(RawHint {
        kind: AstHintKind::InstructionHandler {
            struct_name: mod_name.to_string(),
            fn_name: fn_name.clone(),
        },
        start,
    });

    let mut visitor = BalanceVisitor::new(hints);
    visitor.visit_block(&f.block);
}

fn visit_handler_fn_impl(f: &ImplItemFn, struct_name: &str, hints: &mut Vec<RawHint>) {
    let fn_name = f.sig.ident.to_string();
    let start = f.sig.ident.span().start();
    hints.push(RawHint {
        kind: AstHintKind::InstructionHandler {
            struct_name: struct_name.to_string(),
            fn_name,
        },
        start,
    });

    let mut visitor = BalanceVisitor::new(hints);
    visitor.visit_block(&f.block);
}

struct BalanceVisitor<'a> {
    hints: &'a mut Vec<RawHint>,
    seq_counter: usize,
}

impl<'a> BalanceVisitor<'a> {
    fn new(hints: &'a mut Vec<RawHint>) -> Self {
        Self {
            hints,
            seq_counter: 0,
        }
    }

    fn next_seq(&mut self) -> usize {
        let s = self.seq_counter;
        self.seq_counter += 1;
        s
    }
}

impl<'ast, 'a> Visit<'ast> for BalanceVisitor<'a> {
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        if let syn::Stmt::Macro(sm) = s {
            visit_stmt_macro(self, sm);
        }
        visit::visit_stmt(self, s);
    }

    fn visit_expr(&mut self, e: &'ast Expr) {
        visit::visit_expr(self, e);
    }

    fn visit_expr_binary(&mut self, e: &'ast ExprBinary) {
        if let Some(op) = match e.op {
            BinOp::Add(_) => Some("+"),
            BinOp::Sub(_) => Some("-"),
            BinOp::Mul(_) => Some("*"),
            BinOp::Div(_) => Some("/"),
            BinOp::Rem(_) => Some("%"),
            _ => None,
        } {
            let lhs_ty = expr_type_name(&e.left);
            let rhs_ty = expr_type_name(&e.right);
            let lhs_int = is_int_type(&lhs_ty) || looks_like_int(&lhs_ty);
            let rhs_int = is_int_type(&rhs_ty) || looks_like_int(&rhs_ty);
            let both_clear = lhs_int && rhs_int;
            let one_clear =
                (lhs_int && !is_string_like(&rhs_ty)) || (rhs_int && !is_string_like(&lhs_ty));
            if both_clear || one_clear {
                let start = e.op.span().start();
                self.hints.push(RawHint {
                    kind: AstHintKind::UnsafeArithmetic {
                        op: op.to_string(),
                        lhs_ty: if lhs_ty.is_empty() {
                            "unknown".into()
                        } else {
                            lhs_ty
                        },
                        rhs_ty: if rhs_ty.is_empty() {
                            "unknown".into()
                        } else {
                            rhs_ty
                        },
                    },
                    start,
                });
            }
        }

        match e.op {
            BinOp::SubAssign(_) => {
                if let Some((account, amount)) = extract_lamports_compound(&e.left, &e.right) {
                    let seq = self.next_seq();
                    self.hints.push(RawHint {
                        kind: AstHintKind::LamportsDebit {
                            account,
                            amount_expr: amount,
                            seq,
                        },
                        start: e.op.span().start(),
                    });
                }
            }
            BinOp::AddAssign(_) => {
                if let Some((account, amount)) = extract_lamports_compound(&e.left, &e.right) {
                    let seq = self.next_seq();
                    self.hints.push(RawHint {
                        kind: AstHintKind::LamportsCredit {
                            account,
                            amount_expr: amount,
                            seq,
                        },
                        start: e.op.span().start(),
                    });
                }
            }
            _ => {}
        }

        match e.op {
            BinOp::Ge(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Lt(_) => {
                if let Some((account, check_type)) = extract_lamports_comparison(e) {
                    let seq = self.next_seq();
                    self.hints.push(RawHint {
                        kind: AstHintKind::BalanceCheck {
                            account,
                            check_type,
                            seq,
                        },
                        start: e.op.span().start(),
                    });
                }
            }
            _ => {}
        }

        visit::visit_expr_binary(self, e);
    }

    fn visit_expr_cast(&mut self, e: &'ast ExprCast) {
        let from_ty = expr_type_name(&e.expr);
        let to_ty = type_name(&e.ty);
        if (is_int_type(&from_ty) || looks_like_int(&from_ty))
            && is_int_type(&to_ty)
            && !from_ty.is_empty()
        {
            let start = e.as_token.span.start();
            self.hints.push(RawHint {
                kind: AstHintKind::IntegerCast { from_ty, to_ty },
                start,
            });
        }
        visit::visit_expr_cast(self, e);
    }

    fn visit_expr_assign(&mut self, e: &'ast syn::ExprAssign) {
        if let Some(account) = extract_lamports_zero_assign(&e.left, &e.right) {
            let seq = self.next_seq();
            let start = e.eq_token.span.start();
            self.hints.push(RawHint {
                kind: AstHintKind::LamportsZero { account, seq },
                start,
            });
        }
        visit::visit_expr_assign(self, e);
    }

    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let method = e.method.to_string();
        if method == "set_lamports" {
            if let Some(first_arg) = e.args.first() {
                if is_zero_literal(first_arg) {
                    let account = extract_account_name(&e.receiver);
                    if !account.is_empty() {
                        let seq = self.next_seq();
                        let start = e.method.span().start();
                        self.hints.push(RawHint {
                            kind: AstHintKind::LamportsZero { account, seq },
                            start,
                        });
                    }
                }
            }
        }
        visit::visit_expr_method_call(self, e);
    }

    fn visit_expr_call(&mut self, e: &'ast ExprCall) {
        let path_str = expr_path_name(&e.func);
        if path_str.as_deref().is_some_and(is_invoke_signed_path) {
            let target = "invoke".to_string();
            let seq = self.next_seq();
            let start = e.func.span().start();
            self.hints.push(RawHint {
                kind: AstHintKind::CpiTransfer {
                    target: target.clone(),
                    seq,
                },
                start,
            });
            let (seeds, summary) = match e.args.iter().nth(2) {
                Some(arg) => classify_seed_argument(arg),
                None => (crate::engine::SignerSeedClass::Absent, String::new()),
            };
            self.hints.push(RawHint {
                kind: AstHintKind::CpiInvokeSigned {
                    target,
                    seq,
                    seeds,
                    seed_summary: summary,
                },
                start,
            });
        }
        visit::visit_expr_call(self, e);
    }

    fn visit_expr_macro(&mut self, e: &'ast syn::ExprMacro) {
        let mac_path = e.mac.path.to_token_stream().to_string().replace(' ', "");
        if mac_path == "require"
            || mac_path == "require_gte"
            || mac_path == "require_eq"
            || mac_path == "require_gt"
            || mac_path == "require_keys_eq"
            || mac_path == "require_keys_neq"
        {
            if let Some(account) = extract_account_from_require_macro(&e.mac.tokens) {
                let check_type = mac_path.clone();
                let seq = self.next_seq();
                let start = e.mac.path.span().start();
                self.hints.push(RawHint {
                    kind: AstHintKind::BalanceCheck {
                        account,
                        check_type,
                        seq,
                    },
                    start,
                });
            }
        }
        if mac_path.ends_with("invoke")
            || mac_path.ends_with("invoke_signed")
            || mac_path.ends_with("invoke::invoke")
            || mac_path.ends_with("invoke_signed::invoke_signed")
            || mac_path.ends_with("solana_program::invoke")
        {
            let target = extract_cpi_target(&e.mac.tokens);
            let seq = self.next_seq();
            let start = e.mac.path.span().start();
            self.hints.push(RawHint {
                kind: AstHintKind::CpiTransfer {
                    target: target.clone(),
                    seq,
                },
                start,
            });
            // For `invoke_signed`, additionally inspect the 3rd argument
            // (the signer-seeds array) so the cpi_signer_seed_validation
            // rule can fire Critical findings on dynamic seeds.
            if mac_path.ends_with("invoke_signed") {
                let (seeds, summary) = classify_invoke_signed_seeds(&e.mac.tokens);
                self.hints.push(RawHint {
                    kind: AstHintKind::CpiInvokeSigned {
                        target,
                        seq,
                        seeds,
                        seed_summary: summary,
                    },
                    start,
                });
            }
        }
        visit::visit_expr_macro(self, e);
    }
}

fn visit_stmt_macro(visitor: &mut BalanceVisitor, sm: &syn::StmtMacro) {
    let mac_path = sm.mac.path.to_token_stream().to_string().replace(' ', "");
    if mac_path == "require"
        || mac_path == "require_gte"
        || mac_path == "require_eq"
        || mac_path == "require_gt"
        || mac_path == "require_keys_eq"
        || mac_path == "require_keys_neq"
    {
        if let Some(account) = extract_account_from_require_macro(&sm.mac.tokens) {
            let check_type = mac_path.clone();
            let seq = visitor.next_seq();
            let start = sm.mac.path.span().start();
            visitor.hints.push(RawHint {
                kind: AstHintKind::BalanceCheck {
                    account,
                    check_type,
                    seq,
                },
                start,
            });
        }
    }
    // `invoke` / `invoke_signed` CPI detection.
    if mac_path.ends_with("invoke")
        || mac_path.ends_with("invoke_signed")
        || mac_path.ends_with("invoke::invoke")
        || mac_path.ends_with("invoke_signed::invoke_signed")
        || mac_path.ends_with("solana_program::invoke")
    {
        let target = extract_cpi_target(&sm.mac.tokens);
        let seq = visitor.next_seq();
        let start = sm.mac.path.span().start();
        visitor.hints.push(RawHint {
            kind: AstHintKind::CpiTransfer {
                target: target.clone(),
                seq,
            },
            start,
        });
        if mac_path.ends_with("invoke_signed") {
            let (seeds, summary) = classify_invoke_signed_seeds(&sm.mac.tokens);
            eprintln!("DEBUG invoke_signed: seeds={seeds:?} summary={summary:?}");
            visitor.hints.push(RawHint {
                kind: AstHintKind::CpiInvokeSigned {
                    target,
                    seq,
                    seeds,
                    seed_summary: summary,
                },
                start,
            });
        }
    }
}

/// Detect `account.lamports() -= amount` or `**account.try_borrow_mut_lamports()? -= amount`.
fn extract_lamports_compound(left: &Expr, right: &Expr) -> Option<(String, String)> {
    if is_lamports_dereference_mut(left) || is_lamports_method_call(left) {
        let account = extract_account_name(left);
        if !account.is_empty() {
            return Some((account, expr_to_string(right)));
        }
    }
    None
}

fn extract_lamports_comparison(e: &ExprBinary) -> Option<(String, String)> {
    let op_str = match e.op {
        BinOp::Ge(_) => "gte",
        BinOp::Le(_) => "lte",
        BinOp::Gt(_) => "gt",
        BinOp::Lt(_) => "lt",
        _ => return None,
    };
    if is_lamports_expression(&e.left) {
        return Some((extract_account_name(&e.left), op_str.to_string()));
    }
    if is_lamports_expression(&e.right) {
        return Some((extract_account_name(&e.right), op_str.to_string()));
    }
    None
}

fn extract_lamports_zero_assign(left: &Expr, right: &Expr) -> Option<String> {
    if !is_zero_literal(right) {
        return None;
    }
    if is_lamports_dereference_mut(left) || is_lamports_method_call(left) {
        let account = extract_account_name(left);
        if !account.is_empty() {
            return Some(account);
        }
    }
    None
}

fn is_zero_literal(e: &Expr) -> bool {
    if let Expr::Lit(l) = e {
        if let syn::Lit::Int(li) = &l.lit {
            return li.base10_digits() == "0";
        }
    }
    false
}

fn is_lamports_expression(e: &Expr) -> bool {
    is_lamports_dereference_mut(e) || is_lamports_method_call(e) || is_lamports_var(e)
}

fn is_lamports_dereference_mut(e: &Expr) -> bool {
    match e {
        Expr::Unary(u) => is_lamports_dereference_mut(&u.expr),
        Expr::Try(t) => is_lamports_dereference_mut(&t.expr),
        Expr::Paren(p) => is_lamports_dereference_mut(&p.expr),
        Expr::MethodCall(m) => {
            let method = m.method.to_string();
            method == "try_borrow_mut_lamports"
                || method == "lamports"
                || is_lamports_dereference_mut(&m.receiver)
        }
        _ => false,
    }
}

fn is_lamports_method_call(e: &Expr) -> bool {
    match e {
        Expr::MethodCall(m) => {
            let method = m.method.to_string();
            matches!(method.as_str(), "lamports" | "set_lamports")
        }
        Expr::Unary(u) => is_lamports_method_call(&u.expr),
        Expr::Paren(p) => is_lamports_method_call(&p.expr),
        _ => false,
    }
}

fn is_lamports_var(e: &Expr) -> bool {
    if let Expr::Path(p) = e {
        if let Some(seg) = p.path.segments.last() {
            let name = seg.ident.to_string();
            return name == "lamports" || name == "balance";
        }
    }
    false
}

fn extract_account_name(e: &Expr) -> String {
    match e {
        Expr::Unary(u) => extract_account_name(&u.expr),
        Expr::Try(t) => extract_account_name(&t.expr),
        Expr::Paren(p) => extract_account_name(&p.expr),
        Expr::MethodCall(m) => extract_account_name(&m.receiver),
        Expr::Field(f) => {
            let name = match &f.member {
                syn::Member::Named(i) => i.to_string(),
                syn::Member::Unnamed(_) => String::new(),
            };
            if name == "accounts" || name == "ctx" {
                extract_account_name(&f.base)
            } else {
                name
            }
        }
        Expr::Path(p) => last_path_segment(&p.path),
        _ => String::new(),
    }
}

fn extract_account_from_require_macro(tokens: &proc_macro2::TokenStream) -> Option<String> {
    // The tokens are the content between the delimiters.
    // We look for patterns like `account.lamports()`, `account.lamports`, or
    // `account.key()` / `keys_eq(account.key(), ...)`.
    let s = tokens.to_string();
    find_first_account_in_tokens(&s)
}

fn find_first_account_in_tokens(s: &str) -> Option<String> {
    // Look for patterns: `ctx.accounts.X`, `X.lamports`, `X.key()`, `X.balance`
    let cleaned = s.replace(char::is_whitespace, "");
    if let Some(idx) = cleaned.find("ctx.accounts.") {
        let after = &cleaned[idx + "ctx.accounts.".len()..];
        if let Some(end) = after.find(|c: char| !c.is_alphanumeric() && c != '_') {
            let name = &after[..end];
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    if let Some(idx) = cleaned.find(".lamports") {
        let before = &cleaned[..idx];
        if let Some(dot) = before.rfind('.') {
            let name = &before[dot + 1..];
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(name.to_string());
            }
        }
    }
    // Try `keys_eq(X.key(),` or `require_keys_eq(X.key(),`.
    for marker in &["keys_eq(", "keys_neq("] {
        if let Some(idx) = cleaned.find(marker) {
            let after = &cleaned[idx + marker.len()..];
            if let Some(comma) = after.find([',', ')']) {
                let name = &after[..comma];
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    for token in cleaned.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if !token.is_empty()
            && token.chars().next().unwrap_or(' ').is_alphabetic()
            && token != "require"
            && token != "require_gte"
            && token != "require_eq"
            && token != "Error"
            && token != "error"
        {
            return Some(token.to_string());
        }
    }
    None
}

fn extract_cpi_target(_tokens: &proc_macro2::TokenStream) -> String {
    // Heuristic: the first account-like argument in the invoke call.
    // This is intentionally rough — the rule using this hint is heuristic anyway.
    "invoke".to_string()
}

fn classify_invoke_signed_seeds(tokens: &proc_macro2::TokenStream) -> (SignerSeedClass, String) {
    // Parse the macro body as a comma-separated list of expressions.
    let parser = Punctuated::<Expr, syn::Token![,]>::parse_terminated;
    let args: Punctuated<Expr, _> = match parser.parse2(tokens.clone()) {
        Ok(p) => p,
        Err(_) => return (SignerSeedClass::Absent, String::new()),
    };
    // 3rd argument: the signer-seeds array. The 1st is the instruction,
    // the 2nd is the AccountInfos. 0-indexed, that's args[2].
    let seeds_expr = match args.iter().nth(2) {
        Some(e) => e,
        None => return (SignerSeedClass::Absent, String::new()),
    };
    classify_seed_argument(seeds_expr)
}

fn classify_seed_argument(seeds_expr: &Expr) -> (SignerSeedClass, String) {
    // Walk the outer `&[ ... ]` to find the inner slice(s).
    let inner_slices = match collect_seed_slices(seeds_expr) {
        Some(v) => v,
        None => return (SignerSeedClass::Absent, String::new()),
    };
    if inner_slices.is_empty() {
        return (SignerSeedClass::Absent, String::new());
    }

    let mut class = SignerSeedClass::Safe;
    let mut summary_parts: Vec<String> = Vec::new();
    for slice in inner_slices {
        for seed in &slice.elems {
            let (seed_safe, desc) = classify_seed_expression(seed);
            if !seed_safe {
                class = SignerSeedClass::Dynamic;
            }
            summary_parts.push(desc);
        }
        summary_parts.push("|".to_string());
    }
    if !summary_parts.is_empty() {
        summary_parts.pop(); // drop trailing `|`
    }
    (class, summary_parts.join(" "))
}

fn expr_path_name(e: &Expr) -> Option<String> {
    match e {
        Expr::Path(p) => Some(
            p.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        _ => None,
    }
}

fn is_invoke_signed_path(path: &str) -> bool {
    path == "invoke_signed"
        || path.ends_with("::invoke_signed")
        || path.ends_with("::program::invoke_signed")
}

fn collect_seed_slices(e: &Expr) -> Option<Vec<&ExprArray>> {
    let outer = strip_reference(e)?;
    let outer_array = match outer {
        Expr::Array(a) => a,
        _ => return None,
    };
    let mut out = Vec::new();
    for elem in &outer_array.elems {
        let inner = strip_reference(elem)?;
        if let Expr::Array(inner_array) = inner {
            out.push(inner_array);
        } else {
            return None;
        }
    }
    Some(out)
}

fn strip_reference(e: &Expr) -> Option<&Expr> {
    if let Expr::Reference(ExprReference { expr, .. }) = e {
        Some(expr)
    } else {
        Some(e)
    }
}

fn classify_seed_expression(e: &Expr) -> (bool, String) {
    let inner = strip_reference(e).unwrap_or(e);
    let raw = inner
        .to_token_stream()
        .to_string()
        .replace(char::is_whitespace, "");
    match inner {
        // `b"vault"` — byte string literal.
        Expr::Lit(lit) => {
            if let syn::Lit::ByteStr(_) | syn::Lit::Str(_) | syn::Lit::Byte(_) = &lit.lit {
                (true, raw)
            } else {
                (false, format!("{raw} (non-byte literal)"))
            }
        }
        // `&[bump]` — array of a single integer expression. Safe if the
        // expression is `ctx.bumps.<ident>`; dynamic otherwise.
        Expr::Array(arr) => {
            if arr.elems.len() == 1 {
                let only = &arr.elems[0];
                if is_canonical_bump(only) {
                    (true, format!("[{}]", only.to_token_stream()))
                } else {
                    (
                        false,
                        format!("[{}] (non-canonical bump)", only.to_token_stream()),
                    )
                }
            } else {
                (false, format!("{raw} (multi-element seed array)"))
            }
        }
        _ if is_canonical_bump(e) => (true, raw),
        _ if is_account_key_ref(e) => (true, raw),
        _ => (false, format!("{raw} (function arg or unresolvable)")),
    }
}

fn is_canonical_bump(e: &Expr) -> bool {
    let outer = match e {
        Expr::Field(f) => f,
        _ => return false,
    };
    let middle = match &*outer.base {
        Expr::Field(f) => f,
        _ => return false,
    };
    if !matches!(middle.member, syn::Member::Named(ref n) if n == "bumps") {
        return false;
    }
    let leaf = match &*middle.base {
        Expr::Path(p) => p,
        _ => return false,
    };
    if leaf.path.segments.len() != 1 {
        return false;
    }
    leaf.path.segments[0].ident == "ctx"
}

fn is_account_key_ref(e: &Expr) -> bool {
    if let Expr::MethodCall(call) = e {
        if call.method == "as_ref" {
            if let Expr::MethodCall(inner) = &*call.receiver {
                if inner.method == "key" {
                    return true;
                }
            }
        }
    }
    false
}

fn expr_to_string(e: &Expr) -> String {
    e.to_token_stream()
        .to_string()
        .replace(char::is_whitespace, "")
}

fn expr_type_name(e: &Expr) -> String {
    match e {
        Expr::Lit(l) => match &l.lit {
            syn::Lit::Int(_) => "int_literal".to_string(),
            _ => String::new(),
        },
        Expr::Path(p) => last_path_segment(&p.path),
        Expr::Field(f) => expr_type_name(&f.base),
        Expr::MethodCall(m) => expr_type_name(&m.receiver),
        Expr::Binary(b) => expr_type_name(&b.left),
        _ => String::new(),
    }
}

fn type_name(t: &Type) -> String {
    match t {
        Type::Path(p) => last_path_segment(&p.path),
        _ => String::new(),
    }
}

fn is_int_type(s: &str) -> bool {
    matches!(
        s,
        "u8" | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "int_literal" // numeric literal — assume integer
    )
}

fn looks_like_int(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "amount" | "value" | "total" | "lamports" | "balance" | "delta" | "n" | "i" | "x" | "y"
    ) || lower.ends_with("_amount")
        || lower.ends_with("_total")
        || lower.ends_with("_value")
}

fn is_string_like(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("string")
        || lower.contains("&str")
        || lower.contains("name")
        || lower.contains("msg")
}
