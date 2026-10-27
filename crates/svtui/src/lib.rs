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
    Badge, Button, ButtonVariant, Checkbox, Divider, ProgressBar, Switch, TextInput,
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
        Badge, Button, ButtonVariant, Checkbox, Divider, ProgressBar, Switch, TextInput,
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

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_jsx_custom_components_expansion() {
        let (val, set_val) = create_signal(String::from("Hello"));
        let (checked, set_checked) = create_signal(true);

        let _node: VNode = view! {
            <Box
                min_width=20
                max_width=80
                min_height=5
                max_height=30
                overflow=Overflow::Scroll
                scrollbar_visibility=ScrollbarVisibility::Auto
            >
                <Button variant=ButtonVariant::Primary>"Click Me"</Button>
                <Switch checked=checked set_checked=set_checked label="Toggle" />
                <Checkbox checked=checked set_checked=set_checked label="Accept" />
                <TextInput value=val set_value=set_val placeholder="Type..." title="Input" />
                <Divider title="Separator" />
                <ProgressBar ratio=0.75 width=20 />
                <Badge color=Color::Green>"Active"</Badge>
            </Box>
        };
    }
}
