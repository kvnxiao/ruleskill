---
paths: **/*.{tsx,jsx}
description: "Component-file conventions for SolidJS; one exported component per file, typed Component const declarations from solid-js, and <ComponentName>Props naming."
---

# Component Conventions

These standards govern component files. Framework mechanics (props reactivity, children, and component types) live in the other rules.

## One Exported Component Per File (Default)

Default to one exported component per `.tsx` or `.jsx` file. Keep private helpers beside the exported component, and move independently reusable components to their own files. Export tightly coupled component families or compound components together when their shared contract is clearer in one module.

```tsx
interface InitialsProps {
  initials: string;
}

const Initials: VoidComponent<InitialsProps> = (props) => (
  <span>{props.initials}</span>
);

export interface UserAvatarProps {
  user: User;
}

export const UserAvatar: Component<UserAvatarProps> = (props) => (
  <img alt={props.user.name} src={props.user.avatarUrl} />
);
```

## Declare Components as Typed Consts (Default)

Default to `const PascalCase: Component<Props> = (props) => …` when the Solid component type communicates the children contract. Use a function declaration when hoisting improves module structure or when TypeScript needs a generic component signature that `Component<Props>` cannot express cleanly.

```tsx
import type { Component } from "solid-js";

export interface UserCardProps {
  user: User;
}

export const UserCard: Component<UserCardProps> = (props) => (
  <div class="card">{props.user.name}</div>
);

export function Select<T>(props: SelectProps<T>) {
  return <For each={props.options}>{props.children}</For>;
}
```

## Name Props `<ComponentName>Props` (Default)

Name a component-owned contract `<ComponentName>Props`. When several components share a domain contract, use the domain name instead of renaming the same shape for each component. Avoid generic names such as `Props` that lose context when imported or moved.

```tsx
interface UserCardProps {
  user: User;
}

interface SelectOption {
  value: string;
  label: string;
}

export const UserCard: Component<UserCardProps> = (props) => (
  <article>{props.user.name}</article>
);
```
