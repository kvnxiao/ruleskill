---
paths: .github/workflows/*.{yml,yaml}
description: "Pin third-party GitHub Actions to the latest published major (or a SHA with a version comment); never float on @main or @master."
---

# GitHub Actions Versioning

When authoring or editing files under `.github/workflows/`, **always
pin third-party actions to the latest published major version**.
A `@v4` reference that still works is technically fine, but update it to
`@v5` when the maintainers ship `@v5`.

## Why

GitHub deprecates Node.js runtime versions periodically (Node 16 →
Node 20 → Node 24, etc.). Older action majors usually run on older
Node majors and produce `Node.js N actions are deprecated`
warnings on every CI run. Letting them accumulate means:

- Annotations clutter the run UI and bury real warnings.
- A future GitHub-side hard cutoff turns a workflow that "works fine"
  into a hard failure with no warning window.
- The dependency surface drifts; security fixes published only on the
  newer major never reach us.

A quarterly update is sufficient to keep the action major current.

## How

Two acceptable pinning forms:

```yaml
# Use a floating major tag for low-impact CI workflows.
- uses: actions/checkout@v5
- uses: Swatinem/rust-cache@v2
```

```yaml
# Commit SHA — preferred when the workflow handles secrets, deploys,
# or otherwise has greater impact. Comment the version it corresponds
# to so maintainers can identify future updates.
- uses: actions/checkout@692973e3d937129bcbf40652eb9f2f61becf3332 # v5.0.0
```

**Avoid** floating un-versioned references (`@main`, `@master`); they
silently move and turn workflows non-reproducible.

When upgrading:

1. Check the action's repo for the current latest major (`gh release
   list --repo <owner>/<action>` or the README badge).
2. Bump the tag (or update the SHA + version comment).
3. Read the upstream changelog for the major bump — major releases can
   contain breaking changes.
4. Run the workflow on the bump branch before merging.

## Scope of "third-party"

This rule applies to any `uses:` reference. It includes:

- `actions/*` (GitHub-maintained but still external).
- Vendor actions (`Swatinem/rust-cache`, `EmbarkStudios/cargo-deny-action`,
  `dtolnay/rust-toolchain`, etc.).
- Reusable workflows from other repos.

It does **not** apply to `dtolnay/rust-toolchain@stable` /
`@nightly` style channel references — those are deliberately
floating-by-channel rather than version-pinned, and the upstream
contract is that they track the named Rust channel.

## Reciprocal expectation

When CI logs a deprecated-action warning, fix the action in the same
change. Keep the deprecation backlog at zero.
