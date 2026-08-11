# AGENTS.md

Guidance for agents working in this repository.

## Skills

This repo installs its own rule packs as `*-rules` skills (e.g. `rust-rules`, `github-actions-rules`). Before writing or reviewing code in a domain covered by one, invoke the matching `*-rules` skill first — match on the skill's own description — then apply its reference rules.

## Project Shape

`ruleskill` is a Rust CLI that installs agent skills generated from Markdown rule packs. The catalog lives in `rules/`, the MiniJinja templates live in `templates/skill.md.j2` and `templates/rule.md.j2`, CLI/source code lives in `src/`, and integration tests live in `tests/cli.rs`.

Install the CLI locally with `cargo install --path .`. The installed CLI commands are:

- `ruleskill list`
- `ruleskill validate`
- `ruleskill install <rule-pack> [--target codex|claude|all|auto] [--dry-run] [--force]`

During development, the same commands can be run with `cargo run -- <command>`.

Current input rule packs are `rust`, `github-actions`, and `solidjs`. Output skill folders are named from the manifest name with a `-rules` suffix unless the name already ends in `-rules`, so these install as `rust-rules`, `github-actions-rules`, and `solidjs-rules`.

The default catalog root is the crate root from `CARGO_MANIFEST_DIR`. `RULESKILL_CATALOG_DIR` can point the CLI at a fixture catalog instead. Installs write into the current working directory, not the catalog directory.

The harness target is auto-detected unless `--target` is specified. Auto-detection treats `.agents`, `.codex`, or `AGENTS.md` as Codex markers, and `.claude` or `CLAUDE.md` as Claude markers.

## Checks

Run the narrowest useful checks for the change, and prefer these before handing work back:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo run -- validate` when editing `rules/` or `templates/`

Use `cargo fmt` to fix formatting before re-running `cargo fmt --check`.

## Catalog Rules

Each rule pack lives under `rules/<rule-pack>/skill.toml`; the catalog loader scans `rules/*/skill.toml`. The folder name and manifest `name` must match and must be kebab-case.

Every `skill.toml` needs `name`, `title`, `description`, and at least one `[[rules]]` entry. Each rule needs `title`, `file`, and `when`. An optional top-level `paths` field (comma-separated globs) generates a path-scoped Claude rule file instead of skill frontmatter; it must hold at least one non-empty pattern when set.

Rule file paths must be relative paths that stay inside their rule pack folder. Duplicate rule file paths are invalid. Generated reference files flatten to the source filename, so two rule files with the same basename in one rule pack will collide even if they are in different subdirectories.

## Generated Output

Generated `SKILL.md` files are rendered from `templates/skill.md.j2` and start with YAML frontmatter carrying only `name` and `description`. Generated reference files preserve the source Markdown rule files verbatim.

When a rule pack sets `paths`, the Claude target also writes `.claude/rules/<rule-pack>-rules.md` from `templates/rule.md.j2`: a short pointer whose own `paths` frontmatter makes Claude load it on reading a matching file, telling it to invoke the skill. Never put `paths` in skill frontmatter — Claude Code accepts the field but then hides the skill from the skill listing and rejects `/<rule-pack>-rules` with `Unknown command` until a matching file is read in that session. The Codex target has no rule-file equivalent and gets skills only.

Do not hand-edit generated output under `.agents/skills/`, `.claude/skills/`, or `.claude/rules/`. Edit `rules/` or `templates/`, then regenerate with the CLI. Installs overwrite existing destination files by default; `--force` is accepted for compatibility and currently does not change write behavior.

## Rust Conventions

This is a Rust 2021 project. Keep CLI errors actionable with `anyhow::Context`, use `camino` for UTF-8 paths where the existing code does, and preserve the existing path-safety checks for catalog files.

When changing behavior, add or update integration tests in `tests/cli.rs`. The tests use `assert_cmd`, `assert_fs`, `predicates`, and temporary fixture catalogs.

Avoid adding dependencies unless they materially simplify the implementation. If dependencies change, keep `Cargo.lock` updated.
