---
paths: **/*.{rs,toml}
description: "Rust performance; Cow, capacity pre-allocation, FxHash, arena indices, compact collections, byte-string IO, and Cargo build profiles."
---

# Performance Considerations

## Use `Cow` for Conditional Cloning

```rust
use std::borrow::Cow;

fn process(input: &str) -> Cow<str> {
    if input.contains("special") {
        Cow::Owned(input.replace("special", "SPECIAL"))
    } else {
        Cow::Borrowed(input)
    }
}
```

## Allocations and Capacity

```rust
// Bad: reallocates repeatedly.
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// Good: pre-allocate known capacity.
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

## Avoid Unnecessary Copies

```rust
// Borrow items during iteration.
for item in &collection {  // Borrow the collection; do not consume it.
    process(item);
}

// Consume the collection with drain().
for item in collection.drain(..) {
    consume(item);
}
```

## Fast hashing for internal maps

The std `HashMap` uses SipHash for HashDoS resistance. That protection only matters for maps keyed by untrusted input. For internal maps, `FxHash` is much faster.

```rust
use rustc_hash::FxHasher;
use std::{collections::HashMap, hash::BuildHasherDefault};

// Use FxHashMap for internal maps and std HashMap for attacker-facing maps.
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
```

## Arena indices instead of pointer graphs

```rust
// Bad: `Rc<RefCell<Node>>` allocates per node and checks borrows at runtime.

// Good: use Copy indices into a flat arena. NonZeroU32 gives `Option<NodeId>`
// the same size as `NodeId` via the 0 niche.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(std::num::NonZeroU32);

let nodes: IndexVec<NodeId, Node> = IndexVec::new();
```

## Compact collections on hot paths

```rust
// Store small strings inline without heap allocation up to ~24 bytes.
pub struct Name(compact_str::CompactString);

// Use ThinVec for common short lists.
use thin_vec::ThinVec;

// Store up to N elements inline before spilling to the heap.
// Most lines have zero or one assertion, so optimize for a single element.
type Assertions = smallvec::SmallVec<[Assertion; 1]>;
```

## Shrink before caching; box immutable payloads

`collect`, `extend`, reservation, and `remove` can leave a collection over-allocated. Drop the spare capacity before storing it in a long-lived cache.

```rust
vec.shrink_to_fit();

// Box immutable payloads as slices or str values; they have no capacity field
// and cannot grow.
let payload: Box<[u8]> = data.into_boxed_slice();
let name: Box<str> = s.into_boxed_str();
```

## Byte strings for hot IO

On hot IO paths, work in bytes (`&[u8]`, `bstr`) and defer UTF-8 validation until you actually need text. Name your buffer capacities.

```rust
use bstr::ByteSlice;

const DEFAULT_BUFFER_CAPACITY: usize = 64 * (1 << 10); // 64 KB
```

## Cargo build profiles

```toml
# Preserve line-table debug info in release builds for profilers and
# backtraces; the speed cost is negligible.
[profile.release]
debug = 1

# Use a max-optimization profile for shipping binaries.
[profile.release-lto]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
debug-assertions = false
overflow-checks = false

# Limit extra compile time to crates that dominate runtime.
[profile.release.package.parser]
codegen-units = 1

# Keep release speed, symbols, and full debug info for profiling.
[profile.profiling]
inherits = "release"
debug = "full"
strip = false
lto = false
```
