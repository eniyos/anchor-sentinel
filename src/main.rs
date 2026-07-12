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

struct ScanTimings {
    load: std::time::Duration,
    parse_idls: std::time::Duration,
    ast_hints: std::time::Duration,
    run_rules: std::time::Duration,
    total: std::time::Duration,
}

fn main() -> ExitCode {
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
            verbose,
        } => cmd_scan(&path, format, strict, &ignore, min_severity, verbose),
        Command::Rules => {
            cmd_rules();
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            println!("sentinel {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Command::Explain { rule_id } => {
            cmd_explain(&rule_id);
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
    verbose: bool,
) -> Result<ExitCode> {
    let project = std::path::Path::new(path);
    if !project.exists() {
        anyhow::bail!("project path does not exist: {}", project.display());
    }

    let cfg = config::Config::load(project);

    let mut all_ignore: Vec<String> = cfg.ignore_rules().to_vec();
    for i in ignore {
        if !all_ignore.contains(i) {
            all_ignore.push(i.clone());
        }
    }

    let effective_min_severity = min_severity.or_else(|| {
        cfg.min_severity()
            .and_then(|s| match s.to_lowercase().as_str() {
                "info" => Some(cli::MinSeverity::Info),
                "low" => Some(cli::MinSeverity::Low),
                "medium" => Some(cli::MinSeverity::Medium),
                "high" => Some(cli::MinSeverity::High),
                "critical" => Some(cli::MinSeverity::Critical),
                _ => None,
            })
    });

    if matches!(format, cli::OutputFormat::Text) {
        text::print_hero(&project.display().to_string(), &cfg);
    }

    let t_total_start = Instant::now();

    let t_load_start = Instant::now();
    if verbose {
        eprintln!("[*] Loading project...");
    }
    let loaded = loader::load(project, cfg.exclude_paths()).context("loading project")?;
    if loaded.idl_files.is_empty() {
        anyhow::bail!(
            "no IDL files found. Run `anchor build` inside the project first \
             so that target/idl/*.json is populated."
        );
    }
    let t_load = t_load_start.elapsed();
    if verbose {
        eprintln!(
            "[+] Loaded {} IDL file(s), {} program(s) in {:?}",
            loaded.idl_files.len(),
            loaded.programs.len(),
            t_load
        );
    }

    let t_parse_start = Instant::now();
    if verbose {
        eprintln!("[*] Parsing IDLs...");
    }
    let programs = loader::parse_idls(&loaded).context("parsing IDLs")?;
    let t_parse = t_parse_start.elapsed();
    if verbose {
        let total_instrs: usize = programs.iter().map(|p| p.instructions.len()).sum();
        eprintln!(
            "[+] Parsed {} program(s) with {} instruction(s) in {:?}",
            programs.len(),
            total_instrs,
            t_parse
        );
    }

    let t_ast_start = Instant::now();
    if verbose {
        eprintln!("[*] Analyzing source code...");
    }
    let ast_hints = ast::collect_hints(&loaded.programs);
    let t_ast = t_ast_start.elapsed();
    if verbose {
        eprintln!("[+] AST analysis complete in {:?}", t_ast);
    }

    let t_rules_start = Instant::now();
    if verbose {
        eprintln!("[*] Running security rules...");
    }
    let mut all_findings = Vec::new();
    let rule_count = engine::registry::list_rule_ids().len();
    for (i, ir) in programs.iter().enumerate() {
        if verbose {
            eprintln!("[*] Checking program {} of {}...", i + 1, programs.len());
        }
        let ctx = AnalysisContext {
            ir: ir.clone(),
            ast_hints: ast_hints.clone(),
        };
        all_findings.extend(engine::run_all_rules(&ctx)?);
    }
    let t_rules = t_rules_start.elapsed();
    let t_total = t_total_start.elapsed();
    if verbose {
        eprintln!(
            "[+] Executed {} rule(s) in {:?} (total: {:?})",
            rule_count, t_rules, t_total
        );
    }

    let timings = ScanTimings {
        load: t_load,
        parse_idls: t_parse,
        ast_hints: t_ast,
        run_rules: t_rules,
        total: t_total,
    };

    if !all_ignore.is_empty() {
        all_findings.retain(|f| !all_ignore.iter().any(|i| i == &f.rule));
    }

    let min = effective_min_severity.map(|m| m.into_severity());
    if let Some(min) = min {
        all_findings.retain(|f| f.severity >= min);
    }

    let programs_count = loaded.programs.len();
    let instructions_count: usize = programs.iter().map(|p| p.instructions.len()).sum();

    match format {
        cli::OutputFormat::Sarif => {
            println!("{}", report::sarif::render(&all_findings));
        }
        cli::OutputFormat::Json => {
            println!("{}", report::json::render(&all_findings));
        }
        cli::OutputFormat::Text => {
            let scan_report = text::ScanReport {
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
            text::print_pipeline(&scan_report.timings);
            text::print_security_overview(&all_findings);
            text::print_findings(&all_findings);
            text::print_statistics(&scan_report);
            text::print_verdict(&all_findings);
        }
    }

    // `--min-severity` controls *display* filtering (done above) and the
    // exit code: any finding at or above the threshold → 1.
    // `--strict` controls the *exit code*: any non-Info finding → 1.
    // If neither flag is set, exit 0 always.
    let has_blocking = if strict {
        all_findings.iter().any(|f| f.severity > Severity::Info)
    } else if effective_min_severity.is_some() {
        !all_findings.is_empty()
    } else {
        false
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

fn cmd_explain(rule_id: &str) {
    if let Some(explain) = report::explain::get_explanation(rule_id) {
        println!();
        println!("{}", "=".repeat(70));
        println!("⚓ {}", explain.title);
        println!("{}", "=".repeat(70));
        println!();
        println!("ID:        {}", explain.id);
        println!("Severity:  {}", explain.severity);
        println!();
        println!("{}", "-".repeat(70));
        println!("WHAT");
        println!("{}", "-".repeat(70));
        println!("{}", explain.what);
        println!();
        println!("{}", "-".repeat(70));
        println!("WHY THIS IS DANGEROUS");
        println!("{}", "-".repeat(70));
        println!("{}", explain.why);
        println!();
        println!("{}", "-".repeat(70));
        println!("VULNERABLE EXAMPLE");
        println!("{}", "-".repeat(70));
        for line in explain.vulnerable_example.lines() {
            println!("{}", line);
        }
        println!();
        println!("{}", "-".repeat(70));
        println!("SAFE EXAMPLE");
        println!("{}", "-".repeat(70));
        for line in explain.safe_example.lines() {
            println!("{}", line);
        }
        println!();
        if let Some(ref_url) = explain.exploit_ref {
            println!("{}", "-".repeat(70));
            println!("EXPLOIT REFERENCE");
            println!("{}", "-".repeat(70));
            println!("{}", ref_url);
        }
        if let Some(pat) = explain.detection_pattern {
            println!("{}", "-".repeat(70));
            println!("DETECTION PATTERN");
            println!("{}", "-".repeat(70));
            for line in pat.lines() {
                println!("{}", line);
            }
        }
        if let Some(see_also) = explain.see_also {
            println!();
            println!("{}", "-".repeat(70));
            println!("SEE ALSO");
            println!("{}", "-".repeat(70));
            for rule in see_also {
                println!("  • `sentinel explain {}`", rule);
            }
        }
        println!();
        println!("{}", "=".repeat(70));
        println!();
        println!("Learn more: Run `sentinel scan .` to detect this pattern in your code.");
        println!("See also: `sentinel rules` for all security rules.");
    } else {
        eprintln!("error: unknown rule '{}'", rule_id);
        eprintln!("Run `sentinel rules` to see all available rules.");
    }
}
