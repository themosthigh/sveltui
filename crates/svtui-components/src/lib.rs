pub mod badge;
pub mod button;
pub mod checkbox;
pub mod divider;
pub mod progress_bar;
pub mod switch;
pub mod text_buffer;
pub mod text_input;

pub use badge::Badge;
pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use divider::Divider;
pub use progress_bar::ProgressBar;
pub use switch::Switch;
pub use text_buffer::TextBuffer;
pub use text_input::TextInput;

#[cfg(test)]
mod tests {
    use super::*;
    use svtui_element::{IntoNode, measure_node};

    #[test]
    fn test_button_measurement() {
        let btn = Button::new("Click Me");
        let node = btn.into_node();
        let (w, h) = measure_node(&node, 80, 24);
        assert!(w >= 10);
        assert_eq!(h, 3); // 1 row text + 2 rows border
    }

    #[test]
    fn test_badge_measurement() {
        let badge = Badge::new("OK");
        let node = badge.into_node();
        let (w, h) = measure_node(&node, 80, 24);
        assert!(w >= 4);
        assert_eq!(h, 3);
    }
}
