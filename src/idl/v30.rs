//! Anchor IDL ≥ 0.30 types. The deserialized form mirrors the on-disk JSON.
//!
//! Reference shape (abridged):
//! ```json
//! {
//!   "version": "0.30.0",
//!   "name": "counter",
//!   "instructions": [
//!     { "name": "initialize", "accounts": [...], "args": [...], "discriminator": [..] }
//!   ],
//!   "accounts": [ { "name": "Counter", "discriminator": [..] } ],
//!   "types": [ { "name": "Data", "kind": "struct", "type": { "kind": "struct", "fields": [...] } } ],
//!   "events": [ { "name": "...", "discriminator": [..] } ],
//!   "errors": [ { "code": 6000, "name": "..." } ]
//! }
//! ```

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IdlFile {
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
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct AccountMeta {
    pub name: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub signer: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub pda: Option<Pda>,
    /// Older IDLs (and some custom ones) use `isMut` / `isSigner` instead.
    #[serde(default, alias = "isMut")]
    pub is_mut: Option<bool>,
    #[serde(default, alias = "isSigner")]
    pub is_signer: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Pda {
    pub seeds: Vec<Seed>,
    #[serde(default)]
    pub program: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Seed {
    pub kind: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
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
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

/// A user-defined type. The inner `type` field carries the shape details.
#[derive(Debug, Deserialize)]
pub struct Ty {
    pub name: String,
    pub kind: String,
    #[serde(default, rename = "type")]
    pub ty: Option<TyBody>,
}

#[derive(Debug, Deserialize)]
pub struct TyBody {
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    #[serde(default)]
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
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct Error {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: Option<String>,
}
