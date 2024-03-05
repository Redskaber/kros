//! vga_buffer: A simple implementation of a VGA text buffer

use core::fmt;
use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;

/// define the VGA Colors
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DackGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

/// define the VGA Color Code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


/// define the VGA ScreenChar 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]      // order 
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl core::ops::DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl core::ops::Deref for ScreenChar {
    type Target = ScreenChar;

    fn deref(&self) -> &Self::Target {
        self
    }
}

// buffer size
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDHT: usize = 80;
/// buffer cache
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDHT]; BUFFER_HEIGHT], // [[D: col]; row]
}

/// define write 
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDHT {
                    self.new_line();
                }
                
                let row = BUFFER_HEIGHT -1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;

            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),   // ascii
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// up move and clear bottom row
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDHT {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row -1][col].write(character);
            }
        }

        self.clear_line(BUFFER_HEIGHT-1);
        self.column_position = 0;
    }

    fn clear_line(&mut self, row: usize) {
        let blank = ScreenChar { 
            ascii_character: b' ', 
            color_code: self.color_code, 
        };
        for col in 0..BUFFER_WIDHT {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

/// used build format macro: write! and writeln!
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // debug(hardware Timer): print res deal lock 
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).expect("Printing to vga failed.");
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{$crate::vga_buffer::_print(format_args!($($arg)*));}};
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($($arg:tt)*) => {{$crate::print!("{}\n", format_args!($($arg)*));}};
}


// define global writer: spinlock and interior-mutability 
lazy_static!{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


// test part
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}