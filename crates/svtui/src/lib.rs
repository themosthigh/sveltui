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
    close_all_submenus, close_submenu, close_topmost_submenu, has_open_submenus, is_menu_item,
    is_submenu_open, open_submenu, register_menu_item,
};
pub use element::{
    BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, OverlayElement, TextElement,
    VNode, is_focused, set_focused, use_focus,
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
        close_all_submenus, close_submenu, close_topmost_submenu, has_open_submenus, is_menu_item,
        is_submenu_open, open_submenu, register_menu_item,
    };
    pub use crate::element::{
        BoxElement, FocusHandle, FocusId, FocusState, HitMap, IntoNode, OverlayElement,
        TextElement, VNode, is_focused, set_focused, use_focus,
    };
    pub use crate::layout::{
        Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarChars,
        ScrollbarStyle, ScrollbarVisibility, Size,
    };
    pub use crate::reactive::{ReadSignal, WriteSignal, create_effect, create_signal};
    pub use crate::terminal::TerminalColor as Color;
    pub use crate::view;
}
