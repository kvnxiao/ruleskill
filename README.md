# ruleskill

`ruleskill` installs agent skills generated from Markdown rule packs.

Use it to keep reusable engineering rules in one catalog, then render them into tool-specific skill folders for Codex or Claude. The source of truth is `rules/`; generated files are written under `.agents/skills/` or `.claude/skills/`.

## Usage

List available skills:

```powershell
cargo run -- list
```

Validate the catalog:

```powershell
cargo run -- validate
```

Install a skill:

```powershell
cargo run -- install rust --target codex
cargo run -- install github-actions --target claude
cargo run -- install rust --target all
```

Use `--dry-run` to preview writes. Use `--force` only when replacing a non-generated destination.

## Development

Useful checks:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- validate
```

Run `cargo fmt` before re-checking formatting.

## Adding Rules

Add a new skill under `rules/<skill-name>/` with a `skill.toml` manifest and one or more Markdown rule files.

The manifest `name` must match the folder name and use kebab-case:

```toml
name = "my-skill"
title = "My Skill"
description = "Use when working on this kind of task."

[[rules]]
title = "Rule title"
file = "rule-file.md"
when = "Read when this rule applies."
```

Keep referenced files inside the skill folder. Avoid duplicate Markdown basenames within a single skill because generated references are flattened by filename.

After editing `rules/` or `templates/`, run:

```powershell
cargo run -- validate
```
