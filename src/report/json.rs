//! JSON report output. Stable schema; snapshot-tested.

use serde::Serialize;

use crate::engine::{Finding, Severity};

#[derive(Debug, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub summary: Summary,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub total: usize,
}

pub fn build_report(findings: &[Finding]) -> Report {
    // Sort findings deterministically so JSON snapshots are stable across
    // platforms. The emission order from the rule engine depends on
    // `inventory` plugin registration, which the linker lays out differently
    // on macOS vs Linux; without a sort, CI fails with snapshot drift even
    // when the rule output is otherwise identical.
    let mut sorted: Vec<Finding> = findings.to_vec();
    sorted.sort_by(|a, b| {
        (&a.rule, &a.instruction, &a.account, a.line, a.column).cmp(&(
            &b.rule,
            &b.instruction,
            &b.account,
            b.line,
            b.column,
        ))
    });

    let mut summary = Summary {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        info: 0,
        total: sorted.len(),
    };
    for f in &sorted {
        match f.severity {
            Severity::Critical => summary.critical += 1,
            Severity::High => summary.high += 1,
            Severity::Medium => summary.medium += 1,
            Severity::Low => summary.low += 1,
            Severity::Info => summary.info += 1,
        }
    }
    Report {
        findings: sorted,
        summary,
    }
}

pub fn render(findings: &[Finding]) -> String {
    let report = build_report(findings);
    serde_json::to_string_pretty(&report).expect("finding JSON always serializes")
}
