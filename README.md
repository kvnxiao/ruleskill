# ruleskill

`ruleskill` installs agent skills generated from Markdown rule packs.

Use it to keep reusable engineering rules in one catalog, then render them into harness-specific skill folders for Codex or Claude Code. The source of truth for rules is in `rules/`; generated files are written under `.agents/skills/` or `.claude/skills/` in the directory that runs `ruleskill install <rule-pack>`.

## Usage

Install `ruleskill` using `cargo install --path .`.

List available skills:

```sh
ruleskill list
```

Validate the catalog:

```sh
ruleskill validate
```

Install a skill (harness target is auto-detected, unless `--target` is specified):

```sh
ruleskill install rust # auto-detect harness target
ruleskill install github-actions --target claude # specify harness target
ruleskill install rust --target all # install for all supported harness targets
```

Use `--dry-run` to preview writes. `--force` is accepted for compatibility and currently does not change overwrite behavior.

## Development

Useful checks:

```sh
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
paths = "**/*.ext,**/other/**" # optional

[[rules]]
title = "Rule title"
file = "rule-file.md"
when = "Read when this rule applies."
```

The optional `paths` field is a comma-separated list of glob patterns rendered into the generated skill frontmatter. Harnesses that support path-based activation (Claude Code) auto-load the skill when files matching the patterns are read or edited; other harnesses ignore the field.

Keep referenced files inside the skill folder. Avoid duplicate Markdown basenames within a single skill because generated references are flattened by filename.

After editing `rules/` or `templates/`, run:

```sh
ruleskill validate
```
