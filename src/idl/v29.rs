//! Anchor IDL ≤ 0.29 (legacy) types.
//!
//! Reference shape (abridged):
//! ```json
//! {
//!   "version": "0.0.0",
//!   "name": "counter",
//!   "instructions": [
//!     { "name": "initialize", "accounts": [ { "name": "userAcc", "isMut": true, "isSigner": true } ],
//!       "args": [ { "name": "data", "type": "u64" } ] }
//!   ],
//!   "types": [ { "name": "Data", "type": { "kind": "struct", "fields": [...] } } ]
//! }
//! ```
//!
//! In 0.29 accounts are *namespaced* — they live on the instruction as a flat
//! array but the per-account shape uses `isMut`/`isSigner` rather than
//! `writable`/`signer`. The unified IR smooths over the difference.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IdlFile {
    /// IDL version string. We only branch on the major; the string is
    /// kept for diagnostics.
    #[allow(dead_code)]
    pub version: String,
    pub name: String,
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub types: Vec<Ty>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub errors: Vec<Error>,
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub accounts: Vec<AccountMeta>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Deserialize)]
pub struct AccountMeta {
    pub name: String,
    #[serde(default, alias = "isMut")]
    pub is_mut: bool,
    #[serde(default, alias = "isSigner")]
    pub is_signer: bool,
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Ty {
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(rename = "type")]
    pub ty: TyBody,
}

#[derive(Debug, Deserialize)]
pub struct TyBody {
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    /// Enum variants — not surfaced into the IR today; reserved for a
    /// future `enum_canonicalization` rule.
    #[serde(default)]
    #[allow(dead_code)]
    pub variants: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Error {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: Option<String>,
}
