---
paths: **/*.{ts,tsx,js,jsx}
description: "House architecture for separating business logic from views; state modules own business state, exported accessors and actions, private setters, and components keep only UI-local state."
---

# State Architecture

Business state and the logic that mutates it live in dedicated state modules, not in components. Components are thin views: they read exported accessors, call exported actions, and hold only UI-local state. Solid makes this native — signals, stores, and memos work at module scope under `createRoot` — so no third-party state library is needed to get Jotai-style separation.

## State Modules Own Business Logic

One domain per module (for example `src/state/cart.ts`, or the feature folder's `state.ts`). The module owns the store, derived values, and every mutation, and exports read accessors plus named action functions.

```ts
// src/state/cart.ts
import { createMemo, createRoot } from "solid-js";
import { createStore, produce } from "solid-js/store";

interface CartState {
  items: CartItem[];
}

function createCart() {
  const [state, setState] = createStore<CartState>({ items: [] });

  const itemCount = createMemo(() => state.items.length);
  const total = createMemo(() =>
    state.items.reduce((sum, item) => sum + item.price * item.quantity, 0),
  );

  function addItem(item: CartItem) {
    setState(
      produce((s) => {
        const existing = s.items.find((i) => i.id === item.id);
        if (existing) existing.quantity += 1;
        else s.items.push({ ...item, quantity: 1 });
      }),
    );
  }

  async function checkout() {
    const order = await api.checkout(state.items);
    setState("items", []);
    return order;
  }

  return { items: () => state.items, itemCount, total, addItem, checkout };
}

export const cart = createRoot(createCart);
```

## Never Export the Setter

The exported actions are the entire write API. Exporting `setState` (or a signal's setter) lets any component mutate business state arbitrarily, which destroys the invariants the module exists to hold and makes writes impossible to trace. Every mutation should be a named domain verb that is grep-able and testable.

```ts
// Bad: any component can now write anything
export const [cartState, setCartState] = createStore<CartState>({ items: [] });

// Good: writes only happen through domain verbs
export const cart = createRoot(createCart);
```

## Components Stay Dumb

A component that fetches, derives, and mutates business state fuses the view and the viewmodel; it can only be tested by rendering it, and the logic cannot be reused. Components should read accessors, call actions, and render.

```tsx
// Bad: view and business logic fused in the component
const Cart: Component = () => {
  const [items, setItems] = createSignal<CartItem[]>([]);
  const total = () =>
    items().reduce((sum, item) => sum + item.price * item.quantity, 0);
  const applyCoupon = async (code: string) => {
    const discount = await api.validateCoupon(code);
    setItems(items().map((i) => ({ ...i, price: i.price * discount })));
  };
  /* … */
};

// Good: thin view over the cart module
const Cart: Component = () => (
  <section>
    <For each={cart.items()}>{(item) => <CartRow item={item} />}</For>
    <output>{cart.total()}</output>
    <button onClick={() => cart.checkout()}>Checkout</button>
  </section>
);
```

## Local State Is UI State Only

`createSignal` inside a component is for state that exists only for that view: open/collapsed flags, hover and focus, in-progress input drafts, transient animation state. The litmus test: if the value must survive unmount or navigation, if another component needs it, or if a business rule reads or constrains it, it belongs in a state module.

```tsx
const CouponField: Component = () => {
  const [draft, setDraft] = createSignal("");        // UI-local: fine
  return (
    <form onSubmit={() => cart.applyCoupon(draft())}>
      <input value={draft()} onInput={(e) => setDraft(e.currentTarget.value)} />
    </form>
  );
};
```

## Derive in the Module, Not the View

Business derivations (totals, filters, validity) are exported memos or accessors on the module. A component that recomputes them re-encodes business rules in the view, and sibling views drift apart.

## Async Belongs to Actions

Server mutations, optimistic updates, and rollback logic live inside actions (`checkout` above). For reads, `createResource`/`createAsync` sources can live in the state module under the same root, or at route level via `query` + `preload`; either way the component only renders the result.

## SSR Instantiates via Context

A module-level singleton is the default for client-only apps. Under SSR it leaks state across requests, so keep the same `createCart` factory but instantiate it in a provider and expose it through a throwing accessor hook (see the stores and state rules); components are unaffected because they only ever see the returned API shape.

## Test Modules Without Rendering

This split is what makes business logic testable as plain TypeScript: call `createCart()` inside `createRoot` in a test, drive actions, and assert on accessors — no component render, no DOM. Component tests then only need to cover wiring and presentation.
