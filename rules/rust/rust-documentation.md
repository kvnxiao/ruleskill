---
paths: **/*.{rs,toml}
description: "Rustdoc requirements; public-API and module docs, the module skeleton, crate-root cookbook, include_str rationale, and the docs.rs cfg knob."
---

# Documentation Requirements

## Public API Documentation

Every public item must have documentation. Clippy enforces `# Errors`
and `# Panics` sections (`missing_errors_doc` / `missing_panics_doc`);
this rule covers the rest — the prose summary, `# Arguments`, and
`# Examples`.

```rust
/// Process input data and return a processed result.
///
/// # Arguments
///
/// * `input` - The input string to process
/// * `options` - Processing options
///
/// # Examples
///
/// ```
/// use my_library::{process, Options};
///
/// let result = process("hello", Options::default())?;
/// assert_eq!(result.value(), "HELLO");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn process(input: &str, options: Options) -> Result<ProcessedData> {
    todo!()
}
```

## Module Documentation

```rust
//! Load and validate application configuration.
//!
//! # Examples
//!
//! ```
//! use my_library::config::Config;
//!
//! let config = Config::from_file("config.toml")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
```

## Module `//!` Skeleton

For a substantial module, follow a three-part skeleton: an **Overview** with the types and runnable examples, a **"What is X?"** concept section, and a **"When should I use X?"** decision guide that points to the alternatives.

```rust
//! Provide facilities for civil (time-zone-less) datetimes.
//!
//! # Overview
//!
//! - [`Date`]: a calendar date.
//! - [`Time`]: a wall-clock time.
//!
//! ```
//! use my_crate::civil::date;
//! let d = date(2024, 3, 14);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What is "civil" time?
//!
//! Explain the civil-time model here.
//!
//! # When should I use civil time?
//!
//! Explain when civil time fits and how it differs from the alternatives.
```

## Crate Root as Cookbook and Spec

Because new users start at the crate root, use it to:

- List what the crate supports and what it does **not**, linking each unsupported feature to a tracking issue.
- State the panic policy ("APIs that panic by design are few and clearly documented as such").
- Embed a short cookbook of runnable, task-oriented examples.

## Long-Form Rationale via `include_str!`

Keep design rationale in top-level Markdown (PR-reviewable, one source of truth) and render it into the docs through a hidden documentation module.

```rust
/// Render design and platform documentation.
pub mod _documentation {
    #[doc = include_str!("../DESIGN.md")]
    pub mod design {}
    #[doc = include_str!("../PLATFORM.md")]
    pub mod platform {}
}
```

## Own Your `docs.rs` cfg Knob

To prevent another crate from toggling your nightly-only doc attributes, use a crate-specific cfg name instead of the shared `docsrs`.

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs_mycrate"]
```

```rust
#![cfg_attr(docsrs_mycrate, feature(doc_cfg))]
```
