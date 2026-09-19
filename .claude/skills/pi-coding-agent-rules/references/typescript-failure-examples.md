# Failure propagation example

Use this example when adapting a structured failure for several consumers. Adapt it to the
[error contract](typescript-error-contracts.md); use a plain `Error` when callers only propagate and
report failures. Run the snippets together as one module; the tests use Node assertions.

## Preserve a conflict until its consumer renders it

For a versioned write, require both revisions in the failure contract. Recognize independently
loaded copies by checking the shape, including the revision constraints. Keep the validator beside
the type; use a schema-derived type when the project already has a schema library.

```ts
type Conflict = {
  kind: "conflict";
  message: string;
  data: { suppliedRevision: number; currentRevision: number };
  cause?: unknown;
};

function isConflict(value: unknown): value is Conflict {
  if (
    typeof value !== "object" || value === null ||
    !("kind" in value) || value.kind !== "conflict" ||
    !("message" in value) || typeof value.message !== "string" ||
    !("data" in value) || typeof value.data !== "object" || value.data === null
  ) return false;
  const data = value.data;
  return "suppliedRevision" in data && typeof data.suppliedRevision === "number" &&
    Number.isSafeInteger(data.suppliedRevision) && data.suppliedRevision >= 0 &&
    "currentRevision" in data && typeof data.currentRevision === "number" &&
    Number.isSafeInteger(data.currentRevision) && data.currentRevision >= 0;
}

type SaveOutcome = { status: "saved" } | { status: "failed"; error: Conflict };

async function save(write: () => Promise<void>): Promise<SaveOutcome> {
  try {
    await write();
    return { status: "saved" };
  } catch (error) {
    if (!isConflict(error)) throw error;
    return { status: "failed", error };
  }
}
```

Use `save` only where callers need a returned failure outcome, such as retaining an editor's input
after a rejected write. Let unexpected defects reject unchanged. Require the injected write to check
the revision and mutate atomically; for an interaction-bound callback, reject an expired identity
before comparing revisions, as described in [workflows](typescript-workflows.md).

For a Pi tool, render the conflict's recovery data into the thrown message and retain the original
failure as its cause. At a terminal boundary with a reload control, show the failed action without
the tool's instructions. For a programmatic consumer, return the outcome unchanged. Apply these
adapters once at the selected boundary; do not pass a tool-rendered error back through domain code.

```ts
function toolOutcome(outcome: SaveOutcome): { status: "saved" } {
  if (outcome.status === "failed") {
    const failure = outcome.error;
    throw new Error(
      `${failure.message} Current revision: ${failure.data.currentRevision}. ` +
        "Reload current input before retrying.",
      { cause: failure },
    );
  }
  return outcome;
}

function terminalOutcome(outcome: SaveOutcome): string {
  return outcome.status === "saved" ? "Saved." : outcome.error.message;
}
```

Wire `toolOutcome(await save(write))` into `execute` before constructing its successful Pi result.
An unexpected rejection from `save` bypasses the renderer. For a command without a reload control,
compose a notification with the command's actual recovery action. Keep programmatic callers on
`SaveOutcome`; when publishing the contract, export its type and structural validator and document
the required data and recovery action. See [Pi failure signaling](pi-failure-signaling.md) for the
host's failed-result representation.

Test the original cause, failure identity, structural copies, invalid shapes, and each audience's
output. Assert exact tool wording here because the example defines it as a contract.

```ts
import assert from "node:assert/strict";

const cause = new Error("Conditional write rejected.");
const conflict = Object.assign(new Error("Save rejected.", { cause }), {
  kind: "conflict" as const,
  data: { suppliedRevision: 2, currentRevision: 3 },
});
const outcome = await save(async () => { throw conflict; });
assert.equal(outcome.status, "failed");
if (outcome.status !== "failed") throw new Error("Expected a rejected save.");
assert.equal(outcome.error, conflict);
assert.equal(outcome.error.cause, cause);
assert.deepEqual(outcome.error.data, { suppliedRevision: 2, currentRevision: 3 });
assert.equal(terminalOutcome(outcome), "Save rejected.");
assert.throws(() => toolOutcome(outcome), (error: unknown) => {
  assert.ok(error instanceof Error);
  assert.equal(error.cause, conflict);
  assert.equal(
    error.message,
    "Save rejected. Current revision: 3. Reload current input before retrying.",
  );
  return true;
});

const copy = { kind: "conflict", message: "Save rejected.", data: conflict.data };
assert.equal(isConflict(copy), true);
assert.deepEqual(await save(async () => { throw copy; }), { status: "failed", error: copy });
for (const invalid of [
  { ...copy, kind: "unknown" },
  { ...copy, data: undefined },
  { ...copy, data: { suppliedRevision: 2, currentRevision: -1 } },
  { ...copy, data: { suppliedRevision: 2, currentRevision: NaN } },
]) assert.equal(isConflict(invalid), false);

const defect = new TypeError("Broken invariant.", { cause });
await assert.rejects(async () => toolOutcome(await save(async () => { throw defect; })),
  (error: unknown) => error === defect);
```
