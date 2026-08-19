mod catalog;
mod detect;
mod fs;
mod render;

use std::collections::BTreeSet;
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand};
use fs_err as fs_io;
use serde::Deserialize;

use crate::catalog::{Catalog, RULE_TEMPLATE, SKILL_TEMPLATE, Skill, is_output_skill_name};
use crate::detect::{Harness, Target, resolve_target};
use crate::fs::{
    RemoveKind, RemoveReport, WriteMode, prune_empty_dir, remove_generated, write_generated,
};
use crate::render::render_template;

const PRUNE_STATE_FILE: &str = ".ruleskill-prune.toml";

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PruneState {
    codex: BTreeSet<String>,
    claude: BTreeSet<String>,
}

impl PruneState {
    fn load(repo_root: &Utf8Path) -> Result<Self> {
        let path = repo_root.join(PRUNE_STATE_FILE);
        let content = match fs_io::read_to_string(&path) {
            Ok(content) => content,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err.into()),
        };
        let state = toml::from_str::<Self>(&content)
            .with_context(|| format!("failed to parse prune state at {path}"))?;
        for output_name in state.codex.iter().chain(&state.claude) {
            if !is_output_skill_name(output_name) {
                return Err(anyhow!(
                    "prune state contains invalid output name '{output_name}': {path}"
                ));
            }
        }
        Ok(state)
    }

    fn outputs(&self, harness: Harness) -> &BTreeSet<String> {
        match harness {
            Harness::Codex => &self.codex,
            Harness::Claude => &self.claude,
        }
    }

    fn outputs_mut(&mut self, harness: Harness) -> &mut BTreeSet<String> {
        match harness {
            Harness::Codex => &mut self.codex,
            Harness::Claude => &mut self.claude,
        }
    }

    fn render(&self) -> String {
        fn render_names(names: &BTreeSet<String>) -> String {
            names
                .iter()
                .map(|name| format!("\"{name}\""))
                .collect::<Vec<_>>()
                .join(", ")
        }

        format!(
            "codex = [{}]\nclaude = [{}]\n",
            render_names(&self.codex),
            render_names(&self.claude)
        )
    }
}

#[derive(Debug, Parser)]
#[command(name = "ruleskill")]
#[command(about = "Install agent skills generated from Markdown rule files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Validate,
    Install {
        skill_name: Option<String>,
        #[arg(long, value_enum, default_value_t = Target::Auto)]
        target: Target,
        #[arg(long)]
        dry_run: bool,
        #[arg(
            long,
            conflicts_with = "skill_name",
            help = "Install every rule pack in the catalog"
        )]
        all: bool,
        #[arg(
            long,
            requires = "all",
            help = "Remove previously recorded outputs absent from the catalog"
        )]
        prune: bool,
        #[arg(
            long,
            help = "Accepted for compatibility; installs replace generated skill folders by default"
        )]
        force: bool,
    },
    Uninstall {
        skill_name: Option<String>,
        #[arg(long, value_enum, default_value_t = Target::Auto)]
        target: Target,
        #[arg(long)]
        dry_run: bool,
        #[arg(
            long,
            conflicts_with = "skill_name",
            help = "Remove every rule pack in the catalog"
        )]
        all: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let _ = writeln!(io::stderr().lock(), "error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::List => list(),
        Command::Validate => validate(),
        Command::Install {
            skill_name,
            target,
            dry_run,
            all,
            prune,
            ..
        } => install(skill_name.as_deref(), target, dry_run, all, prune),
        Command::Uninstall {
            skill_name,
            target,
            dry_run,
            all,
        } => uninstall(skill_name.as_deref(), target, dry_run, all),
    }
}

fn list() -> Result<()> {
    let catalog = Catalog::load_default()?;
    let mut stdout = io::stdout().lock();
    for skill in catalog.skills() {
        writeln!(stdout, "{}", skill.display_name()).context("failed to write command output")?;
    }
    Ok(())
}

fn validate() -> Result<()> {
    let catalog = Catalog::load_default()?;
    catalog.validate()?;
    writeln!(
        io::stdout().lock(),
        "validated {} skill(s) in {}",
        catalog.skills().len(),
        catalog.root()
    )
    .context("failed to write command output")?;
    Ok(())
}

fn install(
    skill_name: Option<&str>,
    target: Target,
    dry_run: bool,
    all: bool,
    prune: bool,
) -> Result<()> {
    let catalog = Catalog::load_default()?;
    let skills = if all {
        catalog.validate()?;
        catalog.skills()
    } else {
        let Some(skill_name) = skill_name else {
            return Err(missing_skill_error(
                &catalog,
                "ruleskill install <SKILL_NAME> | ruleskill install --all",
            ));
        };
        std::slice::from_ref(catalog.find_validated_skill(skill_name)?)
    };
    let repo_root = current_dir_utf8()?;
    let harnesses = resolve_target(target, &repo_root)?;
    let mode = if dry_run {
        WriteMode::DryRun
    } else {
        WriteMode::Write
    };
    let mut stdout = io::stdout().lock();

    let mut prune_plan = if prune {
        Some((
            skills
                .iter()
                .map(Skill::output_name)
                .collect::<Result<BTreeSet<_>>>()?,
            PruneState::load(&repo_root)?,
        ))
    } else {
        None
    };
    if let Some((output_names, prune_state)) = &prune_plan {
        for &harness in &harnesses {
            prune_stale_generated(
                &repo_root,
                harness,
                prune_state.outputs(harness),
                output_names,
                mode,
                &mut stdout,
            )?;
        }
    }

    for skill in skills {
        install_skill(&catalog, skill, &repo_root, &harnesses, mode, &mut stdout)?;
    }

    if let Some((output_names, prune_state)) = &mut prune_plan {
        for &harness in &harnesses {
            prune_state.outputs_mut(harness).clone_from(output_names);
        }
        let report = write_generated(
            &repo_root.join(PRUNE_STATE_FILE),
            &prune_state.render(),
            mode,
        )?;
        print_report(&report, &mut stdout)?;
    }

    Ok(())
}

fn install_skill(
    catalog: &Catalog,
    skill: &Skill,
    repo_root: &Utf8Path,
    harnesses: &[Harness],
    mode: WriteMode,
    output: &mut impl Write,
) -> Result<()> {
    let resolved = skill.resolve()?;
    let skill_md = render_template(SKILL_TEMPLATE, catalog.template(), &resolved.render)?;
    let rule_md = resolved
        .rule_file
        .as_ref()
        .map(|rule_file| render_template(RULE_TEMPLATE, catalog.rule_template(), rule_file))
        .transpose()?;
    let references = resolved
        .references
        .iter()
        .map(|reference| {
            Ok((
                reference.reference_file.as_str(),
                fs_io::read_to_string(&reference.source)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    for &harness in harnesses {
        let skill_dir = harness.skill_dir(repo_root, &resolved.render.output_name);
        writeln!(
            output,
            "{} {}",
            if mode == WriteMode::DryRun {
                "would install"
            } else {
                "installing"
            },
            harness.name()
        )
        .context("failed to write command output")?;

        if let Some(report) = remove_generated(&skill_dir, mode)? {
            print_removal(&report, output)?;
        }
        let report = write_generated(&skill_dir.join("SKILL.md"), &skill_md, mode)?;
        print_report(&report, output)?;

        for (reference_file, source) in &references {
            let report = write_generated(
                &skill_dir.join("references").join(reference_file),
                source,
                mode,
            )?;
            print_report(&report, output)?;
        }

        if let Some(path) = harness.rule_file(repo_root, &resolved.render.output_name) {
            if let Some(content) = rule_md.as_deref() {
                let report = write_generated(&path, content, mode)?;
                print_report(&report, output)?;
            } else if path.is_dir() {
                return Err(anyhow!(
                    "stale rule pointer path is a directory; remove or rename it before installing: {path}"
                ));
            } else if let Some(report) = remove_generated(&path, mode)? {
                print_removal(&report, output)?;
                if let Some(parent) = path.parent()
                    && let Some(report) =
                        prune_empty_dir(parent, std::slice::from_ref(&path), mode)?
                {
                    print_removal(&report, output)?;
                }
            }
        }
    }

    Ok(())
}

fn prune_stale_generated(
    repo_root: &Utf8Path,
    harness: Harness,
    installed_outputs: &BTreeSet<String>,
    output_names: &BTreeSet<String>,
    mode: WriteMode,
    output: &mut impl Write,
) -> Result<()> {
    let mut reports = Vec::new();
    for output_name in installed_outputs.difference(output_names) {
        let targets = [
            Some(harness.skill_dir(repo_root, output_name)),
            harness.rule_file(repo_root, output_name),
        ];
        for path in targets.into_iter().flatten() {
            reports.extend(remove_generated(&path, mode)?);
        }
    }

    print_removals_and_prune_parents(&reports, mode, output)
}

fn uninstall(skill_name: Option<&str>, target: Target, dry_run: bool, all: bool) -> Result<()> {
    let catalog = Catalog::load_default()?;
    let output_names = if all {
        catalog
            .skills()
            .iter()
            .map(Skill::output_name)
            .collect::<Result<Vec<_>>>()?
    } else {
        let Some(skill_name) = skill_name else {
            return Err(missing_skill_error(
                &catalog,
                "ruleskill uninstall <SKILL_NAME> | ruleskill uninstall --all",
            ));
        };
        vec![catalog.find_skill(skill_name)?.output_name()?]
    };
    let repo_root = current_dir_utf8()?;
    let harnesses = resolve_target(target, &repo_root)?;
    let mode = if dry_run {
        WriteMode::DryRun
    } else {
        WriteMode::Write
    };
    let mut stdout = io::stdout().lock();

    for harness in harnesses {
        let mut reports = Vec::new();
        for output_name in &output_names {
            let targets = [
                Some(harness.skill_dir(&repo_root, output_name)),
                harness.rule_file(&repo_root, output_name),
            ];
            for path in targets.into_iter().flatten() {
                reports.extend(remove_generated(&path, mode)?);
            }
        }

        if reports.is_empty() {
            writeln!(stdout, "nothing to remove for {}", harness.name())
                .context("failed to write command output")?;
            continue;
        }

        writeln!(
            stdout,
            "{} {}",
            if dry_run {
                "would uninstall"
            } else {
                "uninstalling"
            },
            harness.name()
        )
        .context("failed to write command output")?;
        print_removals_and_prune_parents(&reports, mode, &mut stdout)?;
    }

    Ok(())
}

fn print_removals_and_prune_parents(
    reports: &[RemoveReport],
    mode: WriteMode,
    output: &mut impl Write,
) -> Result<()> {
    for report in reports {
        print_removal(report, output)?;
    }

    let removed = reports
        .iter()
        .map(|report| report.path.clone())
        .collect::<Vec<_>>();
    let parents = removed
        .iter()
        .filter_map(|path| path.parent())
        .collect::<BTreeSet<_>>();
    for parent in parents {
        if let Some(report) = prune_empty_dir(parent, &removed, mode)? {
            print_removal(&report, output)?;
        }
    }

    Ok(())
}

fn missing_skill_error(catalog: &Catalog, usage: &str) -> anyhow::Error {
    let mut msg = String::from("no skill provided; available skills:");
    for skill in catalog.skills() {
        msg.push_str("\n  ");
        msg.push_str(skill.display_name());
    }
    msg.push_str("\nusage: ");
    msg.push_str(usage);
    anyhow!(msg)
}

fn print_report(report: &crate::fs::WriteReport, output: &mut impl Write) -> Result<()> {
    let result = if report.dry_run {
        writeln!(output, "would write {}", report.path)
    } else {
        writeln!(output, "wrote {}", report.path)
    };
    result.context("failed to write command output")
}

fn print_removal(report: &crate::fs::RemoveReport, output: &mut impl Write) -> Result<()> {
    let action = match (report.kind, report.dry_run) {
        (RemoveKind::Removed, false) => "removed",
        (RemoveKind::Removed, true) => "would remove",
        (RemoveKind::Pruned, false) => "pruned",
        (RemoveKind::Pruned, true) => "would prune",
    };
    writeln!(output, "{action} {}", report.path).context("failed to write command output")
}

fn current_dir_utf8() -> Result<Utf8PathBuf> {
    let path = env::current_dir().context("failed to read current directory")?;
    Utf8PathBuf::from_path_buf(path)
        .map_err(|path| anyhow!("current directory is not valid UTF-8: {}", path.display()))
}
