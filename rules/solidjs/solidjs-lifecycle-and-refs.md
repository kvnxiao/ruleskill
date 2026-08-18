---
paths: **/*.{tsx,jsx}
description: "SolidJS lifecycle and ref rules; onMount/onCleanup pairing, no cleanup-return from effects, ref timing, and signal refs for conditional elements."
---

# Lifecycle and Refs

Components run once, so lifecycle hooks cover mount and disposal: `onMount` runs once after the component's elements are in the DOM, and `onCleanup` runs when the owning scope disposes — on unmount, or before each re-run when registered inside an effect or memo. There is no `componentDidUpdate` equivalent; reactive computations handle updates.

## Pair Every Imperative Resource With `onCleanup`

Register cleanup in the same scope that creates the resource: intervals, `window` listeners, observers, third-party widget instances.

```tsx
onMount(() => {
  const chart = new Chart(el, options);
  onCleanup(() => chart.destroy());
});
```

## Effects Do Not Return Cleanup Functions

Returning a function from `createEffect` does nothing. Register `onCleanup` inside the effect instead; it runs before each re-run and on disposal.

```tsx
// Bad: Solid ignores the returned cleanup function.
createEffect(() => {
  const id = setInterval(tick, delay());
  return () => clearInterval(id);
});

// Good: register cleanup with onCleanup.
createEffect(() => {
  const id = setInterval(tick, delay());
  onCleanup(() => clearInterval(id));
});
```

## Refs: Assigned During Render, Ready in `onMount`

Use a definite-assignment local with the `ref` attribute. The ref is set before `onMount`; perform DOM measurement in `onMount`, never in the component body.

```tsx
let el!: HTMLDivElement;

onMount(() => setWidth(el.getBoundingClientRect().width));

return <div ref={el} />;
```

## Signal Refs for Conditional Elements

Inside `<Show>` or other control flow, a plain local can be unset or stale. Use a signal as the ref so consumers react to the element appearing and disappearing.

```tsx
const [el, setEl] = createSignal<HTMLDivElement>();

createEffect(() => {
  const node = el();
  if (node) observer.observe(node);
});

<Show when={open()}>
  <div ref={setEl} />
</Show>
```

## Plain `let` Replaces `useRef` Boxes

Any `let` in the component body is a stable instance variable because the function runs once. Non-reactive instance state does not need a mutable-box wrapper.
