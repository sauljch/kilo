use libc::{self, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
use std::{
    io::{self, Read, Stdin, Stdout, Write},
    mem,
};
use termios::*;

const WELCOME_MESSAGE: &str = "Kilo editor -- version 0.0.1";

const fn ctrl_key(k: u8) -> u8 {
    k & 0x1f
}

const QUIT: u8 = ctrl_key(b'c');

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
            screen_rows: rows,
            screen_cols: cols,
        }
    }

    fn read_key(&mut self) -> u8 {
        let mut key = [0; 1];
        loop {
            let bytes_read = self.stdin.read(&mut key);
            match bytes_read {
                Ok(1) => break,
                _ => continue,
            }
        }

        key[0]
    }

    fn process_keypress(&mut self) -> bool {
        let key = self.read_key();

        match key {
            QUIT => false,
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
        self.buffer.push_str("\x1b[?25L"); // hide cursor
        self.buffer.push_str("\x1b[H"); // reposition cursor

        self.draw_rows();

        self.buffer.push_str("\x1b[H"); // reposition cursor
        self.buffer.push_str("\x1b[?25L"); // show cursor

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
    while editor.process_keypress() {}
}
