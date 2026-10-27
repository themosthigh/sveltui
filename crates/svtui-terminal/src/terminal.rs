use crate::buffer::Buffer;
use crate::cell::Cell;
use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
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
        let mut current_fg = None;
        let mut current_bg = None;
        let mut current_bold = false;
        let mut current_italic = false;
        let mut current_underline = false;

        let force_redraw = self.needs_full_repaint;
        self.needs_full_repaint = false;

        for y in 0..self.height {
            for x in 0..self.width {
                let curr_cell = self
                    .curr_buffer
                    .get(x, y)
                    .copied()
                    .unwrap_or(Cell::default());
                let prev_cell = self
                    .prev_buffer
                    .get(x, y)
                    .copied()
                    .unwrap_or(Cell::default());

                if force_redraw || curr_cell != prev_cell {
                    execute!(self.stdout, MoveTo(x, y))?;

                    if curr_cell.fg != current_fg {
                        match curr_cell.fg {
                            Some(color) => execute!(self.stdout, SetForegroundColor(color))?,
                            None => execute!(self.stdout, ResetColor)?,
                        }
                        current_fg = curr_cell.fg;
                    }

                    if curr_cell.bg != current_bg {
                        match curr_cell.bg {
                            Some(color) => execute!(self.stdout, SetBackgroundColor(color))?,
                            None => execute!(self.stdout, ResetColor)?,
                        }
                        current_bg = curr_cell.bg;
                    }

                    if curr_cell.bold != current_bold {
                        if curr_cell.bold {
                            execute!(self.stdout, SetAttribute(Attribute::Bold))?;
                        } else {
                            execute!(self.stdout, SetAttribute(Attribute::NormalIntensity))?;
                        }
                        current_bold = curr_cell.bold;
                    }

                    if curr_cell.italic != current_italic {
                        if curr_cell.italic {
                            execute!(self.stdout, SetAttribute(Attribute::Italic))?;
                        } else {
                            execute!(self.stdout, SetAttribute(Attribute::NoItalic))?;
                        }
                        current_italic = curr_cell.italic;
                    }

                    if curr_cell.underline != current_underline {
                        if curr_cell.underline {
                            execute!(self.stdout, SetAttribute(Attribute::Underlined))?;
                        } else {
                            execute!(self.stdout, SetAttribute(Attribute::NoUnderline))?;
                        }
                        current_underline = curr_cell.underline;
                    }

                    execute!(self.stdout, Print(curr_cell.ch))?;
                }
            }
        }

        execute!(self.stdout, ResetColor, SetAttribute(Attribute::Reset))?;
        self.stdout.flush()?;

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
