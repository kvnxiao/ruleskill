---
paths: **/*.{tsx,jsx}
description: "Component-file conventions for SolidJS; one exported component per file, typed Component const declarations from solid-js, and <ComponentName>Props naming."
---

# Component Conventions

These standards govern component files. Framework mechanics (props reactivity, children, and component types) live in the other rules.

## One Exported Component Per File

Each `.tsx`/`.jsx` file exports at most one component. Unexported helper subcomponents may live in the same file. When a helper is needed elsewhere, move it to its own file instead of exporting a second component.

```tsx
// Bad: exports two components from one file.
export const UserCard: Component<UserCardProps> = (props) => { /* … */ };
export const UserAvatar: Component<UserAvatarProps> = (props) => { /* … */ };

// Good: UserAvatar.tsx exports UserAvatar; helpers stay private.
interface InitialsProps {
  initials: string;
}

const Initials: VoidComponent<InitialsProps> = (props) => (
  <span>{props.initials}</span>
);

export interface UserAvatarProps {
  user: User;
}

export const UserAvatar: Component<UserAvatarProps> = (props) => { /* … */ };
```

## Declare Components as Typed Consts

Declare components as `const PascalCase: Component<Props> = (props) => …` using the component types from `"solid-js"`, not as plain function declarations. The typed const identifies the component while you scan the file, and its annotation states the children contract: `Component` for no expected children, `ParentComponent` for optional children, and `VoidComponent` to forbid them (see the TypeScript rules).

```tsx
// Bad: the function declaration does not state the children contract.
export function UserCard(props: UserCardProps) {
  return <div class="card">{props.user.name}</div>;
}

// Good: exports one component and keeps helpers private.
import type { Component } from "solid-js";

export interface UserCardProps {
  user: User;
}

export const UserCard: Component<UserCardProps> = (props) => (
  <div class="card">{props.user.name}</div>
);
```

## Name Props `<ComponentName>Props`

The props type is the component name in PascalCase suffixed with `Props`, whether declared as an interface or a type alias. Generic names hide which component a type belongs to and collide when files merge.

```tsx
// Bad: the props type does not identify its component.
interface Props {
  user: User;
}
type CardData = { user: User };

// Good: names the props type after the component.
interface UserCardProps {
  user: User;
}

export const UserCard: Component<UserCardProps> = (props) => { /* … */ };
```
