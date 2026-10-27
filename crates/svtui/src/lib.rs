extern crate self as svtui;

pub mod app;
pub mod watcher;

// Re-export sub-crates
pub use svtui_element as element;
pub use svtui_layout as layout;
pub use svtui_reactive as reactive;
pub use svtui_terminal as terminal;

// Re-export proc-macro
pub use svtui_macros::view;

// Re-export common types
pub use app::App;
pub use element::{BoxElement, TextElement, IntoNode, VNode};
pub use layout::{Align, BorderStyle, Justify, Padding, Size};
pub use reactive::{create_effect, create_signal, ReadSignal, Runtime, WriteSignal};
pub use terminal::{Buffer, Cell, Terminal, TerminalColor as Color};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::element::{BoxElement, TextElement, IntoNode, VNode};
    pub use crate::layout::{Align, BorderStyle, Justify, Padding, Size};
    pub use crate::reactive::{create_effect, create_signal, ReadSignal, WriteSignal};
    pub use crate::terminal::TerminalColor as Color;
    pub use crate::view;
}
