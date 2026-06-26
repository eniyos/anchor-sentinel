//! Unit tests for IDL IR module

use anchor_sentinel::idl::ir::{AccountDef, IdlVersion, Instruction, ProgramIr};

#[test]
fn test_idl_version_equality() {
    assert_eq!(IdlVersion::V29, IdlVersion::V29);
    assert_eq!(IdlVersion::V30Plus, IdlVersion::V30Plus);
    assert_ne!(IdlVersion::V29, IdlVersion::V30Plus);
}

#[test]
fn test_program_ir_creation() {
    let ir = ProgramIr {
        version: IdlVersion::V30Plus,
        name: "test_program".to_string(),
        instructions: vec![],
        accounts: vec![],
        types: vec![],
        events: vec![],
        errors: vec![],
        source_path: "test.json".to_string(),
    };
    assert_eq!(ir.name, "test_program");
    assert_eq!(ir.version, IdlVersion::V30Plus);
}

#[test]
fn test_instruction_creation() {
    let ix = Instruction {
        name: "test".to_string(),
        accounts: vec![],
        args: vec![],
        discriminator: None,
    };
    assert_eq!(ix.name, "test");
    assert!(ix.discriminator.is_none());
}

#[test]
fn test_account_def_creation() {
    let account = AccountDef {
        name: "user".to_string(),
        discriminator: Some(vec![1, 2, 3, 4]),
    };
    assert_eq!(account.name, "user");
    assert!(account.discriminator.is_some());
}
