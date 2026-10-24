use libc::{self, STDIN_FILENO};
use std::io::{self, Read};
use termios::*;

struct RawMode {
    orig_termios: Termios,
}

impl RawMode {
    fn enable_raw_mode() -> io::Result<()> {
        let mut raw = Termios::from_fd(STDIN_FILENO)?;

        raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        raw.c_oflag &= !(OPOST);
        raw.c_cflag |= CS8;
        raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        raw.c_cc[VMIN] = 0;
        raw.c_cc[VTIME] = 1;

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw)?;
        Ok(())
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.orig_termios).expect("Failed to drop RawMode");
    }
}

fn main() {
    let _raw = RawMode::enable_raw_mode();

    loop {
        let mut c = [b'\0'];
        match io::stdin().read(&mut c) {
            Ok(0) => continue, // timeout, no key presses
            _ => (),
        }

        if c[0].is_ascii_control() {
            print!("{:?}\r\n", c);
        } else {
            print!("{:?}\t('{}')\r\n", c, char::from(c[0]));
        }

        if c == [b'q'] {
            break;
        }
    }
}
