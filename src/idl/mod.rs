//! IDL loader: detect Anchor IDL version and produce a unified `ProgramIr`.
//!
//! Auto-detect heuristic:
//!   - Top-level `"version"` string starting with `"0.30"` or later → 0.30+.
//!   - Otherwise (e.g. `"0.0.0"`, `"0.29.0"`, missing) → legacy 0.29.
//!   - When uncertain we fall through to legacy first, then modern, and accept
//!     whichever succeeds.

pub mod ir;
pub mod v29;
pub mod v30;

use anyhow::{anyhow, Context, Result};
use std::path::Path;

use ir::{
    AccountDef, ErrorDef, EventDef, IdlVersion, Instruction, InstructionAccount, InstructionArg,
    PdaDerivation, PdaSeed, ProgramIr, TypeDef, TypeField,
};

/// Load a single IDL file from disk and convert to the unified IR.
pub fn load_idl(path: &Path) -> Result<ProgramIr> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading IDL file {}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing IDL file {} as JSON", path.display()))?;
    from_value(json, &path.display().to_string())
}

/// Build a `ProgramIr` from a raw `serde_json::Value`, auto-detecting the
/// IDL dialect.
pub fn from_value(json: serde_json::Value, source_path: &str) -> Result<ProgramIr> {
    let version_str = json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    let detected = detect_version(&version_str);
    match detected {
        IdlVersion::V30Plus => {
            let parsed: v30::IdlFile = serde_json::from_value(json)
                .with_context(|| format!("deserializing IDL (v30) from {source_path}"))?;
            Ok(convert_v30(parsed, source_path))
        }
        IdlVersion::V29 => {
            let parsed: v29::IdlFile = serde_json::from_value(json)
                .with_context(|| format!("deserializing IDL (v29) from {source_path}"))?;
            Ok(convert_v29(parsed, source_path))
        }
    }
}

fn detect_version(version_str: &str) -> IdlVersion {
    // Strip a leading "0." and parse the minor.
    let minor = version_str
        .strip_prefix("0.")
        .and_then(|s| s.split('.').next())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if minor >= 30 {
        IdlVersion::V30Plus
    } else {
        IdlVersion::V29
    }
}

fn convert_v30(idl: v30::IdlFile, source_path: &str) -> ProgramIr {
    let instructions = idl
        .instructions
        .into_iter()
        .map(|ix| Instruction {
            name: ix.name,
            accounts: ix
                .accounts
                .into_iter()
                .map(|a| {
                    let (is_mut, is_signer) = match (a.is_mut, a.is_signer) {
                        (Some(m), Some(s)) => (m, s),
                        _ => (a.writable, a.signer),
                    };
                    InstructionAccount {
                        name: a.name,
                        is_mut,
                        is_signer,
                        pda: a.pda.map(|p| PdaDerivation {
                            seeds: p
                                .seeds
                                .into_iter()
                                .map(|s| PdaSeed {
                                    kind: s.kind,
                                    value: s.value,
                                    path: s.path,
                                })
                                .collect(),
                            program: p.program,
                        }),
                        address: a.address,
                    }
                })
                .collect(),
            args: ix
                .args
                .into_iter()
                .map(|a| InstructionArg {
                    name: a.name,
                    ty: a.ty,
                })
                .collect(),
            discriminator: ix.discriminator,
        })
        .collect();

    let accounts = idl
        .accounts
        .into_iter()
        .map(|a| AccountDef {
            name: a.name,
            discriminator: a.discriminator,
        })
        .collect();

    let types = idl
        .types
        .into_iter()
        .map(|t| {
            let fields =
                t.ty.as_ref()
                    .map(|b| {
                        b.fields
                            .iter()
                            .map(|f| TypeField {
                                name: f.name.clone(),
                                ty: f.ty.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
            TypeDef {
                name: t.name,
                kind: t.kind,
                fields,
            }
        })
        .collect();

    let events = idl
        .events
        .into_iter()
        .map(|e| EventDef { name: e.name })
        .collect();

    let errors = idl
        .errors
        .into_iter()
        .map(|e| ErrorDef {
            code: e.code,
            name: e.name,
            msg: e.msg,
        })
        .collect();

    ProgramIr {
        version: IdlVersion::V30Plus,
        name: idl.name,
        instructions,
        accounts,
        types,
        events,
        errors,
        source_path: source_path.to_string(),
    }
}

fn convert_v29(idl: v29::IdlFile, source_path: &str) -> ProgramIr {
    let instructions = idl
        .instructions
        .into_iter()
        .map(|ix| Instruction {
            name: ix.name,
            accounts: ix
                .accounts
                .into_iter()
                .map(|a| InstructionAccount {
                    name: a.name,
                    is_mut: a.is_mut,
                    is_signer: a.is_signer,
                    pda: None,
                    address: None,
                })
                .collect(),
            args: ix
                .args
                .into_iter()
                .map(|a| InstructionArg {
                    name: a.name,
                    ty: a.ty,
                })
                .collect(),
            discriminator: None,
        })
        .collect();

    let accounts = idl
        .accounts
        .into_iter()
        .map(|a| AccountDef {
            name: a.name,
            discriminator: None,
        })
        .collect();

    let types = idl
        .types
        .into_iter()
        .map(|t| TypeDef {
            name: t.name,
            kind: t.kind.unwrap_or_else(|| t.ty.kind.clone()),
            fields: t
                .ty
                .fields
                .into_iter()
                .map(|f| TypeField {
                    name: f.name,
                    ty: f.ty,
                })
                .collect(),
        })
        .collect();

    let events = idl
        .events
        .into_iter()
        .map(|e| EventDef { name: e.name })
        .collect();

    let errors = idl
        .errors
        .into_iter()
        .map(|e| ErrorDef {
            code: e.code,
            name: e.name,
            msg: e.msg,
        })
        .collect();

    ProgramIr {
        version: IdlVersion::V29,
        name: idl.name,
        instructions,
        accounts,
        types,
        events,
        errors,
        source_path: source_path.to_string(),
    }
}

/// Find every `*.json` IDL file in a project. Anchor places them at
/// `target/idl/<program_name>.json` after `anchor build`.
pub fn discover_idl_files(project: &Path) -> Result<Vec<std::path::PathBuf>> {
    use walkdir::WalkDir;

    let target_dir = project.join("target").join("idl");
    let mut found = Vec::new();

    if target_dir.is_dir() {
        for entry in WalkDir::new(&target_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                found.push(p.to_path_buf());
            }
        }
    }

    if found.is_empty() {
        return Err(anyhow!(
            "no IDL files found under {}. Run `anchor build` first to generate target/idl/.",
            target_dir.display()
        ));
    }

    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_v30_idl() {
        let v = json!({
            "version": "0.30.0",
            "name": "demo",
            "instructions": [{
                "name": "initialize",
                "accounts": [{ "name": "user", "writable": true, "signer": true }],
                "args": [],
                "discriminator": [1,2,3,4,5,6,7,8]
            }],
            "types": [],
            "events": [],
            "errors": []
        });
        let ir = from_value(v, "<test>").unwrap();
        assert_eq!(ir.version, IdlVersion::V30Plus);
        assert_eq!(ir.instructions.len(), 1);
        assert!(ir.instructions[0].discriminator.is_some());
    }

    #[test]
    fn detects_v29_idl() {
        let v = json!({
            "version": "0.0.0",
            "name": "demo",
            "instructions": [{
                "name": "initialize",
                "accounts": [{ "name": "user", "isMut": true, "isSigner": true }],
                "args": []
            }],
            "types": [],
            "events": [],
            "errors": []
        });
        let ir = from_value(v, "<test>").unwrap();
        assert_eq!(ir.version, IdlVersion::V29);
        let acct = &ir.instructions[0].accounts[0];
        assert!(acct.is_mut);
        assert!(acct.is_signer);
    }

    #[test]
    fn v30_optional_is_signer_field_falls_back() {
        // Some IDL generators omit `writable/signer` and use `isMut/isSigner`.
        let v = json!({
            "version": "0.30.1",
            "name": "demo",
            "instructions": [{
                "name": "noop",
                "accounts": [{ "name": "user", "isMut": false, "isSigner": true }],
                "args": [],
                "discriminator": [0, 0, 0, 0, 0, 0, 0, 0]
            }],
            "types": [],
            "events": [],
            "errors": []
        });
        let ir = from_value(v, "<test>").unwrap();
        assert!(ir.instructions[0].accounts[0].is_signer);
        assert!(!ir.instructions[0].accounts[0].is_mut);
    }
}
