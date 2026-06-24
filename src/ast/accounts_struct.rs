//! Visitor that finds every `#[derive(Accounts)]` struct and extracts the
//! per-field information that rules need: type, constraints, source location.
//!
//! Spans come from `syn`'s `Span::start()` which returns a `LineColumn` we
//! can resolve to (line, column) at parse time. The `accounts_struct::Hint`
//! carries the raw span; the loader resolves it to a final (line, column).

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Fields, Item, ItemStruct};

use crate::engine::AstHintKind;

/// A single raw hint waiting for span resolution.
#[derive(Debug)]
pub struct RawHint {
    pub kind: AstHintKind,
    /// `start()` of the first token of the construct (field, etc.).
    pub start: proc_macro2::LineColumn,
}

/// State collected from a single file.
#[derive(Debug, Default)]
pub struct FileAccounts {
    pub hints: Vec<RawHint>,
}

pub struct AccountsStructVisitor<'a> {
    /// File path the visitor is attached to. Carried on the struct for
    /// future diagnostics (e.g. richer per-finding error messages) but
    /// the current visitor body doesn't read it.
    #[allow(dead_code)]
    pub file: &'a str,
    pub out: &'a mut FileAccounts,
}

impl<'a> AccountsStructVisitor<'a> {
    pub fn new(file: &'a str, out: &'a mut FileAccounts) -> Self {
        Self { file, out }
    }
}

impl<'ast, 'a> Visit<'ast> for AccountsStructVisitor<'a> {
    fn visit_item(&mut self, i: &'ast Item) {
        if let Item::Struct(s) = i {
            if has_derive_accounts(&s.attrs) {
                visit_accounts_struct(s, &mut self.out.hints);
            }
        }
        visit::visit_item(self, i);
    }
}

fn has_derive_accounts(attrs: &[Attribute]) -> bool {
    for a in attrs {
        if !a.path().is_ident("derive") {
            continue;
        }
        let parsed = a.parse_args::<syn::Ident>();
        if let Ok(ident) = parsed {
            if ident == "Accounts" {
                return true;
            }
        }
        if let syn::Meta::List(list) = &a.meta {
            let tokens = list.tokens.to_string();
            if tokens.split(',').any(|t| t.trim() == "Accounts") {
                return true;
            }
        }
    }
    false
}

fn visit_accounts_struct(s: &ItemStruct, hints: &mut Vec<RawHint>) {
    let struct_name = s.ident.to_string();
    let named = match &s.fields {
        Fields::Named(n) => &n.named,
        _ => return,
    };
    for field in named {
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_default();
        let ty = quote::ToTokens::to_token_stream(&field.ty)
            .to_string()
            .replace(char::is_whitespace, "");
        let constraints = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("account"))
            .map(stringify_attr)
            .collect::<Vec<_>>();

        hints.push(RawHint {
            kind: AstHintKind::AccountsField {
                struct_name: struct_name.clone(),
                field_name,
                ty,
                constraints,
            },
            start: field.span().start(),
        });
    }
}

fn stringify_attr(a: &Attribute) -> String {
    match &a.meta {
        syn::Meta::List(list) => list.tokens.to_string(),
        _ => String::new(),
    }
}
