# TypeScript domain boundaries

## Encode valid workflow states (Default)

When fields are valid only together, model the alternatives as a discriminated union. A job with `status: "succeeded"` requires its result; a job with `status: "failed"` requires its error. Independent `loading`, `done`, and `failed` booleans permit combinations the workflow may forbid.

Define allowed transitions in the state owner's operations. Exhaustive switches check coverage after the state model exists; they do not decide which states or transitions belong in that model. [Discriminated unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions).

## Validate when data crosses a trust or persistence boundary (Required)

Treat decoded network responses, configuration, subprocess output, and persisted entries as untrusted data until a runtime parser establishes their contract. A TypeScript assertion cannot validate a payload. Validate shape, then domain constraints such as bounds, supported versions, and relationships between fields.

Normalize accepted external representations into one internal representation at the boundary. Keep validated values typed through internal calls instead of repeatedly parsing them. When missing, explicitly empty, malformed, and unsupported input require different actions, preserve those distinctions. [TypeScript assertions](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-assertions).

## Compare structured inputs by domain meaning (Conditional)

When equality controls answer reuse, revision identity, or cache validity, define which fields determine meaning. JSON object member order must not change identity. Normalize nested fields that determine meaning, including options and recommendations; preserve stable option identifiers. Sort a collection only when its order has no domain meaning. Do not use raw serialization as an equality contract.

Verify equivalent objects with reordered keys, meaningful changes to nested fields, and reordered collections whose order matters. Assert that equivalent input preserves existing work and changed meaning invalidates only its dependents.

## Distinguish identifiers and units when substitution is hazardous (Conditional)

When confusing values of the same primitive type has a concrete consequence, distinguish them through domain types and validated constructors: session IDs versus tool-call IDs, bytes versus tokens, or relative paths versus resolved paths. Keep units visible at arithmetic and serialization boundaries.

Use a brand or wrapper only when it prevents a demonstrated class of substitution errors. Do not brand every string or number. A brand records a validation or construction contract; a cast alone does not establish that contract.

## Separate wire formats from domain models when their contracts differ (Conditional)

When an external API's nullability, names, or versioning differs from the domain, translate through a boundary adapter. Do not spread transport objects into internal state or public results and accidentally adopt unknown fields.

When a schema defines an external contract, derive its TypeScript type where the schema library supports it. Maintain a separate domain type only when the translation has meaning. For exported results, decide which fields callers may rely on and keep transient rendering or transport details private.
