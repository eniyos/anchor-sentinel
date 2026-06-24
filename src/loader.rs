//! Project loader: walk the supplied directory, locate the IDL files and
//! the Rust source for the Anchor programs, and hand them to the parsers.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config;
use crate::idl::{self, ir::ProgramIr};

/// What the loader found in a project directory.
pub struct LoadedProject {
    /// Project root. Kept for future diagnostics (relative error
    /// messages, `--include` patterns). Currently unused after `load()`.
    #[allow(dead_code)]
    pub root: PathBuf,
    pub idl_files: Vec<PathBuf>,
    pub programs: Vec<PathBuf>,
}

impl LoadedProject {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            idl_files: Vec::new(),
            programs: Vec::new(),
        }
    }
}

/// Discover IDL files and program `lib.rs` paths under `project`,
/// excluding any paths matching `exclude_patterns`.
pub fn load(project: &Path, exclude_patterns: &[String]) -> Result<LoadedProject> {
    use walkdir::WalkDir;

    let mut loaded = LoadedProject::new(project.to_path_buf());

    if let Ok(idls) = idl::discover_idl_files(project) {
        loaded.idl_files = idls
            .into_iter()
            .filter(|p| !config::is_excluded(p, exclude_patterns))
            .collect();
    }

    let programs_dir = project.join("programs");
    if programs_dir.is_dir() {
        for entry in WalkDir::new(&programs_dir)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.file_name().and_then(|s| s.to_str()) == Some("lib.rs")
                && !config::is_excluded(p, exclude_patterns)
            {
                loaded.programs.push(p.to_path_buf());
            }
        }
    }

    Ok(loaded)
}

/// Parse every IDL file under the project into a unified `ProgramIr`.
pub fn parse_idls(loaded: &LoadedProject) -> Result<Vec<ProgramIr>> {
    let mut out = Vec::new();
    for idl in &loaded.idl_files {
        out.push(idl::load_idl(idl)?);
    }
    Ok(out)
}
