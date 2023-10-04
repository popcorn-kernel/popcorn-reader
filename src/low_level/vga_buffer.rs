use crate::print;
use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x00,
    Blue = 0x01,
    Green = 0x02,
    Cyan = 0x03,
    Red = 0x04,
    Magenta = 0x05,
    Brown = 0x06,
    LighGrey = 0x07,
    DarkGrey = 0x08,
    LightBlue = 0x09,
    LightGreen = 0x0A,
    LightCyan = 0x0B,
    LightRed = 0x0C,
    LightMagenta = 0x0D,
    Yellow = 0x0E,
    White = 0x0F,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        Self::generate(foreground as u8, background as u8)
    }
    fn generate(foreground: u8, background: u8) -> ColorCode {
        ColorCode((background) << 4 | (foreground))
    }
    fn get_colors(&self) -> (u8, u8) {
        (self.0 % 16u8, self.0 >> 4u8)
    }
    fn invert(&mut self) {
        let colors = self.get_colors();
        *self = Self::generate(colors.1, colors.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii_character: u8,
    color_code: ColorCode,
}
impl Char {
    fn invert_colors(&mut self) {
        self.color_code.invert();
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const ACTUAL_BUFFER_WIDTH: usize = 50;
//Added because input stopped working after user tried to enter the 51 character.
//Probably qemu issue, maybe there is a way, but this is the temporary fix
#[repr(transparent)]
struct Buffer {
    chars: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn move_cursor(&mut self, column_position: usize) {
        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position + 1].invert_colors();
        if column_position == 0 {
            self.next_line();
        } else {
            self.column_position = column_position;
        }
        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position + 1].invert_colors();
    }
    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' || self.column_position >= ACTUAL_BUFFER_WIDTH {
            self.move_cursor(0);
            return;
        }
        self.move_cursor(self.column_position + 1);
        self.set_char(byte);
    }
    fn set_char(&mut self, byte: u8) {
        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position] = Char {
            ascii_character: byte,
            color_code: self.color_code,
        };
    }
    fn next_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            self.buffer.chars[row - 1] = self.buffer.chars[row]
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    pub fn clear_screen(&mut self, color: Color) {
        let blank = Char {
            ascii_character: b' ',
            color_code: ColorCode::new(color, color),
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = blank;
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }

    fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
    pub fn backspace(&mut self) {
        if self.column_position == 0 {
            return;
        }
        self.set_char(b' ');
        self.move_cursor(self.column_position - 1);
    }
    fn cursor_back(&mut self) {
        if self.column_position == 0 {
            return;
        }
        self.move_cursor(self.column_position - 1)
    }
    fn cursor_front(&mut self) {
        if self.column_position == ACTUAL_BUFFER_WIDTH {
            return;
        }
        self.move_cursor(self.column_position + 1)
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background);
    });
}

pub fn clear_screen(color: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen(color);
    });
}

pub fn backspace() {
    interrupts::without_interrupts(|| {
        WRITER.lock().backspace();
    });
}
pub fn cursor_back() {
    interrupts::without_interrupts(|| {
        WRITER.lock().cursor_back();
    });
}
pub fn cursor_front() {
    interrupts::without_interrupts(|| {
        WRITER.lock().cursor_front();
    });
}
