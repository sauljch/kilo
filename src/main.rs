use std::{
    io::{self, Read},
    os::fd::AsRawFd,
};
use termios::*;

/*** data ***/

#[inline(always)]
fn get_stdin_fd() -> i32 {
    io::stdin().as_raw_fd()
}

/*** terminal ***/

fn disable_raw_mode(fd: i32, termios: Termios) {
    tcsetattr(fd, TCSAFLUSH, &termios).expect("tcsetattr");
}

fn enable_raw_mode(fd: i32) {
    let mut raw = Termios::from_fd(fd).expect("Termios::from_fd");

    raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw.c_oflag &= !(OPOST);
    raw.c_cflag |= CS8;
    raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

    tcsetattr(fd, TCSAFLUSH, &raw).expect("tcgetattr");
}

/*** init ***/

fn main() {
    let orig_termios = Termios::from_fd(get_stdin_fd())
        .expect("Failed to get original termios struct from stdin.");

    enable_raw_mode(get_stdin_fd());

    loop {
        let mut c = [b'\0'];
        match io::stdin().read(&mut c) {
            Ok(0) => continue, // timeout, no key presses
            Ok(_) => (),
            Err(e) => {
                // TODO: set disable_raw_mode to be called within std::panic::set_hook().
                disable_raw_mode(get_stdin_fd(), orig_termios);
                panic!("{e}");
            }
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

    // Rust does not have an exit function like C's atexit().
    // This is the best I can do as of now. :(
    disable_raw_mode(get_stdin_fd(), orig_termios);
}
