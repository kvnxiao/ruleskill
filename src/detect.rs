use anyhow::{Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Target {
    Auto,
    Codex,
    Claude,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Harness {
    Codex,
    Claude,
}

impl Harness {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub(crate) fn skill_dir(self, repo_root: &Utf8Path, skill_name: &str) -> Utf8PathBuf {
        match self {
            Self::Codex => repo_root.join(".agents").join("skills").join(skill_name),
            Self::Claude => repo_root.join(".claude").join("skills").join(skill_name),
        }
    }

    pub(crate) fn rule_file(self, repo_root: &Utf8Path, skill_name: &str) -> Option<Utf8PathBuf> {
        match self {
            Self::Codex => None,
            Self::Claude => Some(
                repo_root
                    .join(".claude")
                    .join("rules")
                    .join(format!("{skill_name}.md")),
            ),
        }
    }
}

pub(crate) fn resolve_target(target: Target, repo_root: &Utf8Path) -> Result<Vec<Harness>> {
    match target {
        Target::Auto => {
            let detected = detect_harnesses(repo_root);
            if detected.is_empty() {
                bail!(
                    "no supported harness detected in {repo_root}. Pass --target codex, --target claude, or --target all"
                );
            }
            Ok(detected)
        }
        Target::Codex => Ok(vec![Harness::Codex]),
        Target::Claude => Ok(vec![Harness::Claude]),
        Target::All => Ok(vec![Harness::Codex, Harness::Claude]),
    }
}

fn detect_harnesses(repo_root: &Utf8Path) -> Vec<Harness> {
    let mut harnesses = Vec::new();

    if repo_root.join(".agents").exists()
        || repo_root.join("AGENTS.md").exists()
        || repo_root.join(".codex").exists()
    {
        harnesses.push(Harness::Codex);
    }

    if repo_root.join(".claude").exists() || repo_root.join("CLAUDE.md").exists() {
        harnesses.push(Harness::Claude);
    }

    harnesses
}
