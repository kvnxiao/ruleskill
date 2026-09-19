# TypeScript error contracts

For an implementation with focused assertions, read the optional
[failure propagation example](typescript-failure-examples.md) or
[cancellation ownership example](typescript-cancellation-example.md) for the contract being changed.

## Decide outcome, thrown error, or abort reason before writing a throw (Default)

Choose the channel from the operation's contract: return an expected answer as a result variant,
throw a failure the caller must react to, and convey the caller's cancellation or supersession
through the signal's abort reason. For example, a search returns `{ status: "empty" }`, a stale
write throws a `conflict` with the current revision, and a replaced operation aborts with `{ kind:
"superseded" }`. The channel determines whether the caller consumes an answer, handles a failure, or
stops work. [Abort reasons](https://nodejs.org/api/globals.html#abortcontrollerabortreason).

## Define failure kinds by the caller's reaction (Default)

Define expected failure kinds by distinct recovery actions, such as correcting input, reloading a
revision, or repairing an affected file. Keep a kind only when a consumer branches on it or a
boundary reacts differently; do not create one kind per message. Keep unexpected defects outside the
expected-failure contract and cancellation or supersession in the signal contract.

## Require the data each failure kind needs (Conditional)

When callers need different recovery actions or structured information, use a discriminated union
that requires each kind's data. For example, require supplied and current revisions for a conflict
and an affected path for a file repair. Avoid an optional data bag that permits a conflict without
revision data. Use a plain `Error` when callers only need to propagate and report the failure;
introduce a custom class only when constructing or throwing typed failures benefits from it.
[Discriminated unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions).

```ts
type Failure = { message: string; cause?: unknown } & (
  | { kind: "conflict"; data: { suppliedRevision: number; currentRevision: number } }
  | { kind: "repair-file"; data: { path: string } }
);
```

Narrow caught `unknown` values before reading fields. At a shared boundary, validate the
discriminant and each variant's required data, including domain constraints such as nonnegative
safe-integer revisions or nonempty paths. Reject malformed shapes without guessing a kind from
message text. Dispatch validated variants exhaustively.
[Strict catch bindings](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-4.html#defaulting-to-the-unknown-type-in-catch-variables---useunknownincatchvariables),
[exhaustiveness lint](https://typescript-eslint.io/rules/switch-exhaustiveness-check/).

## Preserve failures through intermediate layers (Required)

Catch only where the layer can translate a recognized failure, recover, release resources, or report
at the final boundary. Rethrow unrecognized failures unchanged; a storage catch must not classify an
unexpected `TypeError` as a repairable I/O failure. Use `finally` for unconditional resource release
without replacing the original failure.

When translating a recognized failure, retain its structured data and original cause. If an
intermediate API returns a failure outcome, include the original error instead of retaining only its
message. For example, return `{ outcome: "error", error }` and let the tool or UI boundary render
it. Preserve a typed host result as the cause when converting it to a thrown failure; keep its
status and data available to consumers. When no translation is needed, preserve object identity.
[Error cause](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/cause).

## Give remediation text one owner at the boundary (Default)

State what failed at the throw site and include the data needed for recovery. Add recovery
instructions once, at the boundary that knows the audience and permitted next action. For example, a
tool may need a revision and reload instruction, while a terminal editor can show the rejected save
beside a reload control. Preserve typed failures for programmatic callers. Never add generic retry
advice or speculative remediation to unexpected defects. Exclude credentials and sensitive payloads
from rendered output, and report a failure once rather than logging it at every layer.

## Keep cancellation and supersession on the signal (Required)

Treat cancellation as control flow and preserve the reason supplied to `controller.abort(reason)`.
When either the caller or lifecycle owner can cancel work, compose their signals with
`AbortSignal.any` where supported. Check cancellation before starting work and after awaits before
further work or mutation, for example with `signal.throwIfAborted()`. At a catch boundary, inspect
the operation's signal and reason before rendering a failure; propagate cancellation or return the
documented cancelled outcome. Do not infer cancellation solely from an error's name or message.

Recheck ownership before mutation, recovery, and cleanup. A composed signal retains its first abort
reason; if cancellation precedes supersession, that reason does not establish ownership of current
state. For example, compare the captured generation before pausing state, and clear a shared
controller slot only if it still contains the captured controller. Release resources owned by the
old operation without modifying replacement work. Preserve unrelated defects when cancellation races
with failure, and define how the operation reports each outcome.
[Node.js AbortSignal](https://nodejs.org/api/globals.html#class-abortsignal).

## Document the kinds a contract may throw (Required)

For a callback or interface other code implements, document each expected failure kind, required
data, recovery action, and cancellation behavior. Export the contract's failure variants from the
public entry point and version changes to the shared contract. Across extension-loader boundaries,
provide structural recognition rather than relying solely on `instanceof`; separately loaded class
copies need not share identity. For example, document `conflict` with both revisions and require
reloading current input before retry. Consumers must be able to recover without parsing messages;
apply the
[public documentation rule](typescript-code-organization.md#document-every-exported-symbol-required).

## Assert kinds, not message text (Default)

Assert failure kinds, required data, and original causes through the public operation. For example,
assert a conflict's supplied and current revisions, and use identity assertions for unchanged errors
or retained causes. Test structural copies without shared class identity and reject unknown kinds,
missing data, and invalid field values. Assert exact messages when wording is part of the contract,
such as an audience's recovery instruction; message substrings alone do not establish
classification.
