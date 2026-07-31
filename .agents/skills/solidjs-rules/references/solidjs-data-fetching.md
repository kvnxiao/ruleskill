---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS async data rules; createResource over async effects, source gating, Suspense/ErrorBoundary composition, mutate/refetch, and solid-router query/createAsync."
---

# Data Fetching

## `createResource`, Not Async Effects

Async effects silently lose tracking after the first `await` and handle neither races nor loading states. `createResource` integrates with `Suspense` and `ErrorBoundary`, re-fetches when its source changes, and exposes `data()`, `data.loading`, `data.error`, `data.latest`, plus `mutate` and `refetch`.

```tsx
// Bad
createEffect(async () => setUser(await fetchUser(userId())));

// Good
const [user, { mutate, refetch }] = createResource(userId, fetchUser);
```

## Gate Fetching Through the Source

The source is reactive: when it changes, the fetcher re-runs with the new value. When the source is `null`, `undefined`, or `false`, the fetcher is skipped; that is the idiomatic "don't fetch until ready".

```tsx
const [user] = createResource(() => session()?.userId, fetchUser);
```

Throw from the fetcher on non-ok responses so errors reach `ErrorBoundary` and `data.error`.

## Compose `ErrorBoundary` Outside, `Suspense` Inside

Reading a loading resource under a `Suspense` shows the fallback; a fetcher error propagates to the nearest `ErrorBoundary`.

```tsx
<ErrorBoundary fallback={<p>Couldn't load recipes.</p>}>
  <Suspense fallback={<p>Loading…</p>}>
    <For each={recipes()}>{(r) => <Recipe recipe={r} />}</For>
  </Suspense>
</ErrorBoundary>
```

Without Suspense, render states explicitly with `<Show when={data.loading}>` and `<Match when={data.error}>`.

## `mutate` for Optimistic Updates, `refetch` for Reload

```tsx
mutate((posts) => [...(posts ?? []), newPost]); // optimistic
await refetch();                                 // manual reload

const timer = setInterval(refetch, 30_000);      // polling
onCleanup(() => clearInterval(timer));
```

## In Routed Apps, Prefer `query` + `createAsync`

With solid-router (and SolidStart), `query(fn, "key")` deduplicates and caches; `createAsync(() => getRecipes())` consumes it as a fine-grained async signal; a route `preload` starts fetching during navigation before the component renders. This is the modern replacement for many `createResource` uses in routed apps.

```tsx
import { createAsync, query } from "@solidjs/router";

const getRecipes = query(async () => {
  const res = await fetch("/api/recipes");
  if (!res.ok) throw new Error("failed to fetch recipes");
  return res.json() as Promise<Recipe[]>;
}, "recipes");

const Recipes: Component = () => {
  const recipes = createAsync(() => getRecipes());
  return <For each={recipes()}>{(r) => <Recipe recipe={r} />}</For>;
};
```
