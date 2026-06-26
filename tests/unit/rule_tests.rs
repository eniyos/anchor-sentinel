//! Unit tests for rules module

use anchor_sentinel::engine::{registry, Severity};

#[test]
fn test_all_rules_registered() {
    let rule_count = registry::list_rule_ids().len();
    assert_eq!(rule_count, 14, "Should have 14 security rules");
}

#[test]
fn test_rule_severities() {
    let rule_ids = registry::list_rule_ids();

    let critical = rule_ids
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::Critical))
        .count();
    let high = rule_ids
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::High))
        .count();
    let medium = rule_ids
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::Medium))
        .count();

    assert_eq!(critical, 3, "Should have 3 critical rules");
    assert_eq!(high, 7, "Should have 7 high rules");
    assert_eq!(medium, 4, "Should have 4 medium rules");
}

#[test]
fn test_rule_ids_are_unique() {
    let rule_ids = registry::list_rule_ids();
    let mut ids: Vec<&str> = rule_ids.iter().map(|(id, _, _)| *id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 14, "All rule IDs should be unique");
}

#[test]
fn test_critical_rules_exist() {
    let rule_ids: Vec<&str> = registry::list_rule_ids()
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::Critical))
        .map(|(id, _, _)| *id)
        .collect();

    assert!(rule_ids.contains(&"cpi_signer_seed_validation"));
    assert!(rule_ids.contains(&"missing_balance_check"));
    assert!(rule_ids.contains(&"missing_signer"));
}

#[test]
fn test_high_rules_exist() {
    let rule_ids: Vec<&str> = registry::list_rule_ids()
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::High))
        .map(|(id, _, _)| *id)
        .collect();

    assert!(rule_ids.contains(&"missing_bump_seed_canonicalization"));
    assert!(rule_ids.contains(&"missing_reinit_guard"));
    assert!(rule_ids.contains(&"lamports_drain"));
    assert!(rule_ids.contains(&"missing_close_authority"));
    assert!(rule_ids.contains(&"duplicate_mutable_accounts"));
    assert!(rule_ids.contains(&"missing_ownership"));
    assert!(rule_ids.contains(&"pda_misconfig"));
}

#[test]
fn test_medium_rules_exist() {
    let rule_ids: Vec<&str> = registry::list_rule_ids()
        .iter()
        .filter(|(_, sev, _)| matches!(sev, Severity::Medium))
        .map(|(id, _, _)| *id)
        .collect();

    assert!(rule_ids.contains(&"missing_mut"));
    assert!(rule_ids.contains(&"unchecked_balance_flow"));
    assert!(rule_ids.contains(&"unsafe_arithmetic"));
    assert!(rule_ids.contains(&"integer_cast_truncation"));
}
