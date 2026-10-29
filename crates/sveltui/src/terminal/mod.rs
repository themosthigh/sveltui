pub mod backend;
pub mod buffer;
pub mod cell;

pub use backend::{Terminal, write_buffer_diff};
pub use buffer::Buffer;
pub use cell::Cell;
pub use crossterm::style::Color as TerminalColor;
