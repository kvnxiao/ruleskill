use std::fs;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

use crate::render::GENERATED_MARKER;

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
    force: bool,
) -> Result<WriteReport> {
    ensure_overwrite_allowed(path, force)?;

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

fn ensure_overwrite_allowed(path: &Utf8Path, force: bool) -> Result<()> {
    if !path.exists() || force {
        return Ok(());
    }

    let existing =
        fs::read(path).with_context(|| format!("failed to read existing destination {path}"))?;
    if existing
        .windows(GENERATED_MARKER.len())
        .any(|window| window == GENERATED_MARKER.as_bytes())
    {
        Ok(())
    } else {
        bail!("refusing to overwrite non-generated file {path}; pass --force to overwrite it");
    }
}
