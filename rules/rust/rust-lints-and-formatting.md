---
paths: **/*.{rs,toml}
description: "Clippy and rustfmt config; pedantic with justified allows, promoted restriction lints, clippy.toml, -D warnings in CI not source, and rustfmt.toml."
---

# Lints and Formatting

## Enable `pedantic` group-wide, then allow back with a reason

Turn on the whole `clippy::pedantic` group at a low priority, then `allow` the handful you reject — each with a one-line reason. `priority = -2` makes the group lose to individual lint lines, so your overrides win regardless of order.

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -2 }

# Allow only pedantic lints with a documented reason.
match_same_arms = "allow"
module_name_repetitions = "allow"
needless_continue = "allow" # an explicit continue can read better than an empty else
```

## Promote restriction lints to warnings

Several `clippy::restriction` lints catch real mistakes in library and tool code. Promote them:

```toml
print_stdout = "warn"
print_stderr = "warn"
dbg_macro = "warn"
exit = "warn"
get_unwrap = "warn"
rc_mutex = "warn"
iter_over_hash_type = "warn" # forces deterministic iteration order
```

`iter_over_hash_type` forces deterministic iteration order: iterating a `HashMap` in hash order is a nondeterminism bug waiting to happen.

## Configure risky std calls in `clippy.toml`

Use per-entry `reason` values in `disallowed-methods` to require an injectable abstraction for `std::fs`/`std::env` calls, which lets tests provide a fake filesystem. The reason appears in the lint message. Use `doc-valid-idents` to exempt domain words from `doc_markdown`.

```toml
# clippy.toml
disallowed-methods = [
    { path = "std::env::var", reason = "use System::env_var so tests can inject env" },
    { path = "std::fs::read_to_string", reason = "use System::read_to_string" },
]

doc-valid-idents = ["NumPy", "PyCharm", "SQLAlchemy"]
```

## Justify every `allow`

Group `#![allow(...)]` by reason and annotate each entry. Prefer `#[expect(...)]` over `#[allow(...)]` where the toolchain supports it — an `expect` that stops firing is itself a warning, so stale suppressions produce a warning.

```rust
#![allow(
    // Clippy issue: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
    // Integer serialization and deserialization require these casts.
    clippy::cast_possible_truncation,
)]
```

## Keep `-D warnings` in CI, not in source

Enforce strict linting in CI so a new compiler or Clippy lint fails *your* build, not every downstream user who compiles your published crate.

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do **not** put `#![deny(warnings)]` in source: a future toolchain that adds a lint would break consumers building your crate through no fault of theirs.

## Commit a minimal `rustfmt.toml`

Pin the formatting so it doesn't drift across contributors and toolchains. Bump `edition`/`style_edition` alongside the crate edition.

```toml
edition = "2024"
style_edition = "2024"
# Optional width settings:
# max_width = 79
# use_small_heuristics = "max"
```
