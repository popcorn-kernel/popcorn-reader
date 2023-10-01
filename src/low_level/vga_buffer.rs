use core::fmt;
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
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

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
    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' || self.column_position >= BUFFER_WIDTH {
            self.next_line();
            return;
        }

        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position] = Char {
            ascii_character: byte,
            color_code: self.color_code,
        };
        self.column_position += 1;
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

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::low_level::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// Sets the foreground and background color for the VGA text buffer.
/// Accepts two `Color` enum values.
/// See the `Color` enum for available colors.
/// The first argument is the foreground color, the second is the background color.
#[macro_export]
macro_rules! set_color {
    ($foreground:expr, $background:expr) => {
        $crate::low_level::vga_buffer::_set_color($foreground, $background)
    };
}

/// Clears the screen with the given color.
/// Accepts a `Color` enum value.
/// See the `Color` enum for available colors.
#[macro_export]
macro_rules! clear_screen {
    ($clear_color:expr) => {
        $crate::low_level::vga_buffer::_clear_screen($clear_color);
    };
}

/**
 *  \brief Sets the foreground and background color for the VGA text buffer.
 *  This function is called by the `set_color!` macro.
 *  \param foreground The foreground color.
 *  \param background The background color.
 */
#[doc(hidden)]
pub fn _set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background);
    });
}

#[doc(hidden)]
pub fn _clear_screen(color: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen(color);
    });
}