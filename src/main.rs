mod catalog;
mod detect;
mod fs;
mod render;

use std::collections::BTreeSet;
use std::env;
use std::fs as std_fs;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand};

use crate::catalog::{Catalog, Skill, RULE_TEMPLATE, SKILL_TEMPLATE};
use crate::detect::{resolve_target, Target};
use crate::fs::{prune_empty_dir, remove_generated, write_generated, RemoveKind, WriteMode};
use crate::render::{generated_reference, render_template};

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
            help = "Accepted for compatibility; installs overwrite by default"
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
            force,
        } => install(skill_name.as_deref(), target, dry_run, force),
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

fn install(skill_name: Option<&str>, target: Target, dry_run: bool, force: bool) -> Result<()> {
    let catalog = Catalog::load_default()?;
    let Some(skill_name) = skill_name else {
        return Err(missing_skill_error(
            &catalog,
            "ruleskill install <SKILL_NAME>",
        ));
    };
    let skill = catalog.find_validated_skill(skill_name)?;
    let resolved = skill.resolve()?;
    let skill_md = render_template(SKILL_TEMPLATE, catalog.template(), &resolved.render)?;
    let rule_md = resolved
        .rule_file
        .as_ref()
        .map(|rule_file| render_template(RULE_TEMPLATE, catalog.rule_template(), rule_file))
        .transpose()?;
    let repo_root = current_dir_utf8()?;
    let harnesses = resolve_target(target, &repo_root)?;
    let mode = if dry_run {
        WriteMode::DryRun
    } else {
        WriteMode::Write
    };
    let mut stdout = io::stdout().lock();

    for harness in harnesses {
        let skill_dir = harness.skill_dir(&repo_root, &resolved.render.output_name);
        writeln!(
            stdout,
            "{} {}",
            if dry_run {
                "would install"
            } else {
                "installing"
            },
            harness.name()
        )
        .context("failed to write command output")?;

        let report = write_generated(&skill_dir.join("SKILL.md"), &skill_md, mode, force)?;
        print_report(&report, &mut stdout)?;

        for reference in &resolved.references {
            let source = std_fs::read_to_string(&reference.source)
                .with_context(|| format!("failed to read {}", reference.source))?;
            let content = generated_reference(&source);
            let report = write_generated(
                &skill_dir.join("references").join(&reference.reference_file),
                &content,
                mode,
                force,
            )?;
            print_report(&report, &mut stdout)?;
        }

        if let Some(path) = harness.rule_file(&repo_root, &resolved.render.output_name) {
            if let Some(content) = rule_md.as_deref() {
                let report = write_generated(&path, content, mode, force)?;
                print_report(&report, &mut stdout)?;
            } else if path.is_dir() {
                return Err(anyhow!(
                    "stale rule pointer path is a directory; remove or rename it before installing: {path}"
                ));
            } else if let Some(report) = remove_generated(&path, mode)? {
                print_removal(&report, &mut stdout)?;
                if let Some(parent) = path.parent() {
                    if let Some(report) =
                        prune_empty_dir(parent, std::slice::from_ref(&path), mode)?
                    {
                        print_removal(&report, &mut stdout)?;
                    }
                }
            }
        }
    }

    Ok(())
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
        for report in &reports {
            print_removal(report, &mut stdout)?;
        }

        let removed: Vec<Utf8PathBuf> = reports.iter().map(|report| report.path.clone()).collect();
        let parents: BTreeSet<&Utf8Path> =
            removed.iter().filter_map(|path| path.parent()).collect();
        for parent in parents {
            if let Some(report) = prune_empty_dir(parent, &removed, mode)? {
                print_removal(&report, &mut stdout)?;
            }
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
    Utf8PathBuf::from_path_buf(env::current_dir()?)
        .map_err(|path| anyhow!("current directory is not valid UTF-8: {}", path.display()))
}
