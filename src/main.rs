//! `sentinel` CLI entry point.

mod ast;
mod cli;
mod config;
mod engine;
mod idl;
mod loader;
mod report;
mod rules;

use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Command};
use engine::{AnalysisContext, Severity};
use report::text;

/// Total wall-clock spent in the scan phases (used to drive the
/// exit-code / strict-mode decision and to populate the Statistics
/// section's timings).
struct ScanTimings {
    load: std::time::Duration,
    parse_idls: std::time::Duration,
    ast_hints: std::time::Duration,
    run_rules: std::time::Duration,
    total: std::time::Duration,
}

fn main() -> ExitCode {
    // Ctrl+C: exit 130. We don't run the spinner anymore, so the
    // "clear the spinner line" branch is gone.
    let _ = ctrlc::set_handler(|| {
        eprintln!();
        eprintln!("  ✗ Interrupted");
        std::process::exit(130);
    });

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
            format,
            strict,
            ignore,
            min_severity,
        } => cmd_scan(&path, format, strict, &ignore, min_severity),
        Command::Rules => {
            cmd_rules();
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            println!("sentinel {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_scan(
    path: &str,
    format: cli::OutputFormat,
    strict: bool,
    ignore: &[String],
    min_severity: Option<cli::MinSeverity>,
) -> Result<ExitCode> {
    let project = std::path::Path::new(path);
    if !project.exists() {
        anyhow::bail!("project path does not exist: {}", project.display());
    }

    // Load config from sentinel.toml (project root or cwd)
    let cfg = config::Config::load(project);

    // Merge CLI ignore with config ignore (CLI takes precedence)
    let mut all_ignore = cfg.ignore.clone();
    for i in ignore {
        if !all_ignore.contains(i) {
            all_ignore.push(i.clone());
        }
    }

    // Apply min_severity from config if not set via CLI
    let effective_min_severity = min_severity.or_else(|| {
        cfg.min_severity.as_ref().and_then(|s| match s.to_lowercase().as_str() {
            "info" => Some(cli::MinSeverity::Info),
            "low" => Some(cli::MinSeverity::Low),
            "medium" => Some(cli::MinSeverity::Medium),
            "high" => Some(cli::MinSeverity::High),
            "critical" => Some(cli::MinSeverity::Critical),
            _ => None,
        })
    });

    // The text-only path opens with the hero, runs the scan, then
    // prints the 6 sections in order. JSON/SARIF bypass the entire
    // text path to keep stdout byte-clean for machine consumers.
    if matches!(format, cli::OutputFormat::Text) {
        text::print_hero(&project.display().to_string(), &cfg);
    }

    // Per-stage timing. Each `Instant::now()` is right before the
    // work, `.elapsed()` right after. The five phases map to the
    // Pipeline section's 5 rows (Loaded rules / Parsed IDL / Built
    // AST / Indexed accounts / Executed security checks) plus the
    // wall-clock total.
    let t_total_start = Instant::now();

    let t_load_start = Instant::now();
    let loaded = loader::load(project, &cfg.exclude).context("loading project")?;
    if loaded.idl_files.is_empty() {
        anyhow::bail!(
            "no IDL files found. Run `anchor build` inside the project first \
             so that target/idl/*.json is populated."
        );
    }
    let t_load = t_load_start.elapsed();

    let t_parse_start = Instant::now();
    let programs = loader::parse_idls(&loaded).context("parsing IDLs")?;
    let t_parse = t_parse_start.elapsed();

    let t_ast_start = Instant::now();
    let ast_hints = ast::collect_hints(&loaded.programs).context("collecting AST hints")?;
    let t_ast = t_ast_start.elapsed();

    let t_rules_start = Instant::now();
    let mut all_findings = Vec::new();
    for ir in &programs {
        let ctx = AnalysisContext {
            ir: ir.clone(),
            ast_hints: ast_hints.clone(),
        };
        all_findings.extend(engine::run_all_rules(&ctx)?);
    }
    let t_rules = t_rules_start.elapsed();
    let t_total = t_total_start.elapsed();

    let timings = ScanTimings {
        load: t_load,
        parse_idls: t_parse,
        ast_hints: t_ast,
        run_rules: t_rules,
        total: t_total,
    };

    // Apply --ignore (merged with config).
    if !all_ignore.is_empty() {
        all_findings.retain(|f| !all_ignore.iter().any(|i| i == &f.rule));
    }

    // Apply --min-severity (config value used if CLI not set).
    let min = effective_min_severity.map(|m| m.into_severity());
    if let Some(min) = min {
        all_findings.retain(|f| f.severity >= min);
    }

    // Compute the aggregate counts once. Used by both the text
    // sections and the exit-code decision.
    let rule_count = engine::registry::list_rule_ids().len();
    let programs_count = loaded.programs.len();
    let instructions_count: usize = programs.iter().map(|p| p.instructions.len()).sum();

    match format {
        cli::OutputFormat::Sarif => {
            // Machine-readable: byte-for-byte identical to v0.1.
            println!("{}", report::sarif::render(&all_findings));
        }
        cli::OutputFormat::Json => {
            // Machine-readable: byte-for-byte identical to v0.1.
            println!("{}", report::json::render(&all_findings));
        }
        cli::OutputFormat::Text => {
            // Sectioned text output. The order is: hero (already
            // printed above) → pipeline → overview → findings →
            // statistics → verdict. Each section is a single
            // println!-driven function in `text`.
            let report = text::ScanReport {
                timings: text::ScanTimings {
                    load: timings.load,
                    parse_idls: timings.parse_idls,
                    ast_hints: timings.ast_hints,
                    run_rules: timings.run_rules,
                    total: timings.total,
                },
                programs: programs_count,
                instructions: instructions_count,
                rules_executed: rule_count,
                findings: &all_findings,
            };
            text::print_pipeline(&report.timings);
            text::print_security_overview(&all_findings);
            text::print_findings(&all_findings);
            text::print_statistics(&report);
            text::print_verdict(&all_findings);
        }
    }

    // Decide exit code.
    let has_blocking = if let Some(min) = min {
        all_findings.iter().any(|f| f.severity >= min)
    } else {
        strict && all_findings.iter().any(|f| f.severity > Severity::Info)
    };

    if has_blocking {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_rules() {
    text::print_rules_table();
}
