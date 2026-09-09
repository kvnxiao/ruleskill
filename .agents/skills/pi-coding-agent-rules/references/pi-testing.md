# Testing

## Test domain contracts through public operations (Default)

Test accepted and rejected state transitions, boundary validation, and externally observable results. A test should fail when the contract breaks even if private helpers or file layout change. Keep pure domain tests independent of Pi and terminal rendering.

For asynchronous state changes, control completion order with deferred operations or a fake clock. Cover cancellation before work starts, cancellation during I/O, and a stale completion after the owner changes. Avoid timing assertions that depend on arbitrary sleeps.

## Reproduce failures at the layer that owns the behavior (Required)

When a defect depends on browser event ordering, use rendered-browser tests; pure state tests cannot establish click and refresh ordering. Use runtime fixtures for settings or initialization failures and persistence fixtures for cancellation and restoration. Assert preserved content, revision, and available next actions, not merely an error result.

For a recovery path, repair the injected failure and retry the same operation. Verify successful continuation of the original revision without duplicate artifacts or unintended submission, and verify that cancellation remains available.

## Distinguish correctness changes from simplification (Default)

When proposing a cleanup, identify a concrete reduction in complexity or work and preserve observable behavior. A change to identity, ordering, or recovery is a correctness change when it changes user-visible outcomes. For correctness changes, reproduce the defect and verify the corrected contract. For behavior-preserving simplification, run the same relevant checks before and after the change.

## Exercise the lifecycle transitions the extension uses (Required)

For session-owned resources, test repeated startup and shutdown, reload, and failed initialization. Verify that cleanup is idempotent and that an old callback cannot modify the replacement session.

For persisted branch state, test resume and tree navigation with divergent branches. Verify that reconstruction selects the active branch and does not repeat external effects. For mutable snapshots, mutate the current state and assert that an earlier snapshot is preserved. [Session lifecycle](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/extensions.md#session-events), [branch-state example](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/examples/extensions/todo.ts).

## Test tool and mode contracts at the adapter boundary (Required)

For custom tools, test malformed arguments, domain failures, cancellation, and output limits. For file mutation tools, exercise competing updates to the same file. Assert Pi's failure status for failed execution; an error-colored renderer does not establish that status.

Test each mode the extension claims to support. Include unavailable UI and dismissed prompts. For terminal components, exercise narrow widths, Unicode, resize, and partial results. For an RPC client, split records across chunks, interleave events and responses, and close the transport with requests pending. [Tool contracts](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/extensions.md#custom-tools), [RPC framing](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/rpc.md#framing), [TUI width](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/tui.md#line-width).

## Isolate tests from user resources (Required)

Use temporary working directories and explicit resource loading for integration tests. Use `SessionManager.inMemory` and in-memory settings when persistence is irrelevant. In-memory sessions do not disable extension discovery or model network access; configure those separately.

Use deterministic model or transport doubles for orchestration contracts. Reserve live-provider tests for behavior that requires the provider, and report a skipped live test as skipped. Do not require a paid model request to test local state transitions. [SDK configuration and in-memory managers](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/sdk.md).

## Match verification claims to the exercised layer (Required)

When reporting verification, name the layer exercised and any skipped layer needed to establish the claim. Unit tests, rendered-browser tests, scripted Pi sessions, and real terminal interaction establish different behavior. Deterministic model doubles test orchestration; they do not establish real-model planning quality. Evaluate that quality separately with supervised scenarios.

Keep routine automated tests local and deterministic. Report a scripted interaction as scripted and a manual interaction as manual; do not infer either from an adapter unit test.

## Verify the distributed artifact (Conditional)

For a package intended for distribution, test installation from the packed artifact in a temporary project. Verify resource discovery and runtime dependencies without development dependencies. Exercise the supported Pi versions and provider schema constraints the package claims, using the repository's existing test stack.

Keep compiler and linter checks in the verification workflow even though these references omit their deterministic rules. [Pi package dependencies](https://github.com/earendil-works/pi/blob/v0.85.1/packages/coding-agent/docs/packages.md#dependencies).
