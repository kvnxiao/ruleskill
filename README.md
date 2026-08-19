# ruleskill

`ruleskill` installs agent skills generated from Markdown rule packs.

Use it to keep reusable engineering rules in one catalog, then render them into harness-specific skill folders for Codex or Claude Code. The source of truth for rules is in `rules/`; generated files are written under `.agents/skills/` or `.claude/skills/` in the directory that runs `ruleskill install <rule-pack>`. Rule packs that set `paths` also get a path-scoped `.claude/rules/` pointer for the Claude target.

## Usage

Install `ruleskill` from this checkout:

```sh
just install
```

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
ruleskill install --all --target all # install every catalog pack for every target
ruleskill install --all --target all --prune # also remove obsolete rule-pack outputs
```

Use `--dry-run` to preview writes and removals. Installs replace the generated skill folder, so stale references are removed. `--force` is accepted for compatibility and currently does not change replacement behavior.

Uninstall a skill (removes the generated skill folder, plus the `.claude/rules/` pointer for the Claude target):

```sh
ruleskill uninstall rust # auto-detect harness target
ruleskill uninstall rust --target claude --dry-run # preview the removals
ruleskill uninstall --all # remove every rule pack in the catalog
```

`ruleskill uninstall --all` removes generated paths for every pack currently in the catalog; it does not scan other skill folders. If you drop a rule pack from the catalog, remove its output by hand or run `install --all --prune`. Uninstall prunes `skills/` and `rules/` once they are empty, but keeps `.claude/` and `.agents/` so harness auto-detection and your other settings survive.

With `install --all`, `--prune` removes only outputs recorded by an earlier pruned install and now absent from the catalog. `.ruleskill-prune.toml` records owned outputs separately for Codex and Claude, so other skills and rule pointers remain untouched.

## Development

`Cargo.toml`'s `package.rust-version` field declares the minimum supported Rust version, and CI reads the value from package metadata.

Run the complete check suite or a narrow check:

```sh
just check
just lint
just test
```

Run `just fmt` to format the workspace with nightly rustfmt. Run `just --list` to see every repository task.

`Cargo.lock` is committed. Dependabot proposes weekly Cargo and GitHub Actions updates, and CI audits dependency advisories, bans, and sources with `cargo-deny`.

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

After editing `rules/` or `templates/`, replace both checked-in harness outputs:

```sh
just reeject
```
