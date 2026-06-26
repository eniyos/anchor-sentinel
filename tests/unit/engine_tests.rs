//! Unit tests for engine module

use anchor_sentinel::engine::{AnalysisContext, AstHint, AstHintKind, Severity, SourceLocation};

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Low);
    assert!(Severity::Low > Severity::Info);
}

#[test]
fn test_severity_as_str() {
    assert_eq!(Severity::Critical.as_str(), "critical");
    assert_eq!(Severity::High.as_str(), "high");
    assert_eq!(Severity::Medium.as_str(), "medium");
    assert_eq!(Severity::Low.as_str(), "low");
    assert_eq!(Severity::Info.as_str(), "info");
}

#[test]
fn test_severity_parse() {
    assert_eq!(Severity::parse("critical"), Some(Severity::Critical));
    assert_eq!(Severity::parse("crit"), Some(Severity::Critical));
    assert_eq!(Severity::parse("high"), Some(Severity::High));
    assert_eq!(Severity::parse("medium"), Some(Severity::Medium));
    assert_eq!(Severity::parse("med"), Some(Severity::Medium));
    assert_eq!(Severity::parse("low"), Some(Severity::Low));
    assert_eq!(Severity::parse("info"), Some(Severity::Info));
    assert_eq!(Severity::parse("unknown"), None);
}

#[test]
fn test_analysis_context_default() {
    let ctx = AnalysisContext::default();
    assert_eq!(ctx.ir.name, "");
    assert!(ctx.ast_hints.is_empty());
}

#[test]
fn test_ast_hint_location() {
    let hint = AstHint {
        kind: AstHintKind::AccountsField {
            struct_name: "Test".to_string(),
            field_name: "field".to_string(),
            ty: "Signer<'info>".to_string(),
            constraints: vec![],
        },
        file: "test.rs".to_string(),
        line: 42,
        column: 10,
    };

    let loc = hint.location();
    assert_eq!(loc.file, "test.rs");
    assert_eq!(loc.line, 42);
    assert_eq!(loc.column, 10);
}

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Critical), "critical");
    assert_eq!(format!("{}", Severity::High), "high");
    assert_eq!(format!("{}", Severity::Medium), "medium");
    assert_eq!(format!("{}", Severity::Low), "low");
    assert_eq!(format!("{}", Severity::Info), "info");
}

#[test]
fn test_source_location() {
    let loc = SourceLocation {
        file: "test.rs".to_string(),
        line: 10,
        column: 5,
    };
    assert_eq!(loc.file, "test.rs");
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}
