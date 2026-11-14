//! Declarative terminal UI with opt-in keyboard interaction.
//!
//! Plain layout boxes, IDs, and pointer handlers do not create Tab stops.
//! Built-in controls explicitly declare focusability, activation and semantics
//! using the same core APIs available to custom components:
//!
//! ```
//! use sveltui::prelude::*;
//! let control = view! {
//!     <Pressable tab_stop=true activation=Activation::Press
//!         semantics={Semantics::new(Role::Button, "Custom action")}
//!         on_click={|| { /* perform the action */ }}>
//!         <TextSpan>"Custom action"</TextSpan>
//!     </Pressable>
//! };
//! ```
//!
//! Keyboard listeners return `EventResult::Handled` or `Ignored` and receive an
//! `EventContext` for queued focus, layer and exit commands. Ignored events
//! bubble through logical owners; modal scopes block the background. A focus
//! group has one external Tab stop and navigates its members with arrows.
//!
//! `App` reserves no exit or save keys. Explicitly opt into the example policy
//! with `.on_key(sveltui::app::exit_shortcuts)`, or supply your own handler. Every
//! key, including Ctrl+C, reaches the focused control and its scopes first.
//!
//! Dialogs and popovers contain Tab and Shift+Tab until dismissed. Menus close
//! on Tab. Popover state is managed by the core by default;
//! supplying `.open(bool)` makes it controlled, with `.on_open`/`.on_close`
//! callbacks for state requests. Rejected dismissals retain popup focus.
//!
//! Allocate persistent `FocusId`s outside the view for reordered controls and
//! layers; pass IDs to both menu roots and their triggers when reordering. Static JSX uses
//! deterministic frame-local IDs. `use_focus` reads the currently bound App;
//! standalone renderers/tests bind a `FocusManager::with_context` explicitly.
//!
//! Migration: global focus setters and submenu registries were removed. Use
//! manager/event-context commands, core layer profiles, and explicit Tab stops.
//! Handlers now take `(KeyEvent, &mut EventContext)` and return `EventResult`.
//! Semantic snapshots are available through the interaction tree; platform
//! screen-reader adapters are not included.
//!
//! Run `cargo run --example component_keyboard` for nested overlays and a core-only
//! custom control. Collection/grid/tree selection helpers are available for
//! composing future widgets; this release migrates the existing component set.

extern crate self as sveltui;

pub mod app;
pub mod element;
pub mod layout;
pub mod reactive;
pub mod terminal;
pub mod watcher;

// Re-export proc-macros
pub use sveltui_macros::{Props, component, view};

// Re-export common types
pub use app::App;
pub use element::collection::*;
pub use element::interaction::*;
pub use element::text_buffer::*;
pub use element::{
    ContextProvider, ContextProviderProps, ErrorBoundary, ErrorBoundaryProps, FocusHandle, FocusId,
    FocusState, HitMap, IntoClickHandler, IntoClickHandlerAt, IntoKeyHandler, IntoNode,
    IntoPropOption, IntoScrollHandler, NodeRef, OptionSnippetExt, Overlay, OverlayElement,
    OverlayProps, Paragraph, ParagraphProps, Pressable, PressableElement, PressableProps,
    PropIsSet, PropSet, Snippet, TextElement, TextSpan, TextSpanProps, VNode, View, ViewElement,
    ViewProps, is_focused, is_layer_open, node_ref, use_focus, use_layout,
};
pub use layout::{
    AlignContent, AlignItems, AlignSelf, Display, FlexWrap, JustifyContent, Point, Position,
};
pub use layout::{
    BorderStyle, Dimension, FlexDirection, FlexDirectionExt, IntoDimension, Margin, Overflow,
    OverflowExt, Padding, Placement, ScrollbarChars, ScrollbarStyle, ScrollbarVisibility,
};
pub use reactive::{
    AsyncState, Bindable, Binding, BlockingFuture, ComponentScope, Derived, IntoBinding,
    ReadSignal, Runtime, State, TaskHandle, WriteSignal, batch, begin_frame, create_effect,
    create_signal, derived, effect, end_frame, on_destroy, on_mount, provide_context,
    reset_hook_counter, spawn_blocking, spawn_task, state, state_with, use_async_state,
    use_context, use_hook,
};
pub use terminal::{
    Buffer, Cell, CursorStyle, PointerShape, Terminal, TerminalColor as Color, hex_color,
};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::element::collection::*;
    pub use crate::element::interaction::*;
    pub use crate::element::text_buffer::*;
    pub use crate::element::{
        ContextProvider, ContextProviderProps, ErrorBoundary, ErrorBoundaryProps, FocusHandle,
        FocusId, FocusState, HitMap, IntoClickHandler, IntoClickHandlerAt, IntoKeyHandler,
        IntoNode, IntoPropOption, IntoScrollHandler, NodeRef, OptionSnippetExt, Overlay,
        OverlayElement, OverlayProps, Paragraph, ParagraphProps, Pressable, PressableElement,
        PressableProps, Snippet, TextElement, TextSpan, TextSpanProps, VNode, View, ViewElement,
        ViewProps, is_focused, is_layer_open, node_ref, use_focus, use_layout,
    };
    pub use crate::layout::{
        AlignContent, AlignItems, AlignSelf, BorderStyle, Dimension, Display, FlexDirection,
        FlexDirectionExt, FlexWrap, Inset, IntoDimension, JustifyContent, Margin, MarginEdge,
        Overflow, OverflowExt, Padding, Placement, Point, Position, ScrollbarChars, ScrollbarStyle,
        ScrollbarVisibility,
    };
    pub use crate::layout::{
        GridAutoFlow, GridLine, GridLines, Repeat, RuleAxes, Side, Track, auto, evenly,
        fit_content, fr, length, line, lines, max_content, min_content, minmax, percent, repeat,
        span,
    };
    pub use crate::reactive::{
        AsyncState, Bindable, Binding, BlockingFuture, ComponentScope, Derived, IntoBinding,
        ReadSignal, State, TaskHandle, WriteSignal, batch, begin_frame, create_effect,
        create_signal, derived, effect, end_frame, on_destroy, on_mount, provide_context,
        reset_hook_counter, spawn_blocking, spawn_task, state, state_with, use_async_state,
        use_context, use_hook,
    };
    pub use crate::terminal::{
        CursorStyle, PointerDialect, PointerShape, TerminalColor as Color, hex_color,
    };
    pub use crate::{Props, component, view};
}
