mod catalog;
mod detect;
mod fs;
mod render;

use std::env;
use std::fs as std_fs;

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
        skill_name: String,
        #[arg(long, value_enum, default_value_t = Target::Auto)]
        target: Target,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
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
        } => install(&skill_name, target, dry_run, force),
    }
}

fn list() -> Result<()> {
    let catalog = Catalog::load_default()?;
    for skill in catalog.skills() {
        println!("{}", skill.display_name());
    }
    Ok(())
}

fn validate() -> Result<()> {
    let catalog = Catalog::load_default()?;
    catalog.validate()?;
    println!(
        "validated {} skill(s) in {}",
        catalog.skills().len(),
        catalog.root()
    );
    Ok(())
}

fn install(skill_name: &str, target: Target, dry_run: bool, force: bool) -> Result<()> {
    let catalog = Catalog::load_default()?;
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

    for harness in harnesses {
        let skill_dir = harness.skill_dir(&repo_root, skill_name);
        println!(
            "{} {}",
            if dry_run {
                "would install"
            } else {
                "installing"
            },
            harness.name()
        );

        let report = write_generated(&skill_dir.join("SKILL.md"), &skill_md, mode, force)?;
        print_report(&report);

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
            print_report(&report);
        }
    }

    Ok(())
}

fn print_report(report: &crate::fs::WriteReport) {
    if report.dry_run {
        println!("would write {}", report.path);
    } else {
        println!("wrote {}", report.path);
    }
}

fn current_dir_utf8() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(env::current_dir()?)
        .map_err(|path| anyhow!("current directory is not valid UTF-8: {}", path.display()))
}
