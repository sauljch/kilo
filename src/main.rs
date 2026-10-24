use libc::{self, STDIN_FILENO};
use std::io::{self, Read, Stdin, Stdout, Write};
use termios::*;

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

fn editor_read_key(stdin: &mut Stdin) -> u8 {
    let mut key = [0; 1];
    loop {
        let bytes_read = stdin.read(&mut key);
        match bytes_read {
            Ok(1) => break,
            _ => continue,
        }
    }

    key[0]
}

fn editor_refresh_screen(stdout: &mut Stdout) -> io::Result<()> {
    stdout.write(b"\x1b[2J")?;
    stdout.write(b"\x1b[H")?;
    stdout.flush()?;
    Ok(())
}

fn editor_process_keypress(stdin: &mut Stdin) -> bool {
    let key = editor_read_key(stdin);

    match key {
        QUIT => false,
        _ => true,
    }
}

fn main() {
    let _raw = RawMode::enable_raw_mode().expect("main(): Failed to enable raw mode");
    let mut stdout = io::stdout();
    let _lock = stdout.lock();
    let mut stdin = io::stdin();

    editor_refresh_screen(&mut stdout).expect("main(): Failed to refresh screen");
    while editor_process_keypress(&mut stdin) {}
}
