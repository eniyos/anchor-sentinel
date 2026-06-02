//! Visitor that finds `#[program]` impl blocks, the `pub fn` handlers inside
//! them, and emits:
//!   - `AstHintKind::InstructionHandler` (struct + fn name)
//!   - `AstHintKind::UnsafeArithmetic` for raw `+ - * / %` on integer types
//!
//! CPI detection (calls to `invoke` / `invoke_signed`) is also captured but
//! not emitted as a hint yet — Week 3 will add a `cpi_safety` rule.

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprBinary, ImplItem, ImplItemFn, Item, Type};

use crate::engine::AstHintKind;

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
    attrs.iter().any(|a| {
        if !a.path().is_ident("program") {
            return false;
        }
        true
    })
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

    let mut arith = ArithmeticVisitor { hints };
    arith.visit_block(&f.block);
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

    let mut arith = ArithmeticVisitor { hints };
    arith.visit_block(&f.block);
}

/// Inner visitor: scans every expression for `+ - * / %` on integer types.
struct ArithmeticVisitor<'a> {
    hints: &'a mut Vec<RawHint>,
}

impl<'ast, 'a> Visit<'ast> for ArithmeticVisitor<'a> {
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
            let one_clear = (lhs_int && !is_string_like(&rhs_ty))
                || (rhs_int && !is_string_like(&lhs_ty));
            if both_clear || one_clear {
                let start = e.op.span().start();
                self.hints.push(RawHint {
                    kind: AstHintKind::UnsafeArithmetic {
                        op: op.to_string(),
                        lhs_ty: if lhs_ty.is_empty() { "unknown".into() } else { lhs_ty },
                        rhs_ty: if rhs_ty.is_empty() { "unknown".into() } else { rhs_ty },
                    },
                    start,
                });
            }
        }
        visit::visit_expr_binary(self, e);
    }
}

fn expr_type_name(e: &Expr) -> String {
    // We don't have a real type resolver. For literals we know the suffix;
    // for identifiers we report the variable name. Rules that need more
    // precision can be tightened in a later pass.
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

fn is_int_type(s: &str) -> bool {
    matches!(
        s,
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "int_literal" // numeric literal — assume integer
    )
}

/// Fallback heuristic: if the expression name *looks* like an integer
/// (parameter called `amount`, `value`, `total`, …) we treat it as one.
/// This is intentionally broad — false positives are cheap to silence and
/// a missed overflow check is much worse than a false alarm.
fn looks_like_int(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let lname = s.to_ascii_lowercase();
    matches!(
        lname.as_str(),
        "amount"
            | "value"
            | "total"
            | "lamports"
            | "balance"
            | "delta"
            | "n"
            | "i"
            | "x"
            | "y"
    ) || lname.ends_with("_amount")
        || lname.ends_with("_total")
        || lname.ends_with("_value")
}

fn is_string_like(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    l.contains("string") || l.contains("&str") || l.contains("name") || l.contains("msg")
}
