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

const CTRL_C: i32 = ctrl_key(b'c');

struct Key;
impl Key {
    const UP: i32 = 1000;
    const LEFT: i32 = 1001;
    const DOWN: i32 = 1002;
    const RIGHT: i32 = 1003;
    const DELETE: i32 = 1004;
    const HOME: i32 = 1005;
    const END: i32 = 1006;
    const PAGE_UP: i32 = 1007;
    const PAGE_DOWN: i32 = 1008;
}

fn get_window_size() -> (u16, u16) {
    let mut ws: winsize = unsafe { mem::zeroed() };
    unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };

    (ws.ws_row, ws.ws_col)
}

fn enable_raw_mode() -> io::Result<Termios> {
    let mut raw = Termios::from_fd(STDIN_FILENO)?;
    let mode = raw;

    raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw.c_oflag &= !(OPOST);
    raw.c_cflag |= CS8;
    raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw)?;
    Ok(mode)
}

struct EditorConfig {
    mode: Termios,
    stdin: Stdin,
    stdout: Stdout,
    buffer: String,
    cursor_x: u16,
    cursor_y: u16,
    screen_rows: u16,
    screen_cols: u16,
}

impl Drop for EditorConfig {
    fn drop(&mut self) {
        self.buffer.push_str("\x1b[?1049l"); // exit alternate screen
        self.clear_screen()
            .expect("EditorConfig::drop(): Failed to clear screen");

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.mode)
            .expect("EditorConfig::drop(): Failed to disable raw mode");
    }
}

impl EditorConfig {
    fn new() -> Self {
        let (rows, cols) = get_window_size();

        let mut config = EditorConfig {
            mode: enable_raw_mode().expect("EditorConfig::new(): Failed to enable raw mode"),
            stdin: io::stdin(),
            stdout: io::stdout(),
            buffer: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            screen_rows: rows,
            screen_cols: cols,
        };

        config.buffer.push_str("\x1b[?1049h"); // enter alternate screen
        config
            .clear_screen()
            .expect("EditorConfig::new(): Failed to clear screen");

        config
    }

    fn read_key(&mut self) -> i32 {
        let mut key = [0; 1];
        loop {
            let _ = self.stdin.read(&mut key);

            if key[0] == b'\x1b' {
                let mut sequence = [0; 3];
                let _ = self.stdin.read(&mut sequence);

                if sequence[0] == b'[' {
                    if sequence[1] >= b'0' && sequence[1] <= b'9' {
                        if sequence[2] == b'~' {
                            match sequence[1] {
                                b'1' => return Key::HOME,
                                b'3' => return Key::DELETE,
                                b'4' => return Key::END,
                                b'5' => return Key::PAGE_UP,
                                b'6' => return Key::PAGE_DOWN,
                                b'7' => return Key::HOME,
                                b'8' => return Key::END,
                                _ => (),
                            }
                        }

                        return b'\x1b' as i32;
                    } else {
                        match sequence[1] {
                            b'A' => return Key::UP,
                            b'B' => return Key::DOWN,
                            b'C' => return Key::RIGHT,
                            b'D' => return Key::LEFT,
                            b'H' => return Key::HOME,
                            b'F' => return Key::END,
                            _ => (),
                        }
                    }
                } else if sequence[0] == b'O' {
                    match sequence[1] {
                        b'H' => return Key::HOME,
                        b'F' => return Key::END,
                        _ => (),
                    }
                }
            }

            return key[0] as i32;
        }
    }

    fn move_cursor(&mut self, key: i32) {
        match key {
            Key::UP => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
            }
            Key::LEFT => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            Key::DOWN => {
                if self.cursor_y != self.screen_rows - 1 {
                    self.cursor_y += 1;
                }
            }
            Key::RIGHT => {
                if self.cursor_x != self.screen_cols - 1 {
                    self.cursor_x += 1;
                }
            }
            _ => (),
        }
    }

    fn process_keypress(&mut self) -> i32 {
        let key = self.read_key();

        match key {
            Key::UP | Key::LEFT | Key::DOWN | Key::RIGHT => {
                self.move_cursor(key);
            }
            Key::PAGE_UP | Key::PAGE_DOWN => {
                for _ in 0..self.screen_rows {
                    self.move_cursor(if key == Key::PAGE_UP {
                        Key::UP
                    } else {
                        Key::DOWN
                    });
                }
            }
            Key::HOME => {
                self.cursor_x = 0;
            }
            Key::END => {
                self.cursor_x = self.screen_cols - 1;
            }
            _ => (),
        }

        key
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

    fn clear_screen(&mut self) -> io::Result<()> {
        self.buffer.push_str("\x1b[2J");
        self.buffer.push_str("\x1b[H");

        self.stdout.write(self.buffer.as_bytes())?;
        self.stdout.flush()?;

        Ok(())
    }
}

fn main() {
    let mut editor = EditorConfig::new();

    while editor.process_keypress() != CTRL_C {
        editor
            .refresh_screen()
            .expect("main(): Failed to refresh screen");
    }
}
