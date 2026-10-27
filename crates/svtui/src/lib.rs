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
    Badge, Button, ButtonVariant, Checkbox, Divider, ProgressBar, Switch, TextArea, TextBuffer,
    TextInput,
};
pub use element::{
    BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, TextElement, VNode, is_focused,
    set_focused, use_focus,
};
pub use layout::{
    Align, BorderStyle, Justify, Overflow, Padding, ScrollbarChars, ScrollbarStyle,
    ScrollbarVisibility, Size,
};
pub use reactive::{ReadSignal, Runtime, WriteSignal, create_effect, create_signal};
pub use terminal::{Buffer, Cell, Terminal, TerminalColor as Color};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::components::{
        Badge, Button, ButtonVariant, Checkbox, Divider, ProgressBar, Switch, TextArea, TextBuffer,
        TextInput,
    };
    pub use crate::element::{
        BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, TextElement, VNode,
        is_focused, set_focused, use_focus,
    };
    pub use crate::layout::{
        Align, BorderStyle, Justify, Overflow, Padding, ScrollbarChars, ScrollbarStyle,
        ScrollbarVisibility, Size,
    };
    pub use crate::reactive::{ReadSignal, WriteSignal, create_effect, create_signal};
    pub use crate::terminal::TerminalColor as Color;
    pub use crate::view;
}
