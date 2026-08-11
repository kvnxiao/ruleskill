use std::fs;

use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteMode {
    Write,
    DryRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WriteReport {
    pub(crate) path: Utf8PathBuf,
    pub(crate) dry_run: bool,
}

pub(crate) fn write_generated(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoveKind {
    Removed,
    Pruned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoveReport {
    pub(crate) path: Utf8PathBuf,
    pub(crate) kind: RemoveKind,
    pub(crate) dry_run: bool,
}

pub(crate) fn remove_generated(path: &Utf8Path, mode: WriteMode) -> Result<Option<RemoveReport>> {
    let is_dir = path.is_dir();
    if !is_dir && !path.exists() {
        return Ok(None);
    }

    if mode == WriteMode::Write {
        let outcome = if is_dir {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        outcome.with_context(|| format!("failed to remove {path}"))?;
    }

    Ok(Some(RemoveReport {
        path: path.to_path_buf(),
        kind: RemoveKind::Removed,
        dry_run: mode == WriteMode::DryRun,
    }))
}

/// Entries listed in `ignoring` are treated as already gone, so a dry run reports the same
/// prune that a real run performs.
pub(crate) fn prune_empty_dir(
    path: &Utf8Path,
    ignoring: &[Utf8PathBuf],
    mode: WriteMode,
) -> Result<Option<RemoveReport>> {
    if !path.is_dir() {
        return Ok(None);
    }

    for entry in fs::read_dir(path).with_context(|| format!("failed to read {path}"))? {
        let entry = entry.with_context(|| format!("failed to read {path}"))?;
        let entry_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|entry| anyhow!("path is not valid UTF-8: {}", entry.display()))?;
        if !ignoring.contains(&entry_path) {
            return Ok(None);
        }
    }

    if mode == WriteMode::Write {
        fs::remove_dir(path).with_context(|| format!("failed to remove {path}"))?;
    }

    Ok(Some(RemoveReport {
        path: path.to_path_buf(),
        kind: RemoveKind::Pruned,
        dry_run: mode == WriteMode::DryRun,
    }))
}
