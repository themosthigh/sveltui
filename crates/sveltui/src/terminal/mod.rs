pub mod backend;
pub mod buffer;
pub mod cell;

pub use backend::Terminal;
pub use buffer::Buffer;
pub use cell::Cell;
pub use crossterm::style::Color as TerminalColor;
