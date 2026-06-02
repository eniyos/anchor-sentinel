//! Human-readable CLI output.

use colored::*;

use crate::engine::{Finding, Severity};

/// Format a finding as the multi-line block shown in the CLI.
pub fn format_finding(f: &Finding) -> String {
    let sev = format_severity(f.severity);
    let mut lines = Vec::new();
    lines.push(format!("[{}] {}", sev, f.rule.bold()));

    if let Some(ix) = &f.instruction {
        lines.push(format!("  instruction: {}", ix));
    }
    if let Some(acct) = &f.account {
        lines.push(format!("  account:     {}", acct));
    }
    if let (Some(file), Some(line)) = (&f.file, f.line) {
        let col = f.column.map(|c| format!(":{c}")).unwrap_or_default();
        lines.push(format!("  location:    {}:{line}{col}", file));
    }
    lines.push(format!("  message:     {}", f.message));
    if let Some(hint) = &f.hint {
        lines.push(format!("  hint:        {}", hint.dimmed()));
    }
    lines.join("\n")
}

fn format_severity(s: Severity) -> ColoredString {
    let label = s.as_str().to_uppercase();
    match s {
        Severity::Critical => label.red().bold(),
        Severity::High => label.red(),
        Severity::Medium => label.yellow(),
        Severity::Low => label.blue(),
        Severity::Info => label.green(),
    }
}

pub fn format_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "✔ no findings".green().to_string();
    }
    let mut out = Vec::new();
    for f in findings {
        out.push(format_finding(f));
    }
    out.push(String::new());
    out.push(format_summary(findings));
    out.join("\n")
}

pub fn format_summary(findings: &[Finding]) -> String {
    let mut counts = [0usize; 5];
    for f in findings {
        let idx = match f.severity {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        };
        counts[idx] += 1;
    }
    format!(
        "summary: {} critical, {} high, {} medium, {} low, {} info",
        counts[4], counts[3], counts[2], counts[1], counts[0]
    )
}
