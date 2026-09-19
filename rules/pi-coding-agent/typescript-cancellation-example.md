# Cancellation ownership example

Use this example when asynchronous work can be replaced before it finishes. Adapt it to the
[cancellation contract](typescript-error-contracts.md#keep-cancellation-and-supersession-on-the-signal-required).
Run the snippets together as one module; the tests use Node assertions.

## Cancel work without changing its replacement

Use a controller identity when only the latest operation may commit to shared state. Compose caller
and owner signals, but check ownership separately before mutation and cleanup. In this example,
`prepare` must reject cancellation with `signal.reason`; a different rejection remains a failure
even after abort. When an I/O API wraps its abort reason, normalize only its documented cancellation
shape at that API's adapter.

```ts
type Owner = { current: AbortController | undefined; value: string };
type WorkOutcome = { status: "committed" } | { status: "cancelled"; reason: unknown };
const superseded = { kind: "superseded" } as const;

async function runLatest(
  owner: Owner,
  caller: AbortSignal,
  prepare: (signal: AbortSignal) => Promise<string>,
  release: () => void,
): Promise<WorkOutcome> {
  const controller = new AbortController();
  const signal = AbortSignal.any([caller, controller.signal]);
  try {
    signal.throwIfAborted();
    const previous = owner.current;
    owner.current = controller;
    previous?.abort(superseded);
    const value = await prepare(signal);
    signal.throwIfAborted();
    if (owner.current !== controller) return { status: "cancelled", reason: superseded };
    owner.value = value;
    return { status: "committed" };
  } catch (error) {
    if (signal.aborted && Object.is(error, signal.reason)) {
      return { status: "cancelled", reason: signal.reason };
    }
    throw error;
  } finally {
    if (owner.current === controller) owner.current = undefined;
    release();
  }
}
```

Pass a nonthrowing `release` that disposes only this invocation's resources, including on pre-abort.
Do not clear shared UI or change shared operation state inside that callback. Keep the ownership
check and in-memory assignment synchronous; for external writes, enforce ownership or revision
checks at the storage boundary. The example discards obsolete prepared values but cannot undo
committed effects or stop a `prepare` implementation that ignores cancellation indefinitely.

Use deferred operations to test cancellation followed by replacement before the old operation
finishes. Assert that the first abort reason survives supersession and that old cleanup leaves the
replacement controller installed. Also inject an unrelated defect after replacement to verify that
cancellation does not hide it.

```ts
import assert from "node:assert/strict";

for (const fail of [false, true]) {
  const owner: Owner = { current: undefined, value: "initial" };
  const first = Promise.withResolvers<string>();
  const second = Promise.withResolvers<string>();
  const caller = new AbortController();
  const reason = { kind: "cancelled" };
  const defect = new TypeError("Preparation invariant failed.");
  const released: string[] = [];
  let observed: AbortSignal | undefined;
  const old = runLatest(owner, caller.signal, async (signal) => {
    observed = signal;
    return await first.promise;
  }, () => { released.push("old"); });
  caller.abort(reason);
  const replacement = runLatest(owner, new AbortController().signal,
    async () => await second.promise, () => { released.push("replacement"); });
  const replacementController = owner.current;
  if (fail) {
    const rejected = assert.rejects(old, (error: unknown) => error === defect);
    first.reject(defect);
    await rejected;
  } else {
    first.resolve("obsolete");
    const result = await old;
    assert.equal(result.status, "cancelled");
    if (result.status !== "cancelled") throw new Error("Expected cancellation.");
    assert.equal(result.reason, reason);
  }
  assert.equal(observed?.reason, reason);
  assert.equal(owner.current, replacementController);
  assert.equal(owner.value, "initial");
  assert.deepEqual(released, ["old"]);
  second.resolve("replacement");
  assert.deepEqual(await replacement, { status: "committed" });
  assert.equal(owner.value, "replacement");
  assert.equal(owner.current, undefined);
  assert.deepEqual(released, ["old", "replacement"]);
}
```

Add pre-abort, direct cancellation rejection, and supersession without prior caller cancellation to
the extension's tests. Verify Pi's tool status and terminal behavior through the actual adapters;
these examples test the failure and ownership contracts without loading Pi.
