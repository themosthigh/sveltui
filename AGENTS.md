# AGENTS.md — Guidelines for AI Coding Agents

Welcome to the **svtui** codebase. This document outlines architectural rules, workflows, and mandatory verification steps for any AI coding agent modifying or extending this repository.

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

`svtui` is a declarative, reactive Terminal User Interface (TUI) framework built from scratch on top of `crossterm`. It is structured as a Cargo workspace:

```
svtui/
├── Cargo.toml                  # Workspace root manifest
├── Makefile                    # Standard developer & agent targets
├── AGENTS.md                   # Instructions and standards for agents
├── crates/
│   ├── svtui/                  # Main facade crate, App runtime, file watcher, prelude
│   ├── svtui-components/       # Built-in components (Button, TextInput, Switch, Checkbox, etc.)
│   ├── svtui-element/          # VNodes, BoxElement, TextElement, layout execution, hit testing, focus
│   ├── svtui-layout/           # Flex layout, BorderStyle, Padding, Size, Rect geometry
│   ├── svtui-macros/           # Declarative JSX macro (`view! { ... }`)
│   ├── svtui-reactive/         # Fine-grained reactive signals (`create_signal`, `Runtime`)
│   └── svtui-terminal/         # Double-buffering, terminal rendering, raw mode
└── examples/
    ├── hello_world.rs          # Minimal centered text in a single-border box (`cargo run --example hello_world`)
    ├── counter.rs              # Reactive signal counter with buttons (`cargo run --example counter`)
    ├── keyboard_listener.rs    # Real-time keypress & modifier inspector (`cargo run --example keyboard_listener`)
    ├── form.rs                 # Comprehensive form with inputs & switches (`cargo run --example form`)
    ├── interactive_components.rs # Components showcase with `:focus-within` (`cargo run --example interactive_components`)
    └── scroll_container.rs     # Scrollable box container with scrollbars (`cargo run --example scroll_container`)
```

---

## 3. Core Development Principles

### 3.1. Pure JSX Syntax for Components
- Components are instantiated via JSX syntax directly inside `view! { ... }`:
  ```rust
  view! {
      <Box border=BorderStyle::Single title=" Dialog ">
          <TextInput value=username set_value=set_username placeholder="User..." />
          <Divider title="Actions" />
          <Button variant=ButtonVariant::Primary on_click={move || submit()}>
              "Submit"
          </Button>
      </Box>
  }
  ```
- **Do not** use raw block expressions (`{ Component::new(...).into_node() }`) for components when standard JSX tags (`<Component ... />`) are available.
- All components in `svtui-components` implement:
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

---

## 5. Running Examples

Verify interactive behavior by running any example directly via Cargo:

```bash
cargo run --example hello_world            # Hello World example
cargo run --example counter                # Simple Counter example
cargo run --example keyboard_listener      # Keyboard Event Listener
cargo run --example form                   # Form with text inputs & toggles
cargo run --example interactive_components # Full Component Showcase
cargo run --example scroll_container       # Scroll Container with scrollbars
cargo run --example popover_and_dialog     # Popover, Dropdown & Modal Dialog Showcase
cargo run --example application_syntax     # Application Syntax (Svelte 5-style) Demo
```
