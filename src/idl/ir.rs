//! Unified intermediate representation (IR) that both Anchor 0.29 and 0.30+ IDLs
//! deserialize into. Rules consume this — they never touch the raw IDL types.

use serde::{Deserialize, Serialize};

/// Which IDL dialect produced this IR. Mostly informational; rules should
/// branch on the IR fields themselves, not on the version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdlVersion {
    /// Legacy Anchor IDL (≤ 0.29). Single `types` map, namespaced `accounts`.
    V29,
    /// Modern Anchor IDL (≥ 0.30). Arrays for `types`, `events`, `errors`,
    /// and per-instruction `accounts` is a flat array.
    V30Plus,
}

/// Top-level program representation handed to the rule engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramIr {
    pub version: IdlVersion,
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub accounts: Vec<AccountDef>,
    pub types: Vec<TypeDef>,
    pub events: Vec<EventDef>,
    pub errors: Vec<ErrorDef>,
    /// Source path of the IDL file, for diagnostics.
    pub source_path: String,
}

/// A single on-chain instruction exposed by the program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub accounts: Vec<InstructionAccount>,
    pub args: Vec<InstructionArg>,
    /// `discriminator` for 0.30+ IDLs (8-byte sighash).
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

/// An account referenced by an instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionAccount {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    /// Optional PDA-derivation info (0.30+).
    #[serde(default)]
    pub pda: Option<PdaDerivation>,
    /// Optional address constraint (0.30+).
    #[serde(default)]
    pub address: Option<String>,
}

/// An argument to an instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionArg {
    pub name: String,
    pub ty: String,
}

/// Account state struct (top-level `accounts:` in the IDL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDef {
    pub name: String,
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

/// User-defined type (struct or enum).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    /// `"struct"` or `"enum"`.
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<TypeField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDef {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: Option<String>,
}

/// PDA derivation metadata, present on 0.30+ instruction accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaDerivation {
    pub seeds: Vec<PdaSeed>,
    /// Optional `program` field for cross-program PDAs.
    #[serde(default)]
    pub program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaSeed {
    pub kind: String,
    #[serde(default)]
    pub value: Option<String>,
    /// Path into another account's field, for `Account::data` seeds.
    #[serde(default)]
    pub path: Option<String>,
}
