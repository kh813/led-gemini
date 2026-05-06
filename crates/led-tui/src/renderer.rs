use crossterm::{
    cursor,
    style::{self, Color, ContentStyle},
    QueueableCommand,
};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            underline: false,
        }
    }
}

pub struct Renderer {
    width: u16,
    height: u16,
    prev_buffer: Vec<Cell>,
    curr_buffer: Vec<Cell>,
}

impl Renderer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            prev_buffer: vec![Cell::default(); size],
            curr_buffer: vec![Cell::default(); size],
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.prev_buffer = vec![Cell::default(); size];
        self.curr_buffer = vec![Cell::default(); size];
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.curr_buffer[idx] = cell;
        }
    }

    pub fn clear(&mut self) {
        for cell in self.curr_buffer.iter_mut() {
            *cell = Cell::default();
        }
    }

    pub fn present<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let mut last_style = ContentStyle::default();
        let mut cursor_moved = false;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let curr = self.curr_buffer[idx];
                let prev = self.prev_buffer[idx];

                if curr != prev {
                    if !cursor_moved {
                        writer.queue(cursor::MoveTo(x, y))?;
                        cursor_moved = true;
                    }

                    let mut style = ContentStyle::default();
                    style.foreground_color = Some(curr.fg);
                    style.background_color = Some(curr.bg);
                    
                    if curr.bold {
                        style.attributes.set(style::Attribute::Bold);
                    }
                    if curr.underline {
                        style.attributes.set(style::Attribute::Underlined);
                    }

                    if style != last_style {
                        writer.queue(style::SetStyle(style))?;
                        last_style = style;
                    }

                    writer.queue(style::Print(curr.ch))?;
                    
                    // After printing a character, the cursor moves forward.
                    // If the next cell also needs update, we don't need MoveTo.
                    // But if there's a gap, we'll need MoveTo for the next changed cell.
                } else {
                    cursor_moved = false;
                }
            }
            cursor_moved = false; // Reset for next line
        }

        self.prev_buffer.copy_from_slice(&self.curr_buffer);
        writer.flush()?;
        Ok(())
    }
}
