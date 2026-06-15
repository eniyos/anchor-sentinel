//! Anchor IDL ≥ 0.31 types (metadata-wrapped format).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IdlFile {
    #[allow(dead_code)]
    pub address: Option<String>,
    pub metadata: Metadata,
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
pub struct Metadata {
    pub name: String,
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub spec: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub accounts: Vec<AccountMeta>,
    pub args: Vec<Arg>,
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
    #[allow(dead_code)]
    #[serde(default)]
    pub returns: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct AccountMeta {
    pub name: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub signer: bool,
    #[allow(dead_code)]
    #[serde(default)]
    pub optional: Option<bool>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub pda: Option<Pda>,
    #[serde(default)]
    pub relations: Vec<String>,
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
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub name: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub ty: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub name: String,
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct Ty {
    pub name: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub ty: Option<TyBody>,
}

impl Ty {
    pub fn body(&self) -> Option<&TyBody> {
        self.ty.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct TyBody {
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    #[allow(dead_code)]
    #[serde(default)]
    pub variants: Vec<Variant>,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Variant {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub name: String,
    #[allow(dead_code)]
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
