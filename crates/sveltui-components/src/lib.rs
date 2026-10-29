pub mod badge;
pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod divider;
pub mod menu;
pub mod popover;
pub mod progress_bar;
pub mod switch;
pub mod text_area;
pub mod text_buffer;
pub mod text_input;

pub use badge::{Badge, BadgeProps};
pub use button::{Button, ButtonProps};
pub use checkbox::{Checkbox, CheckboxBinding, CheckboxProps};
pub use dialog::{Dialog, DialogProps};
pub use divider::{Divider, DividerProps};
/// Shared menu primitive and composable parts.
pub use menu as Menu;
#[allow(non_snake_case)]
pub mod DropdownMenu {
    pub use crate::menu::{
        Content, ContentProps, Group, GroupHeading, GroupHeadingProps, GroupProps, Item, ItemProps,
        Root, RootProps, Separator, SeparatorProps, Sub, SubContent, SubContentProps, SubProps,
        SubTrigger, SubTriggerProps, Trigger, TriggerProps,
    };
}
#[allow(non_snake_case)]
pub mod ContextMenu {
    pub use crate::menu::{
        Content, ContentProps, ContextRoot as Root, Group, GroupHeading, GroupHeadingProps,
        GroupProps, Item, ItemProps, RootProps, Separator, SeparatorProps, Sub, SubContent,
        SubContentProps, SubProps, SubTrigger, SubTriggerProps, Trigger, TriggerProps,
    };
}
#[allow(non_snake_case)]
pub mod MenuBar {
    pub use crate::menu::{
        BarMenu as Menu, BarMenuProps as MenuProps, BarRoot as Root, BarRootProps as RootProps,
        Content, ContentProps, Group, GroupHeading, GroupHeadingProps, GroupProps, Item, ItemProps,
        Separator, SeparatorProps, Sub, SubContent, SubContentProps, SubProps, SubTrigger,
        SubTriggerProps, Trigger, TriggerProps,
    };
}
pub use popover::{Popover, PopoverProps};
pub use progress_bar::{ProgressBar, ProgressBarProps};
pub use switch::{Switch, SwitchBinding, SwitchProps};
pub use text_area::{TextArea, TextAreaProps};
pub use text_buffer::*;
pub use text_input::{TextInput, TextInputBinding, TextInputProps, get_cursor, set_cursor};
