use crate::terminal::buffer::Buffer;
use crate::terminal::cell::Cell;
use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static CLEANED_UP: AtomicBool = AtomicBool::new(false);

pub struct Terminal {
    stdout: Stdout,
    pub curr_buffer: Buffer,
    pub prev_buffer: Buffer,
    width: u16,
    height: u16,
    needs_full_repaint: bool,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::new_with_mouse(true)
    }

    pub fn new_with_mouse(enable_mouse: bool) -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let mut stdout = io::stdout();

        terminal::enable_raw_mode()?;
        if enable_mouse {
            execute!(
                stdout,
                EnterAlternateScreen,
                cursor::Hide,
                EnableMouseCapture
            )?;
        } else {
            execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        }

        Self::set_cleanup_hook();

        Ok(Self {
            stdout,
            curr_buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            width,
            height,
            needs_full_repaint: true,
        })
    }

    fn set_cleanup_hook() {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            Self::restore_terminal();
            default_hook(panic_info);
        }));
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.curr_buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        self.needs_full_repaint = true;
    }

    pub fn flush(&mut self) -> io::Result<()> {
        write_buffer_diff(
            &mut self.stdout,
            &self.curr_buffer,
            &self.prev_buffer,
            self.needs_full_repaint,
        )?;
        self.needs_full_repaint = false;
        self.prev_buffer.cells.clone_from(&self.curr_buffer.cells);
        Ok(())
    }

    fn restore_terminal() {
        if !CLEANED_UP.swap(true, Ordering::SeqCst) {
            let mut stdout = io::stdout();
            let _ = execute!(
                stdout,
                ResetColor,
                SetAttribute(Attribute::Reset),
                cursor::Show,
                DisableMouseCapture,
                LeaveAlternateScreen
            );
            let _ = stdout.flush();
            let _ = terminal::disable_raw_mode();
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        Self::restore_terminal();
    }
}

// Keep terminal output independent of raw-mode setup so the actual emitted
// commands can be verified alongside the in-memory buffer.
fn write_buffer_diff(
    mut stdout: impl Write,
    curr_buffer: &Buffer,
    prev_buffer: &Buffer,
    force_redraw: bool,
) -> io::Result<()> {
    let mut current_fg = None;
    let mut current_bg = None;
    let mut current_bold = false;
    let mut current_italic = false;
    let mut current_underline = false;

    for y in 0..curr_buffer.height {
        for x in 0..curr_buffer.width {
            let curr_cell = curr_buffer.get(x, y).copied().unwrap_or(Cell::default());
            let prev_cell = prev_buffer.get(x, y).copied().unwrap_or(Cell::default());

            if force_redraw || curr_cell != prev_cell {
                execute!(stdout, MoveTo(x, y))?;

                if curr_cell.fg != current_fg {
                    match curr_cell.fg {
                        Some(color) => execute!(stdout, SetForegroundColor(color))?,
                        None => execute!(stdout, SetForegroundColor(Color::Reset))?,
                    }
                    current_fg = curr_cell.fg;
                }

                if curr_cell.bg != current_bg {
                    match curr_cell.bg {
                        Some(color) => execute!(stdout, SetBackgroundColor(color))?,
                        None => execute!(stdout, SetBackgroundColor(Color::Reset))?,
                    }
                    current_bg = curr_cell.bg;
                }

                if curr_cell.bold != current_bold {
                    if curr_cell.bold {
                        execute!(stdout, SetAttribute(Attribute::Bold))?;
                    } else {
                        execute!(stdout, SetAttribute(Attribute::NormalIntensity))?;
                    }
                    current_bold = curr_cell.bold;
                }

                if curr_cell.italic != current_italic {
                    if curr_cell.italic {
                        execute!(stdout, SetAttribute(Attribute::Italic))?;
                    } else {
                        execute!(stdout, SetAttribute(Attribute::NoItalic))?;
                    }
                    current_italic = curr_cell.italic;
                }

                if curr_cell.underline != current_underline {
                    if curr_cell.underline {
                        execute!(stdout, SetAttribute(Attribute::Underlined))?;
                    } else {
                        execute!(stdout, SetAttribute(Attribute::NoUnderline))?;
                    }
                    current_underline = curr_cell.underline;
                }

                execute!(stdout, Print(curr_cell.ch))?;
            }
        }
    }

    execute!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;

    if let Some((cx, cy)) = curr_buffer.cursor {
        if cx < curr_buffer.width && cy < curr_buffer.height {
            execute!(stdout, MoveTo(cx, cy), cursor::Show)?;
        } else {
            execute!(stdout, cursor::Hide)?;
        }
    } else {
        execute!(stdout, cursor::Hide)?;
    }

    stdout.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_menu_spaces_keep_the_highlight_background() {
        let mut current = Buffer::new(3, 1);
        current.set_char(0, 0, 'A', Some(Color::Black), Some(Color::White), false);
        current.set_char(1, 0, ' ', None, Some(Color::White), false);
        current.set_char(2, 0, ' ', None, Some(Color::White), false);
        let mut output = Vec::new();
        write_buffer_diff(&mut output, &current, &Buffer::new(3, 1), true).unwrap();
        let output = String::from_utf8(output).unwrap();

        // Resetting just the foreground must preserve the row's white
        // background when the label ends and unstyled padding begins.
        let padding = format!("{}{} ", MoveTo(1, 0), SetForegroundColor(Color::Reset));
        assert!(output.contains(&padding), "{output:?}");
    }

    #[test]
    fn resetting_background_preserves_the_foreground() {
        let mut current = Buffer::new(2, 1);
        current.set_char(0, 0, 'A', Some(Color::White), Some(Color::Black), false);
        current.set_char(1, 0, 'B', Some(Color::White), None, false);
        let mut output = Vec::new();
        write_buffer_diff(&mut output, &current, &Buffer::new(2, 1), true).unwrap();
        let output = String::from_utf8(output).unwrap();
        let next = format!("{}{}B", MoveTo(1, 0), SetBackgroundColor(Color::Reset));
        assert!(output.contains(&next), "{output:?}");
    }
}
