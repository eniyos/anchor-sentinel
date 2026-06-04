//! `sentinel` CLI entry point.

mod ast;
mod cli;
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
use report::spinner::Spinner;
use report::text;

fn main() -> ExitCode {
    // Ctrl+C: clear the spinner line (best-effort) and exit 130 in red.
    // The `termination` feature on ctrlc forwards SIGINT/SIGTERM to
    // the default handler so the process actually exits; without it
    // we'd just install a no-op handler.
    let _ = ctrlc::set_handler(|| {
        use std::io::Write;
        // Clear any spinner line.
        let _ = std::io::stderr().write_all(b"\r\x1b[2K\r");
        let _ = std::io::stderr().flush();
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

    // Spinner + header only apply to the human-readable text format.
    // For JSON and SARIF, we must keep stdout byte-clean so the
    // machine-readable output stays parseable.
    let rule_count = engine::registry::list_rule_ids().len();
    if matches!(format, cli::OutputFormat::Text) {
        text::print_header(&project.display().to_string(), rule_count);
    }

    let spinner = Spinner::start("Loading IDL files...");
    let loaded = loader::load(project).context("loading project")?;
    if loaded.idl_files.is_empty() {
        spinner.finish();
        anyhow::bail!(
            "no IDL files found. Run `anchor build` inside the project first \
             so that target/idl/*.json is populated."
        );
    }
    let programs = loader::parse_idls(&loaded).context("parsing IDLs")?;
    spinner.set_message("Parsing Rust source...");
    let ast_hints = ast::collect_hints(&loaded.programs).context("collecting AST hints")?;
    spinner.set_message(&format!("Running {rule_count} rules..."));

    let mut all_findings = Vec::new();
    for ir in &programs {
        let ctx = AnalysisContext {
            ir: ir.clone(),
            ast_hints: ast_hints.clone(),
        };
        all_findings.extend(engine::run_all_rules(&ctx)?);
    }
    // Stop the spinner before we print anything else.
    spinner.finish();

    // Apply --ignore.
    if !ignore.is_empty() {
        all_findings.retain(|f| !ignore.iter().any(|i| i == &f.rule));
    }

    // Apply --min-severity.
    let min = min_severity.map(|m| m.into_severity());
    if let Some(min) = min {
        all_findings.retain(|f| f.severity >= min);
    }

    let started = Instant::now();
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
            // Print findings (with optional reveal animation) and
            // then the summary footer.
            text::format_findings(&all_findings);
            text::print_summary_footer(&all_findings, started.elapsed());
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
