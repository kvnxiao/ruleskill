---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS server-state rules; TanStack Query with queryOptions factories in domain modules, no destructuring of query results, Suspense/ErrorBoundary composition, mutations as domain options, router loader integration, and createResource as the low-level fallback."
---

# Data Fetching

## TanStack Query Owns Server State

Server data lives in the Query cache; do not fetch it ad hoc in components or copy it into stores. Raw `fetch` in an effect loses tracking after the first `await` and does not provide race handling, caching, deduplication, or retries. `useQuery` from `@tanstack/solid-query` is the default for reads; its `data` is backed by a Solid resource, so Suspense and transitions work without additional wiring.

```tsx
// Bad: fetches data in an async effect.
createEffect(async () => setUser(await fetchUser(userId())));

// Good: use the query cache.
const user = useQuery(() => userQueryOptions(userId()));
```

## Define `queryOptions` in Domain Modules

Query keys, fetchers, and staleness policy are business decisions; define them in the domain module (see the state architecture rules) as `queryOptions` factories. Reuse the same factory in components, router loaders, and `queryClient` calls to keep query keys consistent.

```ts
// Domain module: src/state/todos.ts
import { queryOptions } from "@tanstack/solid-query";

export function todosQueryOptions(filter: TodoFilter) {
  return queryOptions({
    queryKey: ["todos", filter],
    queryFn: () => api.fetchTodos(filter),
    staleTime: 5 * 60 * 1000,
  });
}
```

## Options In as a Function, Results Out Fine-Grained

This is the pack-wide adapter convention (see the ecosystem rules). For Query, `useQuery` takes an accessor returning options; signals read inside it are tracked, and changes re-key or re-run the query. Gate dependent queries with `enabled` instead of conditional calls. The result is a fine-grained store: read `query.data`, `query.isPending`, and `query.isError` as properties inside tracking scopes, and never destructure it. Because the result is store-backed, `query.data` is a proxy; call `unwrap` before cloning, serializing, or sending it across IPC (see the stores and state rules).

```tsx
const [todo, setTodo] = createSignal(0);

const todoQuery = useQuery(() => ({
  ...todoQueryOptions(todo()),
  enabled: todo() > 0,
}));

// Bad: destructuring severs reactivity, as it does for props and stores.
const { data, isPending } = useQuery(() => todosQueryOptions("all"));
```

## Compose `ErrorBoundary` Outside, `Suspense` Inside

Reading `query.data` under a `Suspense` boundary triggers the fallback while loading. Set `throwOnError: true` to surface fetch errors to the nearest `ErrorBoundary`; otherwise render states explicitly with `<Switch>` on `isPending`/`isError`.

```tsx
<ErrorBoundary fallback={<p>Couldn't load todos.</p>}>
  <Suspense fallback={<p>Loading…</p>}>
    <For each={todos.data}>{(todo) => <TodoRow todo={todo} />}</For>
  </Suspense>
</ErrorBoundary>
```

## Mutations Are Domain Logic

`useMutation` also takes function-wrapped options. What a mutation does — the request, optimistic update, rollback, and which queries it invalidates — is business logic and belongs in the domain module via `mutationOptions`; the component only calls `mutation.mutate`.

```ts
// Domain module: src/state/todos.ts
import { mutationOptions, type QueryClient } from "@tanstack/solid-query";

export function addTodoMutationOptions(queryClient: QueryClient) {
  return mutationOptions({
    mutationFn: (todo: NewTodo) => api.addTodo(todo),
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: ["todos"] }),
  });
}
```

```tsx
const queryClient = useQueryClient();
const addTodo = useMutation(() => addTodoMutationOptions(queryClient));

<button onClick={() => addTodo.mutate(draft())}>Add</button>
```

## Integrate the Router Through the Cache

Put the `QueryClient` in router context and populate the cache in loaders with `ensureQueryData`; the component subscribes with `useQuery` on the same options factory. With `defaultPreload: "intent"`, hover and focus start fetching before navigation.

```tsx
export const Route = createFileRoute("/todos")({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(todosQueryOptions("all")),
  component: TodosPage,
});

const TodosPage: Component = () => {
  const todos = useQuery(() => todosQueryOptions("all"));
  return <For each={todos.data}>{(todo) => <TodoRow todo={todo} />}</For>;
};
```

Route hooks return accessors in the Solid adapter — `Route.useParams()`, `Route.useSearch()`, and `Route.useLoaderData()` are called as functions (`params().postId`).

## Split Code at Routes, Transition Between States

Route-level code splitting uses TanStack Router's lazy route files: move a route's component into `posts.lazy.tsx` with `createLazyFileRoute` while the loader and route config stay eager. For component-level splitting, `lazy(() => import("./HeavyEditor"))` from `solid-js` renders under the same `Suspense` boundaries as data. When a signal change swaps Suspense-bound content (tab switches or filter changes), wrap the write in `useTransition` from `solid-js` to preserve the current UI while the new content loads.

```tsx
const [pending, start] = useTransition();
<button onClick={() => start(() => setTab("stats"))} data-pending={pending()}>
  Stats
</button>
```

## `createResource` Is the Low-Level Fallback

For library code and contexts without a `QueryClient`, `createResource(source, fetcher)` remains correct: a source of `null`/`undefined`/`false` skips the fetcher, changes re-run it, and `data.loading`/`data.error` plus `mutate`/`refetch` cover local needs. It is the primitive solid-query itself builds on — use it when pulling in the full cache is unjustified, not as the default for application server state.
