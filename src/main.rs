mod catalog;
mod detect;
mod fs;
mod render;

use std::env;
use std::fs as std_fs;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

use crate::catalog::Catalog;
use crate::detect::{resolve_target, Target};
use crate::fs::{write_generated, WriteMode};
use crate::render::{generated_reference, render_skill};

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
        let mut msg = String::from("no skill provided; available skills:");
        for skill in catalog.skills() {
            msg.push_str("\n  ");
            msg.push_str(skill.display_name());
        }
        msg.push_str("\nusage: ruleskill install <SKILL_NAME>");
        return Err(anyhow!(msg));
    };
    let skill = catalog.find_validated_skill(skill_name)?;
    let resolved = skill.resolve()?;
    let skill_md = render_skill(catalog.template(), &resolved.render)?;
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
    }

    Ok(())
}

fn print_report(report: &crate::fs::WriteReport, output: &mut impl Write) -> Result<()> {
    let result = if report.dry_run {
        writeln!(output, "would write {}", report.path)
    } else {
        writeln!(output, "wrote {}", report.path)
    };
    result.context("failed to write command output")
}

fn current_dir_utf8() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(env::current_dir()?)
        .map_err(|path| anyhow!("current directory is not valid UTF-8: {}", path.display()))
}
