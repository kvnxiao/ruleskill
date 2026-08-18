---
paths: **/*.{rs,toml}
description: "Rust error handling; anyhow for apps versus thiserror for libraries, opaque error types, from and source attributes, context, and keeping the happy path hot."
---

# Error Handling

## `anyhow` for applications, `thiserror` for libraries

Choose by whether the *caller* branches on the failure — not by taste.

- **Applications** that just propagate failures toward a human (CLIs, services): use `anyhow`. One `anyhow::Error`, `?` everywhere, `.context()` for breadcrumbs. The caller never matches on the error, so a bespoke type buys nothing.
- **Libraries** whose callers need to *react differently* to different failures: define your own error type, usually with `thiserror`. The caller can `match` a variant or call an `is_*` predicate.

The error crate is an implementation detail. Switching between a hand-written `impl std::error::Error`, `thiserror`, and back is **not** a breaking change — `thiserror` never appears in your public API. Start with `anyhow`; introduce a typed error only once a caller actually needs to branch.

## Typed errors: opaque wrapper over a private repr

A public `enum` freezes every variant into your semver surface. Wrap a **private** enum in an opaque public type so you can add, remove, or reorder variants freely.

```rust
use thiserror::Error;

// Keep the public type opaque so its representation can evolve.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ParseError(#[from] ErrorRepr);

impl ParseError {
    // Expose only the classifications callers need.
    pub fn is_eof(&self) -> bool {
        matches!(self.0, ErrorRepr::UnexpectedEof)
    }
}

// Keep variants private so they can evolve without a breaking change.
#[derive(Debug, Error)]
enum ErrorRepr {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid token at byte {offset}")]
    InvalidToken { offset: usize },
}
```

## `#[from]`, `#[source]`, and `'static`

```rust
#[derive(Debug, Error)]
pub enum Error {
    // `#[from]` generates the `From` impl and implies `#[source]`.
    #[error("i/o failed")]
    Io(#[from] std::io::Error),

    // A field named `source` is detected automatically as the source.
    #[error("parse failed at byte {offset}")]
    Parse { source: ParseError, offset: usize },
}
```

A source must be `'static` — `std::error::Error::source` returns `&(dyn Error + 'static)`, so a source field carrying a borrowed lifetime will not compile.

## One error type per crate

An alternative to a public enum is a single crate-wide `Error` whose variants stay private. Callers classify through non-exhaustive `is_*` predicates. Keep the public type to one pointer and make it cheap to clone by boxing the payload behind `Arc`.

```rust
use std::sync::Arc;

/// Return one error type from every fallible API in this crate.
#[derive(Clone)]
pub struct Error {
    // One pointer; cloning copies the pointer even when it wraps an io::Error.
    inner: Option<Arc<ErrorInner>>,
}

struct ErrorInner {
    kind: ErrorKind,
    // ... source chain, context messages ...
}

// Keep variants private; callers use predicates instead of matching on them.
enum ErrorKind { /* ... */ }

impl Error {
    /// Predicates are not exhaustive, so new ones are additive.
    pub fn is_not_found(&self) -> bool {
        matches!(self.inner.as_deref().map(|i| &i.kind), Some(ErrorKind::NotFound))
    }
}
```

Use this when you want a stable, tiny error surface across a large API and are willing to expose classification rather than structure. Use the opaque-wrapper enum when callers benefit from a real (if private) variant set.

## `Result` alias with a defaulted error param

```rust
// The default error type lets callers override E when needed.
pub type Result<T, E = Error> = core::result::Result<T, E>;
```

## Add context: eager vs lazy

`.context(v)` evaluates its argument eagerly, on every call including the success path. `.with_context(|| ...)` defers it to the moment an error actually occurs. Let the cost of building the message decide.

```rust
use anyhow::{Context, Result};

fn load(path: &Utf8Path) -> Result<Config> {
    // Bad: `format!` allocates on successful reads.
    let text = fs_err::read_to_string(path).context(format!("reading {path}"))?;

    // Lazy context: the closure runs only on failure.
    let text = fs_err::read_to_string(path).with_context(|| format!("reading {path}"))?;

    // Eager context is fine for a bare string literal.
    toml::from_str(&text).context("parsing config")
}
```

A crate-wide error type can offer the same `.context()` chaining on its own type; a `thiserror` enum instead carries context through `#[source]` fields.

## Inspect an error: walk the chain, downcast

```rust
use anyhow::Error;

fn io_error_kind(err: &Error) -> Option<std::io::ErrorKind> {
    // Walk the full cause chain; `.chain().last()` is the root cause.
    for cause in err.chain() {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            return Some(io.kind());
        }
    }
    None
}
```

## Keep the happy path hot

Error construction is cold. Mark constructors `#[cold]` (and, for the hottest crates, `#[inline(never)]`) so the optimizer lays them out away from the success path.

```rust
impl Error {
    #[cold]
    #[inline(never)]
    fn new(kind: ErrorKind) -> Error {
        Error { inner: Some(Arc::new(ErrorInner { kind })) }
    }
}
```
