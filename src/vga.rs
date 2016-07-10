use core::fmt;
use spin::Mutex;

const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;
const VGA_WIDTH: u8 = 80;
const VGA_HEIGHT: u8 = 25;

#[repr(u8)]
enum Color {
    Black        = 0,
    Blue         = 1,
    Green        = 2,
    Cyan         = 3,
    Red          = 4,
    Magenta      = 5,
    Brown        = 6,
    LightGrey    = 7,
    DarkGrey     = 8,
    LightBlue    = 9,
    LightGreen   = 10,
    LightCyan    = 11,
    LightRed     = 12,
    LightMagenta = 13,
    LightBrown   = 14,
    White        = 15,
}

pub struct Style(u8);

impl Style {
    const fn new(foreground: Color, background: Color) -> Style {
        Style((foreground as u8) | (background as u8) << 4)
    }
}

pub struct Writer {
    row: u8,
    col: u8,
    style: Style,
}

impl Writer {
    const fn new() -> Writer {
        Writer {
            row: 0,
            col: 0,
            style: Style::new(Color::White, Color::Black),
        }
    }

    fn set_pos(&mut self, row: u8, col: u8) {
        self.row = row;
        self.col = col;
    }

    fn get_size(&self) -> u16 {
        (VGA_WIDTH as u16) * (VGA_HEIGHT as u16)
    }

    fn calc_index(row: u8, col: u8) -> isize {
        (row as isize) * (VGA_WIDTH as isize) + (col as isize)
    }

    fn get_index(&self) -> isize {
        Writer::calc_index(self.row, self.col)
    }

    fn scroll(&mut self) {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                unsafe {
                    *VGA_BUFFER.offset(Writer::calc_index(y-1, x)) =
                        *VGA_BUFFER.offset(Writer::calc_index(y, x));
                }
            }
        }

        let from = (VGA_HEIGHT as isize) * ((VGA_WIDTH-1) as isize);
        let to = (VGA_HEIGHT as isize) * (VGA_WIDTH as isize);

        for i in from..to {
            unsafe {
                *VGA_BUFFER.offset(i) = 0b0000000000100000
            }
        }
    }

    fn next(&mut self) {
        if self.row > VGA_HEIGHT {
            self.col = 0;
            self.scroll();
        } else if self.col > VGA_WIDTH {
            self.col = 0;
            self.row += 1;
        } else {
            self.col += 1;
        }
    }

    fn newline(&mut self) {
        self.col = 0;

        if self.row > VGA_HEIGHT {
            self.scroll();
        } else {
            self.row += 1;
        }
    }

    pub fn write_byte(&mut self, c: u8) {
        match c {
            b' ' => self.next(),
            b'\n' => self.newline(),
            _ => {
                let ptr = unsafe {
                    VGA_BUFFER.offset(self.get_index() as isize)
                };

                unsafe { *ptr = (c as u16) | (self.style.0 as u16) << 8 };

                self.next();
            }
        }
    }
}

impl ::core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> ::core::fmt::Result {
        self.write_byte(c as u8);
        Ok(())
    }
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::vga::WRITER.lock();
        writer.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

pub fn clear_screen() {
    let mut writer = WRITER.lock(); 
    
    writer.set_pos(0, 0);

    for i in 0..writer.get_size() {
        unsafe {
            *VGA_BUFFER.offset(i as isize) = 0b0000000000100000;
        }
    }
}

pub unsafe fn print_error(fmt: fmt::Arguments) {
    use core::fmt::Write;

    let mut writer = Writer::new();

    for i in 0..(VGA_WIDTH as usize)*(VGA_HEIGHT as usize) {
        // Large binary number = black space on black background
        *VGA_BUFFER.offset(i as isize) = 0b0000000000100000 as u16;
    }

    writer.write_fmt(fmt);
}

