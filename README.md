# ruleskill

`ruleskill` installs agent skills generated from Markdown rule packs.

Use it to keep reusable engineering rules in one catalog, then render them into harness-specific skill folders for Codex or Claude Code. The source of truth for rules is in `rules/`; generated files are written under `.agents/skills/` or `.claude/skills/` in the directory that runs `ruleskill install <rule-pack>`. Rule packs that set `paths` also get a path-scoped `.claude/rules/` pointer for the Claude target.

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

Uninstall a skill (removes the generated skill folder and, for the Claude target, the `.claude/rules/` pointer):

```sh
ruleskill uninstall rust # auto-detect harness target
ruleskill uninstall rust --target claude --dry-run # preview the removals
ruleskill uninstall --all # remove every rule pack in the catalog
```

`--all` covers the packs in the catalog, so it never deletes skills that `ruleskill` did not install. Output installed from a pack that has since left the catalog has to be deleted by hand. Empty `skills/` and `rules/` folders are pruned; `.claude/` and `.agents/` are kept so harness auto-detection and other settings survive.

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

The optional `paths` field is a comma-separated list of glob patterns. For the Claude target it generates a path-scoped rule at `.claude/rules/<rule-pack>-rules.md` that points back at the skill, so Claude loads the pointer when it reads a matching file. Other harnesses ignore the field.

`paths` is deliberately kept out of skill frontmatter. Claude Code accepts `paths` on a skill, but it then gates the whole skill: the skill is absent from the skill listing and `/<rule-pack>-rules` fails with `Unknown command` until a matching file is read in that session. Splitting the two artifacts keeps the skill invocable from the first turn and still auto-attaches when matching files are touched.

Keep referenced files inside the skill folder. Avoid duplicate Markdown basenames within a single skill because generated references are flattened by filename.

After editing `rules/` or `templates/`, run:

```sh
ruleskill validate
```
