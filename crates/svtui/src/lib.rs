//! Declarative terminal UI with opt-in keyboard interaction.
//!
//! Plain layout boxes, IDs, and pointer handlers do not create Tab stops.
//! Built-in controls explicitly declare focusability, activation and semantics
//! using the same core APIs available to custom components:
//!
//! ```
//! use svtui::prelude::*;
//! let control = view! {
//!     <Box tab_stop=true activation=Activation::Press
//!         semantics={Semantics::new(Role::Button, "Custom action")}
//!         on_click={|| { /* perform the action */ }}>
//!         <Text>"Custom action"</Text>
//!     </Box>
//! };
//! ```
//!
//! Keyboard listeners return `EventResult::Handled` or `Ignored` and receive an
//! `EventContext` for queued focus, layer and exit commands. Ignored events
//! bubble through logical owners; modal scopes block the background. A focus
//! group has one external Tab stop and navigates its members with arrows.
//!
//! `App` reserves no exit or save keys. Explicitly opt into the example policy
//! with `.on_key(svtui::app::exit_shortcuts)`, or supply your own handler. Every
//! key, including Ctrl+C, reaches the focused control and its scopes first.
//!
//! Dialogs and popovers contain Tab and Shift+Tab until dismissed. Menus close
//! on Tab. Popover/Menu state is managed by the core by default;
//! supplying `.open(bool)` makes it controlled, with `.on_open`/`.on_close`
//! callbacks for state requests. Rejected dismissals retain popup focus.
//!
//! Allocate persistent `FocusId`s outside the view for reordered controls and
//! layers; pass both `id` and `layer_id` to dynamic submenus. Static JSX uses
//! deterministic frame-local IDs. `use_focus` reads the currently bound App;
//! standalone renderers/tests bind a `FocusManager::with_context` explicitly.
//!
//! Migration: global focus setters and submenu registries were removed. Use
//! manager/event-context commands, core layer profiles, and explicit Tab stops.
//! Handlers now take `(KeyEvent, &mut EventContext)` and return `EventResult`.
//! Semantic snapshots are available through the interaction tree; platform
//! screen-reader adapters are not included.
//!
//! Run `cargo run --example keyboard_scopes` for nested overlays and a core-only
//! custom control. Collection/grid/tree selection helpers are available for
//! composing future widgets; this release migrates the existing component set.

extern crate self as svtui;

pub mod app;
pub mod watcher;

// Re-export sub-crates
pub use svtui_components as components;
pub use svtui_element as element;
pub use svtui_layout as layout;
pub use svtui_reactive as reactive;
pub use svtui_terminal as terminal;

// Re-export proc-macro
pub use svtui_macros::view;

// Re-export common types
pub use app::App;
pub use components::{
    Badge, Button, ButtonVariant, Checkbox, Dialog, Divider, DropdownItem, DropdownMenu,
    DropdownSubmenu, Popover, ProgressBar, Switch, TextArea, TextBuffer, TextInput,
};
pub use element::collection::*;
pub use element::interaction::*;
pub use element::{
    BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, OverlayElement, TextElement,
    VNode, is_focused, use_focus,
};
pub use layout::{
    Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarChars,
    ScrollbarStyle, ScrollbarVisibility, Size,
};
pub use reactive::{ReadSignal, Runtime, WriteSignal, create_effect, create_signal};
pub use terminal::{Buffer, Cell, Terminal, TerminalColor as Color};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::components::{
        Badge, Button, ButtonVariant, Checkbox, Dialog, Divider, DropdownItem, DropdownMenu,
        DropdownSubmenu, Popover, ProgressBar, Switch, TextArea, TextBuffer, TextInput,
    };
    pub use crate::element::collection::*;
    pub use crate::element::interaction::*;
    pub use crate::element::{
        BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, OverlayElement,
        TextElement, VNode, is_focused, use_focus,
    };
    pub use crate::layout::{
        Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarChars,
        ScrollbarStyle, ScrollbarVisibility, Size,
    };
    pub use crate::reactive::{ReadSignal, WriteSignal, create_effect, create_signal};
    pub use crate::terminal::TerminalColor as Color;
    pub use crate::view;
}
