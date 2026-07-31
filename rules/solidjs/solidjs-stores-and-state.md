---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS state rules; signals vs createStore, path setters, produce, reconcile for server snapshots, context patterns, and global state ownership."
---

# Stores and State

## Signals for Values, Stores for Structures

Use `createSignal` for independent single values and `createStore` for nested objects and arrays. Store property access is tracked per property, so updating one field notifies only readers of that field. One giant signal holding a large object updates every reader on any change; that destroys fine-grained reactivity.

## Never Destructure a Store

Stores are proxies; destructuring reads the value once and severs reactivity, exactly like props. Access properties at the point of use inside tracking scopes.

```tsx
// Bad
const { name } = store.user;

// Good
<div>{store.user.name}</div>
```

## Write Through the Setter

Never mutate the store object directly. Use path syntax for targeted updates and `produce` for multi-field or array-mutation updates.

```tsx
const [state, setState] = createStore({ users: [], count: 0 });

// Path syntax
setState("users", 0, "loggedIn", true);
setState("users", (users) => [...users, newUser]);

// produce: localized mutation for compound updates
setState(produce((s) => {
  s.count += 1;
  s.users.push(newUser);
}));
```

## Reconcile Server Snapshots

When replacing store data wholesale (typically a server response), wrap it in `reconcile` so unchanged parts keep identity and only real changes propagate. Items match by `id` by default; pass `key` for a different field.

```tsx
setState("todos", reconcile(fetchedTodos));
```

## Context: Provider-Created State, Throwing Accessor

Create the signals or store inside the provider component, and expose a hook that throws when the provider is missing. A `createContext` default silently masks a missing provider.

```tsx
const CounterContext = createContext<CounterValue>();

export function CounterProvider(props: ParentProps) {
  const [count, setCount] = createSignal(0);
  return (
    <CounterContext.Provider value={{ count, setCount }}>
      {props.children}
    </CounterContext.Provider>
  );
}

export function useCounter() {
  const ctx = useContext(CounterContext);
  if (!ctx) throw new Error("useCounter must be used within CounterProvider");
  return ctx;
}
```

## Global State Needs a Root

Module-level singletons are acceptable in client-only Solid, but computations created at module scope leak without an owner; wrap them in `createRoot`. Use context instead of module state when per-subtree instances are needed or the app server-renders, because module state leaks across SSR requests.

```tsx
export const counter = createRoot(() => {
  const [count, setCount] = createSignal(0);
  return { count, setCount };
});
```
