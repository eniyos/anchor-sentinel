//! `sentinel` CLI entry point.

mod ast;
mod cli;
mod engine;
mod idl;
mod loader;
mod report;
mod rules;

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Command};
use engine::{AnalysisContext, Severity};

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Scan {
            path,
            json,
            strict,
            ignore,
            min_severity,
        } => cmd_scan(&path, json, strict, &ignore, min_severity),
        Command::Rules => {
            cmd_rules();
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            // clap prints the version automatically with `--version`; for
            // the explicit subcommand, print build info.
            println!("sentinel {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_scan(
    path: &str,
    json: bool,
    strict: bool,
    ignore: &[String],
    min_severity: Option<cli::MinSeverity>,
) -> Result<ExitCode> {
    let project = std::path::Path::new(path);
    if !project.exists() {
        anyhow::bail!("project path does not exist: {}", project.display());
    }

    let loaded = loader::load(project).context("loading project")?;
    if loaded.idl_files.is_empty() {
        anyhow::bail!(
            "no IDL files found. Run `anchor build` inside the project first \
             so that target/idl/*.json is populated."
        );
    }

    let programs = loader::parse_idls(&loaded).context("parsing IDLs")?;
    let ast_hints = ast::collect_hints(&loaded.programs).context("collecting AST hints")?;

    let mut all_findings = Vec::new();
    for ir in &programs {
        let ctx = AnalysisContext {
            ir: ir.clone(),
            ast_hints: ast_hints.clone(),
        };
        all_findings.extend(engine::run_all_rules(&ctx)?);
    }

    // Apply --ignore.
    if !ignore.is_empty() {
        all_findings.retain(|f| !ignore.iter().any(|i| i == &f.rule));
    }

    // Apply --min-severity.
    let min = min_severity.map(|m| m.into_severity());
    if let Some(min) = min {
        all_findings.retain(|f| f.severity >= min);
    }

    if json {
        println!("{}", report::json::render(&all_findings));
    } else {
        println!("{}", report::text::format_findings(&all_findings));
    }

    // Decide exit code.
    let has_blocking = if let Some(min) = min {
        all_findings.iter().any(|f| f.severity >= min)
    } else {
        // No explicit min — but `--strict` implies "anything non-info".
        strict && all_findings.iter().any(|f| f.severity > Severity::Info)
    };

    if has_blocking {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_rules() {
    let mut rows: Vec<(&str, Severity, &str)> = engine::registry::list_rule_ids();
    rows.sort_by_key(|(id, _, _)| *id);
    println!("{:<22} {:<10} DESCRIPTION", "RULE", "SEVERITY");
    println!("{}", "-".repeat(72));
    for (id, sev, desc) in rows {
        println!("{:<22} {:<10} {}", id, sev.as_str(), desc);
    }
}
