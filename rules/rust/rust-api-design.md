---
paths: **/*.{rs,toml}
description: "Public API design for libraries; ergonomic, semver-evolvable interfaces via options structs, sealed traits, non_exhaustive, features and no_std, unsafe and macro hygiene."
---

# API Design

Use public interfaces that stay ergonomic for callers and can evolve without breaking changes.

## Options struct + `impl Into` for overload-like ergonomics

Rust has no function overloading. Accept `impl Into<Options>` and provide a family of `From` impls: the simple call passes a bare value, and richer calls pass a tuple or the full struct. This avoids a builder for common calls and keeps optional parameters in `RoundOptions`.

```rust
pub struct RoundOptions { smallest: Unit, increment: i64 }

impl From<Unit> for RoundOptions {
    fn from(smallest: Unit) -> Self { Self { smallest, increment: 1 } }
}
impl From<(Unit, i64)> for RoundOptions {
    fn from((smallest, increment): (Unit, i64)) -> Self { Self { smallest, increment } }
}

impl Span {
    // Common forms: Unit, (Unit, i64), or RoundOptions.
    pub fn round<R: Into<RoundOptions>>(self, options: R) -> Result<Span> {
        let options = options.into();
        // ...
    }
}
```

## Deferred-validation builder

Validating inside each setter makes some valid end states unreachable through valid intermediate states.

```rust
// Bad: setting the day before the month validates Feb 29 against the current
// month and rejects it before the leap year is selected.
date.with().day(29).month(2).build()  // spurious error

// Good: setters store values, and `build()` validates the complete date once.
date.with()
    .month(2)
    .day(29)   // order-independent, not checked yet
    .build()?  // single validation point
```

## Don't derive `Eq`/`Ord`/`PartialEq` reflexively

A derived `PartialEq` compares field-by-field. That is often wrong: two values can be semantically equal with different representations.

```rust
// Bad: derived PartialEq makes 2.hours() != 120.minutes() despite equal duration.

// Compare the meaningful field:
impl PartialEq for Zoned {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp() == other.timestamp() // compare the instant, ignore the zone
    }
}

// Withhold equality when "equal" is ambiguous, and expose an explicit opt-in
// newtype for field-wise comparison:
#[repr(transparent)]
pub struct SpanFieldwise(pub Span); // via `span.fieldwise()`, only when asked for
```

## `#[non_exhaustive]` on config enums expected to grow

```rust
/// Allow new strategies in semver-compatible releases.
#[non_exhaustive]
pub enum Disambiguation {
    Compatible,
    Earlier,
    Later,
    Reject,
}
```

## Paired panicking / fallible constructors

Give literals a terse panicking constructor and untrusted input a fallible one. A `const {}` block moves the literal's panic to compile time.

```rust
let d = date(2024, 2, 29);          // panicking: for author-known-good literals
let d = Date::new(year, month, day)?; // fallible: for runtime / user input

const NEW_YEAR: Date = const { date(2025, 1, 1) }; // invalid literal fails to compile
```

## Extension traits for literal ergonomics

An extension trait can add literal syntax to primitives. Document it as literals-only and panicking, and pair it with `try_*` methods for user input.

```rust
use jiff::ToSpan;

let span = 2.hours().minutes(30); // ergonomic literals; panics if out of range
let span = n.try_hours()?;        // for untrusted input
```

## Sealed traits

A trait bounded on a `pub(crate)` `Sealed` supertrait can be called publicly but cannot be implemented downstream. You can add methods later without a major bump, and no downstream type can implement it.

```rust
pub trait Context<T>: private::Sealed {
    fn context<C: Display + Send + Sync + 'static>(self, cx: C) -> Result<T>;
}

mod private {
    pub trait Sealed {}
    impl<T, E: std::error::Error> Sealed for Result<T, E> {}
}
```

## Hide macro glue behind `#[doc(hidden)]`

Generated macro code needs public items for expansion. Put them in a `#[doc(hidden)] pub mod __private` and keep them outside the intended semver surface.

```rust
#[doc(hidden)]
pub mod __private {
    pub use core::result::Result;
    // Re-export items used by generated code.
}
```

## Private modules, one curated `pub use`

Decouple your file layout from your public path: keep modules private and export a single curated block. Rename or move files without touching the public API.

```rust
mod error;
mod span;
pub mod civil; // only genuinely-public modules are `pub`

pub use crate::{
    error::Error,
    span::{Span, SpanRound, Unit},
};
```

## Lossless → `From`, lossy → `TryFrom`

A conversion that can overflow or lose data must be fallible. Never hide truncation behind an infallible `From`.

```rust
impl From<i32> for Duration { /* widening: always succeeds */ }

impl TryFrom<std::time::Duration> for SignedDuration {
    type Error = Error;
    fn try_from(d: std::time::Duration) -> Result<Self> {
        let secs = i64::try_from(d.as_secs())?; // may not fit
        // ...
    }
}
```

## Library authoring: features and `no_std`

Applies when you publish a library.

```rust
// Enable `no_std` when the std feature is disabled.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
```

```toml
[features]
default = ["std"]
std = ["alloc"]        # tier features: std ⊃ alloc ⊃ core
alloc = []
derive = ["dep:my_derive"] # optional proc-macro, off by default

# Preserve the removed feature as a documented no-op so `features = ["backtrace"]` still builds.
backtrace = []

# Semantic trade-off: does not preserve identity — enable only if intended.
rc = []
```

- **Features must be additive.** Enabling one only adds behavior, never changes it. Downstream crates share your feature resolution.
- **Document what degrades** when a feature is off, and what a footgun feature does.
- **Don't forward target-sensitive features** from a library. Let the final binary opt in (e.g. a `js` feature that only makes sense on `wasm32-unknown-unknown`).
- **Centralize `#[cfg]`** in one module as type aliases + macros, so the rest of the crate stays cfg-free.
- **Probe unstable APIs in `build.rs`** and emit `println!("cargo:rustc-check-cfg=cfg(...)")`, rather than leaking a nightly feature to users.

## `unsafe` discipline

Forbid `unsafe` by default. If a crate must relax that baseline, keep the discipline; `unsafe_code = "warn"` is a middle ground.

Before writing `unsafe`, search for a safe wrapper crate:

| Domain | Unsafe Bindings | Safe Wrapper |
|--------|-----------------|--------------|
| Windows API | `windows-sys` | `winsafe` |
| POSIX/Unix | `libc` | `nix`, `rustix` |
| SQLite | `libsqlite3-sys` | `rusqlite` |
| OpenSSL | `openssl-sys` | `openssl` |
| Memory | raw pointers | `bytemuck`, `zerocopy` |

```rust
#![deny(unsafe_op_in_unsafe_fn)]

// State the invariant required by each unsafe block.
// Safety: `ptr` is non-null and points to an initialized `T` (checked above).
let value = unsafe { &*ptr };
```

Keep unsafe blocks minimal, do not expose them in a public API (wrap them in a safe abstraction), and document caller obligations in a `/// # Safety` section. Run `cargo +nightly miri test` over crates containing `unsafe`.

Lock the public auto-trait surface with `assert_send::<T>()`-style tests, and add drop-count tests for by-value ownership tricks.

## Macro-author hygiene

Generated code runs in the caller's namespace, so it must be self-contained.

```rust
// In a derive macro's `quote!` expansion:
quote! {
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl #generics ::core::fmt::Display for #ty { /* ... */ }
}
```

- Fully-qualify every path (`::core::`, `::std::`, `::your_crate::`) so it works regardless of the caller's `use`s.
- Emit `#[automatically_derived]` on generated impls.
- Add targeted `#[allow(...)]` for lints your codegen cannot avoid, and test the output under `#![deny(...)]`.
