use std::fs;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    Write,
    DryRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReport {
    pub path: Utf8PathBuf,
    pub dry_run: bool,
}

pub fn write_generated(
    path: &Utf8Path,
    content: &str,
    mode: WriteMode,
    _force: bool,
) -> Result<WriteReport> {
    if mode == WriteMode::Write {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent}"))?;
        }
        fs::write(path, content).with_context(|| format!("failed to write {path}"))?;
    }

    Ok(WriteReport {
        path: path.to_path_buf(),
        dry_run: mode == WriteMode::DryRun,
    })
}
