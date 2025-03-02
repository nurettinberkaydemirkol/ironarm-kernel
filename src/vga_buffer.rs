use volatile::Volatile;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6, 
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
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
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                 self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
                set_cursor_position(self.column_position, row);
            }
        }
    }

    
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

fn get_cursor_position() -> (usize, usize) {
    use x86_64::instructions::port::{Port};

    let mut port3d4 = Port::new(0x3D4);
    let mut port3d5 = Port::new(0x3D5);

    unsafe {
        port3d4.write(0x0F_u8);
        let low: u16 = port3d5.read();
        port3d4.write(0x0E_u8);
        let high: u16 = port3d5.read();
        let pos = ((high as u16) << 8) | (low as u16);
        (pos as usize % BUFFER_WIDTH, pos as usize / BUFFER_WIDTH)
    }
}

fn set_cursor_position(x: usize, y: usize) {
    use x86_64::instructions::port::{Port};

    let pos: u16 = (y as u16) * (BUFFER_WIDTH as u16) + (x as u16);
    let mut port3d4 = Port::new(0x3D4);
    let mut port3d5 = Port::new(0x3D5);

    unsafe {
        port3d4.write(0x0E_u8);
        port3d5.write(((pos >> 8) & 0xFF) as u8);
        port3d4.write(0x0F_u8);
        port3d5.write((pos & 0xFF) as u8);
    }
}


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: get_cursor_position().0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

use x86_64::instructions::port::Port;

pub fn enable_mouse() {
    let mut command_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);

    unsafe {
        command_port.write(0xA8_u8); 
        command_port.write(0x20_u8); 

        let status = data_port.read();
        command_port.write(0x60_u8);
        data_port.write(status | 2);

        command_port.write(0xD4_u8);
        data_port.write(0xF4_u8);
    }
}

pub fn read_mouse() -> Option<(i8, i8)> {
    let mut data_port = Port::new(0x60);
    unsafe {
        let packet0: u8 = data_port.read();
        if packet0 & 0x08 == 0 {
            return None;
        }
        let packet1: u8 = data_port.read();
        let packet2: u8 = data_port.read();
        Some((packet1 as i8, packet2 as i8))
    }
}

pub fn clear_mouse(x: usize, y: usize) {
    let blank = ScreenChar {
        ascii_character: b' ',
        color_code: ColorCode::new(Color::Black, Color::Black),
    };
    let buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };
    buffer.chars[y][x].write(blank);
}

pub fn draw_mouse(x: usize, y: usize) {
    const MOUSE_CHAR: u8 = 0xdb;
    let color = ColorCode::new(Color::White, Color::Black);
    let mouse_char = ScreenChar {
        ascii_character: MOUSE_CHAR,
        color_code: color,
    };
    let buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };
    buffer.chars[y][x].write(mouse_char);
}

static mut MOUSE_X: usize = 40;
static mut MOUSE_Y: usize = 12;


pub fn update_mouse() {
    if let Some((x_offset, y_offset)) = read_mouse() {
        unsafe {
            clear_mouse(MOUSE_X, MOUSE_Y);

            let new_x = (MOUSE_X as isize + x_offset as isize)
                .clamp(0, (BUFFER_WIDTH - 1) as isize) as usize;
            let new_y = (MOUSE_Y as isize - y_offset as isize)
                .clamp(0, (BUFFER_HEIGHT - 1) as isize) as usize;

            MOUSE_X = new_x;
            MOUSE_Y = new_y;

            draw_mouse(MOUSE_X, MOUSE_Y);
        }
    }
}


// MACROS 

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}