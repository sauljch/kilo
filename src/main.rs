use libc::{self, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
use std::{
    io::{self, Read, Stdin, Stdout, Write},
    mem,
};
use termios::*;

const WELCOME_MESSAGE: &str = "Kilo editor -- version 0.0.1";

const fn ctrl_key(k: u8) -> i32 {
    (k & 0x1f) as i32
}

const QUIT: i32 = ctrl_key(b'c');
const UP: i32 = (b'w' as i32) + 1000;
const LEFT: i32 = (b'a' as i32) + 1000;
const DOWN: i32 = (b's' as i32) + 1000;
const RIGHT: i32 = (b'd' as i32) + 1000;

struct RawMode {
    orig_termios: Termios,
}

impl RawMode {
    fn enable_raw_mode() -> io::Result<Self> {
        let mut raw = Termios::from_fd(STDIN_FILENO)?;
        let mode = Self { orig_termios: raw };

        raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        raw.c_oflag &= !(OPOST);
        raw.c_cflag |= CS8;
        raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        raw.c_cc[VMIN] = 0;
        raw.c_cc[VTIME] = 1;

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw)?;
        Ok(mode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.orig_termios).expect("Failed to drop RawMode");
    }
}

struct EditorConfig {
    stdin: Stdin,
    stdout: Stdout,
    buffer: String,
    cursor_x: u16,
    cursor_y: u16,
    screen_rows: u16,
    screen_cols: u16,
}

impl EditorConfig {
    fn new() -> Self {
        let (rows, cols) = get_window_size();

        EditorConfig {
            stdin: io::stdin(),
            stdout: io::stdout(),
            buffer: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            screen_rows: rows,
            screen_cols: cols,
        }
    }

    fn read_key(&mut self) -> i32 {
        let mut key = [0; 1];
        loop {
            let _ = self.stdin.read(&mut key);

            if key[0] == b'\x1b' {
                let mut sequence = [0; 3];
                let _ = self.stdin.read(&mut sequence);

                if sequence[0] == b'[' {
                    match sequence[1] {
                        b'A' => return UP,    // up arrow
                        b'B' => return DOWN,  // down arrow
                        b'C' => return RIGHT, // right arrow
                        b'D' => return LEFT,  // left arrow
                        _ => (),
                    }
                }
            }

            return key[0] as i32;
        }
    }

    fn move_cursor(&mut self, key: i32) {
        match key {
            UP => self.cursor_y -= 1,
            LEFT => self.cursor_x -= 1,
            DOWN => self.cursor_y += 1,
            RIGHT => self.cursor_x += 1,
            _ => (),
        }
    }

    fn process_keypress(&mut self) -> bool {
        let key = self.read_key();

        match key {
            QUIT => false,
            UP | LEFT | DOWN | RIGHT => {
                self.move_cursor(key);
                true
            }
            _ => true,
        }
    }

    fn draw_rows(&mut self) {
        for r in 0..self.screen_rows {
            if r == self.screen_rows / 3 && WELCOME_MESSAGE.len() < self.screen_cols.into() {
                // Print welcome message
                let mut padding = (self.screen_cols - WELCOME_MESSAGE.len() as u16) / 2;
                if padding > 0 {
                    self.buffer.push_str("~");
                    padding -= 1;
                }
                while padding != 0 {
                    self.buffer.push_str(" ");
                    padding -= 1;
                }

                self.buffer.push_str(WELCOME_MESSAGE);
            } else {
                self.buffer.push_str("~");
            }

            self.buffer.push_str("\x1b[K");
            if r < self.screen_rows - 1 {
                self.buffer.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        self.buffer.push_str("\x1b[?25l"); // hide cursor
        self.buffer.push_str("\x1b[H"); // reposition cursor

        self.draw_rows();

        let cursor_pos = format!("\x1b[{};{}H", self.cursor_y + 1, self.cursor_x + 1);
        self.buffer.push_str(&cursor_pos);

        self.buffer.push_str("\x1b[?25h"); // show cursor

        self.stdout.write(self.buffer.as_bytes())?;
        self.stdout.flush()?;

        Ok(())
    }
}

fn get_window_size() -> (u16, u16) {
    let mut ws: winsize = unsafe { mem::zeroed() };
    unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };

    (ws.ws_row, ws.ws_col)
}

fn main() {
    let _raw = RawMode::enable_raw_mode().expect("main(): Failed to enable raw mode");
    let mut editor = EditorConfig::new();

    editor
        .refresh_screen()
        .expect("main(): Failed to refresh screen");
    while editor.process_keypress() {
        editor
            .refresh_screen()
            .expect("main(): Failed to refresh screen");
    }
}
