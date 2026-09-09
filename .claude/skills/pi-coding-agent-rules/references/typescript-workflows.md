# TypeScript workflows

## Bind actions to the revision the user saw (Required)

When approval, submission, or an answer applies to displayed content, include the content identity and revision in the action. After an asynchronous save or refresh, recheck that revision before committing the action. If the interaction was replaced, stop the queued action and require fresh input; never apply an old click to the latest content.

Verify by delaying a save, replacing the displayed revision, and delivering the queued action. Assert that neither revision is submitted by the stale action and that the replacement remains actionable. Use the rendered interface to establish event ordering.

## Invalidate derived artifacts with their prerequisites (Required)

When a review, approval, or generated artifact depends on decisions that change, mark the dependent artifact superseded at the state owner. Preserve historical content, but exclude superseded artifacts from current selection, resumption, and approval. Do not infer validity from file existence or the latest timestamp.

Verify by changing a prerequisite, reopening the workflow, and attempting to select the old artifact. Assert that history remains readable and only an artifact based on current prerequisites can advance.

## Recover the entire state transition (Required)

When a transition leaves a reviewable state, define recovery for every fallible step, including settings reads, validation, UI startup, and persistence preparation. Before a commit, preserve the exact revision and restore usable retry and cancellation actions on failure. After a possible commit, reconcile the stored outcome before retrying; do not repeat a mutation whose result is unknown.

Inject failures at the participating boundaries. Repair each cause and retry the same operation. Assert preserved content and revision, successful continuation without duplicate artifacts or unintended submission, and an available cancellation path.

## Replace interactive views through an owned loop (Default)

When users can repeatedly switch between terminal or browser views, let each view finish and release its resources before opening its replacement. Use iteration to avoid accumulating suspended recursive calls. Across switches, preserve the workflow revision, cancellation ownership, and generation checks for late callbacks.

Verify repeated switching with a delayed callback from an old view. Assert that only the current view accepts input, cancelled work cannot commit, and prior listeners and resources are released.
