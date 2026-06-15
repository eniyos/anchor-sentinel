//! Config file (`sentinel.toml`) support.
//!
//! Sentinel looks for `sentinel.toml` in two places:
//!   1. The project root (the directory containing `Anchor.toml`)
//!   2. The current working directory
//!
//! The project root takes precedence.

use std::path::{Path, PathBuf};

/// Sentinel configuration.
#[derive(Debug, Default)]
pub struct Config {
    /// Patterns of files/directories to exclude from scanning.
    pub exclude: Vec<String>,
    /// Default severity threshold (if not overridden by CLI).
    pub min_severity: Option<String>,
    /// Additional rules to ignore by default.
    pub ignore: Vec<String>,
}

impl Config {
    /// Load config from the project root, falling back to cwd.
    pub fn load(project_root: &Path) -> Self {
        // Try project root first, then cwd
        if let Some(config) = Self::load_from(project_root) {
            return config;
        }

        if let Some(cwd) = std::env::current_dir().ok() {
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
        Self::parse(&contents)
    }

    fn parse(contents: &str) -> Option<Self> {
        let mut config = Config::default();

        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key = value pairs
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Remove quotes from values
                let value = value
                    .trim_matches('"')
                    .trim_matches('\'');

                match key {
                    "exclude" => {
                        // Can be a single value or inline array
                        if value.starts_with('[') {
                            // Inline array: exclude = ["path1", "path2"]
                            let items = value
                                .trim_start_matches('[')
                                .trim_end_matches(']')
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').trim_matches('\''));
                            config.exclude.extend(items.map(String::from));
                        } else {
                            config.exclude.push(value.to_string());
                        }
                    }
                    "min_severity" => {
                        config.min_severity = Some(value.to_string());
                    }
                    "ignore" => {
                        if value.starts_with('[') {
                            let items = value
                                .trim_start_matches('[')
                                .trim_end_matches(']')
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').trim_matches('\''));
                            config.ignore.extend(items.map(String::from));
                        } else {
                            config.ignore.push(value.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        if config.exclude.is_empty() && config.min_severity.is_none() && config.ignore.is_empty() {
            None
        } else {
            Some(config)
        }
    }
}

/// Check if a path should be excluded based on patterns.
pub fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in patterns {
        // Simple glob matching for now
        if pattern.contains('*') {
            // Glob pattern: match any characters
            let glob_match = glob_match(pattern, &path_str);
            if glob_match {
                return true;
            }
        } else {
            // Exact or prefix match
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
            // For the first part, it must match from the start
            if i == 0 && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    // If pattern ends with *, it's a match
    if pattern.ends_with('*') {
        true
    } else {
        // Must match to the end
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
        assert!(is_excluded(&path, &["programs/test/src/lib.rs".to_string()]));
        assert!(!is_excluded(&path, &["programs/other".to_string()]));
    }

    #[test]
    fn test_glob_match() {
        let path = PathBuf::from("tests/fixtures/vault/src/lib.rs");
        assert!(is_excluded(&path, &["tests/*".to_string()]));
        assert!(!is_excluded(&path, &["programs/*".to_string()]));
    }

    #[test]
    fn test_config_parse() {
        let toml = r#"
            exclude = ["tests", "migrations"]
            ignore = ["missing_mut"]
            min_severity = "high"
        "#;
        let config = Config::parse(toml).unwrap();
        assert_eq!(config.exclude, vec!["tests", "migrations"]);
        assert_eq!(config.ignore, vec!["missing_mut"]);
        assert_eq!(config.min_severity, Some("high".to_string()));
    }

    #[test]
    fn test_config_parse_inline_array() {
        let toml = r#"
            exclude = ["tests/fixtures", "target/debug"]
            ignore = ["missing_mut", "unsafe_arithmetic"]
        "#;
        let config = Config::parse(toml).unwrap();
        assert_eq!(config.exclude, vec!["tests/fixtures", "target/debug"]);
        assert_eq!(config.ignore, vec!["missing_mut", "unsafe_arithmetic"]);
    }
}
