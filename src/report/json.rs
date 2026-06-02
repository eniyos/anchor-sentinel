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
    let mut summary = Summary {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        info: 0,
        total: findings.len(),
    };
    for f in findings {
        match f.severity {
            Severity::Critical => summary.critical += 1,
            Severity::High => summary.high += 1,
            Severity::Medium => summary.medium += 1,
            Severity::Low => summary.low += 1,
            Severity::Info => summary.info += 1,
        }
    }
    Report {
        findings: findings.to_vec(),
        summary,
    }
}

pub fn render(findings: &[Finding]) -> String {
    let report = build_report(findings);
    serde_json::to_string_pretty(&report).expect("finding JSON always serializes")
}
