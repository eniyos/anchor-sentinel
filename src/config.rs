//! Config file (`sentinel.toml`) support.
//!
//! Sentinel looks for `sentinel.toml` in two places:
//!   1. The project root (the directory containing `Anchor.toml`)
//!   2. The current working directory
//!
//! The project root takes precedence.
//!
//! Supported TOML format:
//!
//!   [exclude]
//!   paths = ["tests", "migrations"]
//!
//!   [ignore]
//!   rules = ["missing_mut"]
//!
//!   [severity]
//!   min = "high"

use std::path::{Path, PathBuf};

/// Sentinel configuration.
#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    /// Nested exclude configuration.
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// Nested ignore configuration.
    #[serde(default)]
    pub ignore: IgnoreConfig,
    /// Nested severity configuration.
    #[serde(default)]
    pub severity: SeverityConfig,
}

/// Exclude configuration.
#[derive(Debug, Default, serde::Deserialize)]
pub struct ExcludeConfig {
    /// Paths to exclude from scanning.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Ignore configuration.
#[derive(Debug, Default, serde::Deserialize)]
pub struct IgnoreConfig {
    /// Rules to ignore.
    #[serde(default)]
    pub rules: Vec<String>,
}

/// Severity configuration.
#[derive(Debug, Default, serde::Deserialize)]
pub struct SeverityConfig {
    /// Minimum severity level.
    #[serde(default)]
    pub min: Option<String>,
}

impl Config {
    /// Load config from the project root, falling back to cwd.
    pub fn load(project_root: &Path) -> Self {
        if let Some(config) = Self::load_from(project_root) {
            return config;
        }

        if let Ok(cwd) = std::env::current_dir() {
            if cwd != project_root {
                if let Some(config) = Self::load_from(&cwd) {
                    return config;
                }
            }
        }

        Self::default()
    }

    fn load_from(dir: &Path) -> Option<Self> {
        let config_path = dir.join("sentinel.toml");
        if !config_path.exists() {
            return None;
        }

        let contents = std::fs::read_to_string(&config_path).ok()?;
        Self::parse(&contents).ok()
    }

    fn parse(contents: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(contents)
    }

    /// Returns the exclude paths.
    pub fn exclude_paths(&self) -> &[String] {
        &self.exclude.paths
    }

    /// Returns the ignore rules.
    pub fn ignore_rules(&self) -> &[String] {
        &self.ignore.rules
    }

    /// Returns the minimum severity.
    pub fn min_severity(&self) -> Option<&str> {
        self.severity.min.as_deref()
    }
}

/// Check if a path should be excluded based on patterns.
pub fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in patterns {
        if pattern.contains('*') {
            let glob_match = glob_match(pattern, &path_str);
            if glob_match {
                return true;
            }
        } else {
            let pattern_path = PathBuf::from(pattern);
            if path == pattern_path || path.starts_with(&pattern_path) {
                return true;
            }
        }
    }

    false
}

/// Simple glob matching (handles `*` wildcards).
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if let Some(found) = text[pos..].find(part) {
            if i == 0 && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if pattern.ends_with('*') {
        true
    } else {
        pos == text.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let path = PathBuf::from("programs/test/src/lib.rs");
        assert!(is_excluded(&path, &["programs/test".to_string()]));
        assert!(is_excluded(
            &path,
            &["programs/test/src/lib.rs".to_string()]
        ));
        assert!(!is_excluded(&path, &["programs/other".to_string()]));
    }

    #[test]
    fn test_glob_match() {
        let path = PathBuf::from("tests/fixtures/vault/src/lib.rs");
        assert!(is_excluded(&path, &["tests/*".to_string()]));
        assert!(!is_excluded(&path, &["programs/*".to_string()]));
    }

    #[test]
    fn test_config_parse_nested() {
        let toml = r#"
            [exclude]
            paths = ["src/lib.rs", "tests/**/*.rs"]

            [ignore]
            rules = ["missing_mut", "unsafe_arithmetic"]

            [severity]
            min = "critical"
        "#;
        let config = Config::parse(toml).unwrap();
        assert_eq!(
            config.exclude.paths,
            vec!["src/lib.rs", "tests/**/*.rs"]
        );
        assert_eq!(
            config.ignore.rules,
            vec!["missing_mut", "unsafe_arithmetic"]
        );
        assert_eq!(config.severity.min, Some("critical".to_string()));
    }

    #[test]
    fn test_config_parse_empty() {
        let toml = "";
        let config = Config::parse(toml).unwrap();
        assert!(config.exclude.paths.is_empty());
        assert!(config.severity.min.is_none());
        assert!(config.ignore.rules.is_empty());
    }

    #[test]
    fn test_config_parse_comments_only() {
        let toml = r#"
            # This is a comment
            # [exclude]
            # paths = ["tests"]
        "#;
        let config = Config::parse(toml).unwrap();
        assert!(config.exclude.paths.is_empty());
    }

    #[test]
    fn test_config_parse_partial() {
        let toml = r#"
            [exclude]
            paths = ["tests", "migrations"]
        "#;
        let config = Config::parse(toml).unwrap();
        assert_eq!(config.exclude.paths, vec!["tests", "migrations"]);
        assert!(config.ignore.rules.is_empty());
    }
}
