---
paths: **/*.{rs,toml}
description: "Required Rust lint baseline, scoped exceptions, conditional restrictions, nightly rustfmt settings, and shared local/CI tasks with stable Clippy."
---

# Lints and Formatting

## Install the complete lint baseline (Required)

When bootstrapping a Rust project, install the complete configuration below in `Cargo.toml`. For a
workspace, use `[workspace.lints.rust]` and `[workspace.lints.clippy]` instead, and set `[lints]
workspace = true` in every member, including a root package. Keep the baseline in one manifest
location; apply the same policy to application and library crates.

```toml
[lints.rust]
unsafe_code = "forbid"
unsafe_op_in_unsafe_fn = "deny"
missing_docs = "warn"

[lints.clippy]
all = { level = "warn", priority = -2 }
pedantic = { level = "warn", priority = -2 }
cargo = { level = "warn", priority = -2 }

multiple_crate_versions = "allow"
cast_precision_loss = "allow"

unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unreachable = "deny"
string_slice = "deny"
indexing_slicing = "deny"
get_unwrap = "deny"
unwrap_in_result = "deny"
panic_in_result_fn = "deny"
unchecked_time_subtraction = "deny"
todo = "warn"
unimplemented = "warn"

let_underscore_future = "deny"
let_underscore_must_use = "deny"
unused_result_ok = "deny"
map_err_ignore = "deny"

await_holding_lock = "deny"
await_holding_refcell_ref = "deny"
if_let_mutex = "deny"
large_futures = "deny"

mem_forget = "deny"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
unnecessary_safety_doc = "deny"
unnecessary_safety_comment = "deny"

float_cmp = "deny"
lossy_float_literal = "deny"
invalid_upcast_comparisons = "deny"

rc_mutex = "deny"
debug_assert_with_mut_call = "deny"
iter_not_returning_iterator = "deny"
expl_impl_clone_on_copy = "deny"
infallible_try_from = "deny"
dbg_macro = "deny"

print_stdout = "deny"
print_stderr = "deny"
exit = "deny"

cast_lossless = "deny"
cast_possible_truncation = "deny"
cast_possible_wrap = "deny"
cast_sign_loss = "deny"

allow_attributes = "deny"
allow_attributes_without_reason = "deny"

inefficient_to_string = "deny"
large_enum_variant = "warn"
large_stack_arrays = "deny"
needless_pass_by_value = "warn"
missing_errors_doc = "warn"
missing_panics_doc = "warn"
```

Use the negative group priority so individual settings override group membership. Select restriction
lints individually; do not enable `clippy::restriction` as a group. See
[Clippy lint configuration](https://doc.rust-lang.org/clippy/usage.html#lint-configuration).

Keep Cargo metadata checks enabled. Mark intentionally unpublished packages `publish = false`;
Clippy skips their publication metadata by default. Allow duplicate dependency versions and inspect
`cargo tree -d` when build size, compile times, or incompatible dependency types warrant it. See
[Cargo metadata checking](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/cargo/common_metadata.rs)
and
[dependency duplication](https://doc.rust-lang.org/cargo/reference/resolver.html#version-incompatibility-hazards).

## Configure test allowances (Required)

Commit this `clippy.toml` at the project or workspace root:

```toml
allow-expect-in-tests = true
allow-print-in-tests = true
```

Default test functions to returning `()` and use descriptive `expect` messages for fallible setup.
Keep shared helpers fallible or place test-only helpers in `#[cfg(test)]` modules. Apply the
allowances only in contexts Clippy recognizes as tests; a helper's location under `tests/` alone
does not establish that context. Keep the remaining restrictions active in tests, including
`unwrap_used` and `indexing_slicing`. See
[test configuration](https://doc.rust-lang.org/clippy/lint_configuration.html#allow-expect-in-tests).

## Limit exceptions to their approved scope (Required)

Apply the predefined exceptions below without further approval. Before introducing another exception
or weakening the baseline, obtain the user's approval. Use `#[expect(lint, reason = "...")]` on the
smallest applicable item and state the concrete contract that permits the operation. Remove stale
expectations when the lint stops firing. Keep `unfulfilled_lint_expectations` enabled and do not
substitute broad `allow` attributes.

| Condition                                                                                     | Permitted exception                                                                                                                                                                                                        |
| --------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A function implements CLI output or diagnostics                                               | Expect only the `print_stdout` or `print_stderr` lint that fires in that function.                                                                                                                                         |
| A build script emits Cargo directives or a function implements another stdout/stderr protocol | Scope the corresponding printing expectation to the protocol-emitting function.                                                                                                                                            |
| An invalid literal is rejected during forced compile-time evaluation                          | Keep rejection in a const evaluation context; request approval if a lint suppression is still needed.                                                                                                                      |
| The user approves unsafe implementation in a crate                                            | Change the applicable `unsafe_code` policy from `forbid` to `deny`, then scope expectations to the approved implementation and satisfy the [unsafe obligations](rust-api-design.md#unsafe-soundness-obligations-required). |

```rust
#[expect(clippy::print_stdout, reason = "CLI output lists the installed packages")]
fn print_packages(packages: &[String]) {
    for package in packages {
        println!("{package}");
    }
}
```

Return an exit status from the entry point rather than calling `std::process::exit`. Use fallible
APIs for runtime validation; an author-controlled value alone does not authorize a runtime panic.
Declaring a function `const fn` does not force its callers to evaluate it at compile time. Follow
the [constructor policy](rust-api-design.md#use-fallible-runtime-constructors-required).

When a workspace member needs a different unsafe policy, account for Cargo's lint inheritance:
member manifests cannot combine `workspace = true` with additional lint entries. Preserve all other
baseline settings in the approved configuration. A local expectation cannot override `forbid`. See
[Cargo lint inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table)
and
[Rust lint levels](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes).

## Enable project-specific restrictions only when needed (Conditional)

Enable the following restrictions only when their stated requirements apply. Set the corresponding
lint in the Clippy manifest table and document the requirement.

| Requirement                                                                            | Setting                        |
| -------------------------------------------------------------------------------------- | ------------------------------ |
| Integer-to-float conversions must preserve exact values                                | `cast_precision_loss = "deny"` |
| Floating-point comparisons against constants must use a domain-defined error tolerance | `float_cmp_const = "deny"`     |
| Iteration order affects reproducible output or other observable behavior               | `iter_over_hash_type = "deny"` |

For approximate numeric work, choose conversions and tolerances from the numerical contract. Keep
ordinary `float_cmp` active; seek approval for a necessary exact comparison it flags. Sort hash
collection output only where ordering matters. Use Clippy's default size thresholds unless an
approved project requirement establishes different limits.

When a project requires injectable environment or filesystem access, enable `disallowed_methods =
"deny"` and configure the actual adapter API in `clippy.toml`. Treat this as a conditional example,
replacing the adapter name with the project's API:

```toml
disallowed-methods = [
    { path = "std::env::var", reason = "use system::env_var for injectable environment access" },
]
```

When documentation uses identifiers that `doc_markdown` should accept as prose, add the project's
identifiers to `doc-valid-idents` and include `".."` to preserve Clippy's built-in list. Do not copy
another project's domain vocabulary.

## Commit the complete nightly rustfmt configuration (Required)

Commit `.rustfmt.toml` or `rustfmt.toml` with all settings below. Set `edition` and `style_edition`
to the project's edition; this example uses Rust 2024. Format imports as one sorted group with one
imported item per `use` statement, including re-exports.

```toml
edition = "2024"
style_edition = "2024"
unstable_features = true
reorder_imports = true
group_imports = "One"
imports_granularity = "Item"
newline_style = "Unix"
normalize_comments = true
normalize_doc_attributes = true
wrap_comments = true
float_literal_trailing_zero = "Always"
```

Run formatting with `cargo +nightly fmt`; the unstable settings require nightly rustfmt. Use
floating stable Rust for compilation, tests, and Clippy, and floating nightly for formatting. Keep
any MSRV check separate from the stable lint task. See
[rustfmt configuration](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md).

## Share local fix, lint, and CI tasks (Required)

Provide local tasks that apply formatting, apply Clippy fixes, check lints, and run tests. Use the
same toolchain channels, target and feature coverage, and strict checks locally and in CI. Install
or update the floating toolchains with:

```sh
rustup toolchain install stable --profile minimal --component clippy
rustup toolchain install nightly --profile minimal --component rustfmt
```

For a project using `just`, use these recipes. Adapt them to an existing task runner without
changing the policy. Generate and commit the lockfile for reproducible resolution; `--locked`
requires it to exist and match the manifest.

```just
clippy_scope := "--workspace --all-targets --all-features --locked"

fmt:
    cargo +nightly fmt --all

lint:
    cargo +nightly fmt --all --check
    cargo +stable clippy {{clippy_scope}} -- -D warnings

fix:
    cargo +stable clippy --fix {{clippy_scope}} --allow-dirty
    just fmt
    just lint

test:
    cargo +stable test --workspace --all-features --locked

check: lint test
```

When features cannot be enabled together, replace `--all-features` with explicit supported
combinations and use those same combinations for local fixes, linting, tests, and CI. Keep doctests
in test coverage. Have CI install the toolchains and invoke the shared tasks rather than maintain
separate Cargo commands. Review the fix diff; a successful fix task must finish with strict lint
verification.

Enforce `-D warnings` in both the local lint task and CI. Keep `#![deny(warnings)]` out of source.
Treat warnings from new stable Clippy releases and formatting changes from new nightly releases as
maintenance work. `todo!()` and `unimplemented!()` fail the strict check even though their manifest
levels are `warn`.
