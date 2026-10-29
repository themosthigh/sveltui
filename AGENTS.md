# AGENTS.md — Guidelines for AI Coding Agents

Welcome to the **sveltui** codebase. This document outlines architectural rules, workflows, and mandatory verification steps for any AI coding agent modifying or extending this repository.

---

## 1. Mandatory Code Verification (Build, Lint & Test)

Before concluding any work or reporting back to the user, **you MUST verify that your changes build cleanly, pass all lint checks without warnings, conform to standard formatting, and pass all unit tests.**

### Quick Verification Command

Run the full verification pipeline:
```bash
make verify
```
This single command executes:
1. `make fmt-check` (`cargo fmt --check`)
2. `make check` (`cargo check --workspace --all-targets`)
3. `make lint` (`cargo clippy --workspace --all-targets -- -D warnings`)
4. `make test` (`cargo test --workspace`)

### Individual Verification Commands

If any step in `make verify` fails, use the individual commands below to diagnose and resolve issues:

| Command | Tool | Purpose |
|---|---|---|
| `make check` | `cargo check --workspace --all-targets` | Type-checks all crates, tests, and examples |
| `make lint` | `cargo clippy --workspace --all-targets -- -D warnings` | Catches code quality and idiomatic Rust issues |
| `make fmt-check` | `cargo fmt --check` | Validates indentation and formatting |
| `make fmt` | `cargo fmt` | Automatically fixes code formatting |
| `make test` | `cargo test --workspace` | Runs all unit and integration tests |
| `make build` | `cargo build --workspace --all-targets` | Full debug compilation of all crates and examples |

> [!IMPORTANT]
> **Zero Warnings Policy**: All changes must compile with **0 compiler warnings** and **0 Clippy warnings** under `-D warnings`. Never ignore or suppress warnings without explicit justification.

---

## 2. Workspace & Architecture Overview

`sveltui` is a declarative, reactive Terminal User Interface (TUI) framework built from scratch on top of `crossterm`. It is structured as a Cargo workspace:

```
sveltui/
├── Cargo.toml                  # Workspace root manifest
├── Makefile                    # Standard developer & agent targets
├── AGENTS.md                   # Instructions and standards for agents
├── crates/
│   ├── sveltui/                  # Main facade crate, App runtime, file watcher, prelude
│   ├── sveltui-components/       # Built-in components (Button, TextInput, Switch, Checkbox, etc.)
│   ├── sveltui-element/          # VNodes, FlexBoxElement, TextElement, layout execution, hit testing, focus
│   ├── sveltui-layout/           # Flex layout, BorderStyle, Padding, Size, Rect geometry
│   ├── sveltui-macros/           # Declarative JSX macro (`view! { ... }`)
│   ├── sveltui-reactive/         # Fine-grained reactive signals (`create_signal`, `Runtime`)
│   └── sveltui-terminal/         # Double-buffering, terminal rendering, raw mode
└── examples/                   # Framework and component examples
```

---

## 3. Core Development Principles

### 3.1. Pure JSX Syntax for Components
- Components are instantiated via JSX syntax directly inside `view! { ... }`:
  ```rust
  view! {
      <FlexBox border=BorderStyle::Single title=" Dialog ">
          <TextInput value=username set_value=set_username placeholder="User..." />
          <Divider title="Actions" />
          <Button variant=ButtonVariant::Primary on_click={move || submit()}>
              "Submit"
          </Button>
      </FlexBox>
  }
  ```
- **Do not** use raw block expressions (`{ Component::new(...).into_node() }`) for components when standard JSX tags (`<Component ... />`) are available.
- All components in `sveltui-components` implement:
  - `Default` (required for macro tag instantiation)
  - Builder setters returning `Self`
  - `IntoNode` (rendering into `VNode`)
  - `.child(...)` / `.label(...)` for child text or elements where appropriate.

### 3.2. Conservative "Base-UI" Aesthetic
- Stick to clean, minimal monochrome styling:
  - Foreground: `Color::White`
  - Secondary/Hint/Unfocused: `Color::DarkGrey`
  - Borders: `BorderStyle::Single` (unfocused/default), `BorderStyle::Double` (focused/active)
  - Avoid loud, multi-colored backgrounds unless specifically requested by the user.

### 3.3. Opt-in DOM-Like Focus Model
- Interactive elements register hit targets with a stable `FocusId`.
- Focus querying uses `use_focus(id)`:
  - `focus.is_focused()`: Element currently has keyboard focus.
  - `focus.has_focus_within()`: Element or any of its nested children contains focus (`:focus-within`).
  - `focus.is_active()`: Either focused or contains focus.
- Per-frame focus IDs are stabilized via `reset_focus_counter()` called before each render pass in `App`.

### 3.4. Signal-Driven Reactivity
- Signals are created using `let (read, write) = create_signal(val)`.
- Updates use `write.set(new_val)` or `write.update(|val| *val += 1)`.
- Triggering a write automatically marks the runtime dirty (`Runtime::set_dirty(true)`), which schedules an instant double-buffered terminal repaint on the next frame.

---

## 4. Git Commit Guidelines for AI Agents

When authoring or executing git commits as an AI agent, you must follow these standards:

- **100% AI-Generated Commit Prefix**: Any commit that is 100% AI generated **must** use the prefix `evil-` before the conventional commit type:
  - `evil-feat: ...`
  - `evil-fix: ...`
  - `evil-refactor: ...`
  - `evil-chore: ...`
  - `evil-docs: ...`
- **Subject Line Length**: Prefer ~50 characters for the commit subject line (keep concise and actionable).
- **Body / Description Wrap**: Wrap commit description/body text at a maximum of **72 characters** per line. Break to the next line when exceeded.
