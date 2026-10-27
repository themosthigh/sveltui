pub mod buffer;
pub mod cell;
pub mod terminal;

pub use buffer::Buffer;
pub use cell::Cell;
pub use crossterm::style::Color as TerminalColor;
pub use terminal::Terminal;
