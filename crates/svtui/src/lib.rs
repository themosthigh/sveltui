//! Declarative terminal UI with opt-in keyboard interaction.
//!
//! Plain layout boxes, IDs, and pointer handlers do not create Tab stops.
//! Built-in controls explicitly declare focusability, activation and semantics
//! using the same core APIs available to custom components:
//!
//! ```
//! use svtui::prelude::*;
//! let control = view! {
//!     <FlexBox tab_stop=true activation=Activation::Press
//!         semantics={Semantics::new(Role::Button, "Custom action")}
//!         on_click={|| { /* perform the action */ }}>
//!         <TextSpan>"Custom action"</TextSpan>
//!     </FlexBox>
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
pub use svtui_element as element;
pub use svtui_layout as layout;
pub use svtui_reactive as reactive;
pub use svtui_terminal as terminal;

// Re-export proc-macros
pub use svtui_macros::{Props, component, view};

// Re-export common types
pub use app::App;
pub use element::collection::*;
pub use element::interaction::*;
pub use element::text_buffer::*;
pub use element::{
    ContextProvider, ContextProviderProps, ErrorBoundary, ErrorBoundaryProps, FlexBox,
    FlexBoxElement, FlexBoxProps, FocusHandle, FocusId, FocusState, HitMap, IntoClickHandler,
    IntoClickHandlerAt, IntoKeyHandler, IntoNode, IntoPropOption, IntoScrollHandler,
    OptionSnippetExt, Overlay, OverlayElement, OverlayProps, Paragraph, ParagraphProps, Snippet,
    TextElement, TextSpan, TextSpanProps, VNode, is_focused, is_layer_open, use_focus,
};
pub use layout::{
    Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarChars,
    ScrollbarStyle, ScrollbarVisibility, Size,
};
pub use reactive::{
    AsyncState, BlockingFuture, ComponentScope, Derived, ReadSignal, Runtime, State, TaskHandle,
    WriteSignal, batch, begin_frame, create_effect, create_signal, derived, effect, end_frame,
    on_destroy, on_mount, provide_context, reset_hook_counter, spawn_blocking, spawn_task, state,
    state_with, use_async_state, use_context, use_hook,
};
pub use terminal::{Buffer, Cell, Terminal, TerminalColor as Color};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::element::collection::*;
    pub use crate::element::interaction::*;
    pub use crate::element::text_buffer::*;
    pub use crate::element::{
        ContextProvider, ContextProviderProps, ErrorBoundary, ErrorBoundaryProps, FlexBox,
        FlexBoxElement, FlexBoxProps, FocusHandle, FocusId, FocusState, HitMap, IntoClickHandler,
        IntoClickHandlerAt, IntoKeyHandler, IntoNode, IntoPropOption, IntoScrollHandler,
        OptionSnippetExt, Overlay, OverlayElement, OverlayProps, Paragraph, ParagraphProps,
        Snippet, TextElement, TextSpan, TextSpanProps, VNode, is_focused, is_layer_open, use_focus,
    };
    pub use crate::layout::{
        Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarChars,
        ScrollbarStyle, ScrollbarVisibility, Size,
    };
    pub use crate::reactive::{
        AsyncState, BlockingFuture, ComponentScope, Derived, ReadSignal, State, TaskHandle,
        WriteSignal, batch, begin_frame, create_effect, create_signal, derived, effect, end_frame,
        on_destroy, on_mount, provide_context, reset_hook_counter, spawn_blocking, spawn_task,
        state, state_with, use_async_state, use_context, use_hook,
    };
    pub use crate::terminal::TerminalColor as Color;
    pub use crate::{Props, component, view};
}
