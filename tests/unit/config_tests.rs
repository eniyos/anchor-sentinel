//! Unit tests for config module

use anchor_sentinel::config::{Config, ExcludeConfig, IgnoreConfig, SeverityConfig};

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(config.exclude.paths.is_empty());
    assert!(config.ignore.rules.is_empty());
    assert!(config.severity.min.is_none());
}

#[test]
fn test_config_exclude_paths() {
    let config = Config::default();
    assert!(config.exclude_paths().is_empty());
}

#[test]
fn test_config_ignore_rules() {
    let config = Config::default();
    assert!(config.ignore_rules().is_empty());
}

#[test]
fn test_config_min_severity() {
    let config = Config::default();
    assert!(config.min_severity().is_none());
}

#[test]
fn test_exclude_config_default() {
    let exclude = ExcludeConfig::default();
    assert!(exclude.paths.is_empty());
}

#[test]
fn test_ignore_config_default() {
    let ignore = IgnoreConfig::default();
    assert!(ignore.rules.is_empty());
}

#[test]
fn test_severity_config_default() {
    let severity = SeverityConfig::default();
    assert!(severity.min.is_none());
}

#[test]
fn test_config_load_missing_file() {
    let temp_dir = std::env::temp_dir();
    let config = Config::load(&temp_dir);
    // Should return default when no sentinel.toml exists
    assert!(config.exclude.paths.is_empty());
}

#[test]
fn test_config_parse_empty_toml() {
    let config: Config = serde_json::from_str("{}").unwrap();
    assert!(config.exclude.paths.is_empty());
}

#[test]
fn test_glob_match_literal() {
    // Test literal matching
    let pattern = "tests/fixtures/vault";
    let path = std::path::PathBuf::from("tests/fixtures/vault");
    assert!(anchor_sentinel::config::is_excluded(&path, &[pattern.to_string()]));
}

#[test]
fn test_glob_match_prefix() {
    let pattern = "tests";
    let path = std::path::PathBuf::from("tests/fixtures/vault");
    assert!(anchor_sentinel::config::is_excluded(&path, &[pattern.to_string()]));
}

#[test]
fn test_glob_match_no_match() {
    let pattern = "src";
    let path = std::path::PathBuf::from("tests/fixtures/vault");
    assert!(!anchor_sentinel::config::is_excluded(&path, &[pattern.to_string()]));
}

#[test]
fn test_glob_match_extension() {
    let pattern = "**/*.rs";
    let path = std::path::PathBuf::from("src/lib.rs");
    assert!(anchor_sentinel::config::is_excluded(&path, &[pattern.to_string()]));
}

#[test]
fn test_glob_match_double_star() {
    let pattern = "tests/**/*.rs";
    let path = std::path::PathBuf::from("tests/fixtures/vault/lib.rs");
    assert!(anchor_sentinel::config::is_excluded(&path, &[pattern.to_string()]));
}

#[test]
fn test_is_excluded_multiple_patterns() {
    let path = std::path::PathBuf::from("tests/lib.rs");
    let patterns = vec!["tests".to_string(), "src".to_string()];
    assert!(anchor_sentinel::config::is_excluded(&path, &patterns));
}

#[test]
fn test_is_excluded_no_match() {
    let path = std::path::PathBuf::from("src/lib.rs");
    let patterns = vec!["tests".to_string()];
    assert!(!anchor_sentinel::config::is_excluded(&path, &patterns));
}
