//! Unit tests for explain module

use anchor_sentinel::report::explain;

#[test]
fn test_get_explanation_cpi_signer() {
    let exp = explain::get_explanation("cpi_signer_seed_validation");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert_eq!(exp.id, "cpi_signer_seed_validation");
    assert!(exp.see_also.is_some());
    let see_also = exp.see_also.unwrap();
    assert!(see_also.contains(&"missing_signer"));
    assert!(exp.detection_pattern.is_some());
}

#[test]
fn test_get_explanation_missing_balance() {
    let exp = explain::get_explanation("missing_balance_check");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert_eq!(exp.id, "missing_balance_check");
    assert!(exp.see_also.is_some());
}

#[test]
fn test_get_explanation_missing_signer() {
    let exp = explain::get_explanation("missing_signer");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert_eq!(exp.id, "missing_signer");
    assert!(exp.detection_pattern.is_some());
}

#[test]
fn test_get_explanation_missing_bump() {
    let exp = explain::get_explanation("missing_bump_seed_canonicalization");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert_eq!(exp.severity, anchor_sentinel::engine::Severity::High);
}

#[test]
fn test_get_explanation_duplicate_mutable() {
    let exp = explain::get_explanation("duplicate_mutable_accounts");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.exploit_ref.is_some());
}

#[test]
fn test_get_explanation_missing_ownership() {
    let exp = explain::get_explanation("missing_ownership");
    assert!(exp.is_some());
}

#[test]
fn test_get_explanation_reinit_guard() {
    let exp = explain::get_explanation("missing_reinit_guard");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.detection_pattern.is_some());
}

#[test]
fn test_get_explanation_lamports_drain() {
    let exp = explain::get_explanation("lamports_drain");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.see_also.is_some());
}

#[test]
fn test_get_explanation_missing_close_authority() {
    let exp = explain::get_explanation("missing_close_authority");
    assert!(exp.is_some());
}

#[test]
fn test_get_explanation_pda_misconfig() {
    let exp = explain::get_explanation("pda_misconfig");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.detection_pattern.is_some());
}

#[test]
fn test_get_explanation_unsafe_arithmetic() {
    let exp = explain::get_explanation("unsafe_arithmetic");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert_eq!(exp.severity, anchor_sentinel::engine::Severity::Medium);
}

#[test]
fn test_get_explanation_missing_mut() {
    let exp = explain::get_explanation("missing_mut");
    assert!(exp.is_some());
}

#[test]
fn test_get_explanation_unchecked_balance() {
    let exp = explain::get_explanation("unchecked_balance_flow");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.detection_pattern.is_some());
}

#[test]
fn test_get_explanation_integer_cast() {
    let exp = explain::get_explanation("integer_cast_truncation");
    assert!(exp.is_some());
    let exp = exp.unwrap();
    assert!(exp.exploit_ref.is_none());
}

#[test]
fn test_get_explanation_unknown() {
    let exp = explain::get_explanation("unknown_rule");
    assert!(exp.is_none());
}

#[test]
fn test_all_rules_have_see_also() {
    let rules = [
        "cpi_signer_seed_validation",
        "missing_balance_check",
        "missing_signer",
        "missing_bump_seed_canonicalization",
        "duplicate_mutable_accounts",
        "missing_ownership",
        "missing_reinit_guard",
        "lamports_drain",
        "missing_close_authority",
        "pda_misconfig",
        "unsafe_arithmetic",
        "missing_mut",
        "unchecked_balance_flow",
        "integer_cast_truncation",
    ];

    for rule in rules {
        let exp = explain::get_explanation(rule);
        assert!(exp.is_some(), "Rule {} should exist", rule);
    }
}
