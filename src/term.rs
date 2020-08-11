extern crate libc;
use libc::c_void;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! unix {
    ($call:expr, $fmt:expr) => {{
        let err = unsafe { $call };
        if err != 0 {
            Err(Error::new(ErrorKind::Other, format!($fmt, err)))
        } else {
            Ok(())
        }
    }}
}

fn stdout_write(s: &[u8]) -> std::io::Result<()> {
    let stdout = libc::STDOUT_FILENO;
    let size = unsafe { libc::write(stdout, s.as_ptr() as *const c_void, s.len()) };
    if size == -1 {
        Err(Error::new(
            ErrorKind::Other,
            "Failed to write to reminal",
        ))
    } else {
        Ok(())
    }
}

fn terminal_size() -> std::io::Result<(usize, usize)> {
    let stdout = libc::STDOUT_FILENO;
    let is_tty = unsafe { libc::isatty(stdout) == 1 };

    if !is_tty {
        return Err(Error::new(ErrorKind::InvalidInput, "Tried to get size of not tty"));
    }

    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unix!(libc::ioctl(stdout, libc::TIOCGWINSZ, &mut winsize)
         ,"ioctl to get winsize failed with {}"
         )?;

    if winsize.ws_row == 0 || winsize.ws_col == 0 {
        Err(Error::new(ErrorKind::Other, "got zero dimensions for size"))
    } else {
        Ok((winsize.ws_col as usize, winsize.ws_row as usize))
    }
}

static mut TERM_EXISTS: AtomicBool = AtomicBool::new(false);

pub struct Term {
    original_settings: libc::termios,
    pub width: usize,
    pub height: usize,
}

impl Drop for Term {
    fn drop(&mut self) {
        stdout_write(b"\x1b[?1049l\x1b[?25h").unwrap();

        let stdout = libc::STDOUT_FILENO;
        unix!(libc::tcsetattr(stdout, libc::TCSANOW, &self.original_settings)
             ,"Unable to restore terminal info; code: {}"
             ).unwrap();
        let was_terminal = unsafe { TERM_EXISTS.swap(false, Ordering::Relaxed) };
        if !was_terminal {
            panic!("Double restore of terminal settings");
        }
    }
}

impl Term {
    pub fn new() -> std::io::Result<Term> {
        let was_terminal = unsafe { TERM_EXISTS.swap(true, Ordering::Relaxed) };
        if was_terminal {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "Only one Term allowed to exist at the same time",
            ));
        }

        stdout_write(b"\x1b[?1049h\x1b[?25l")?;

        let stdout = libc::STDOUT_FILENO;
        let mut terminfo = libc::termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: 0,
            c_ospeed: 0,
        };
        unix!(libc::tcgetattr(stdout, &mut terminfo)
             ,"Failed to get terminal info; code {}"
             )?;
        let mut new_info = terminfo.clone();
        // `man tcsetattr` and grep for all those flags. Baiscally set raw mode
        new_info.c_iflag &= !(libc::IXON | libc::ICRNL | libc::ISTRIP);
        new_info.c_oflag &= !(libc::OPOST);
        new_info.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
        new_info.c_cc[libc::VTIME] = 0;
        new_info.c_cc[libc::VMIN] = 0;
        unix!(libc::tcsetattr(stdout, libc::TCSANOW, &new_info)
             ,"Failed to set noncanon terminal mode; code {}"
             )?;

        let (width, height) = terminal_size()?;
        Ok(Term {
            original_settings: terminfo,
            width,
            height,
        })
    }

    pub fn get_input_buffer(&self) -> std::io::Result< std::vec::Vec<u8> > {
        let stdin = libc::STDIN_FILENO;
        let mut full_buffer = std::vec::Vec::new();
        let mut buffer      = std::vec::Vec::new();
        buffer.resize(8, b'\0');

        loop {
            let size = unsafe {
                libc::read(stdin, buffer.as_mut_ptr() as *mut c_void, buffer.len())
            };
            if size == -1 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to read keyboard",
                ));
            }
            buffer.resize(size as usize, b'\0');
            full_buffer.append(&mut buffer);
            if size < 8 {
                break;
            }
        }

        Ok(full_buffer)
    }

    fn cursor_reset(&self) -> std::io::Result<()> {
        stdout_write(b"\x1b[H")?;
        Ok(())
    }

    pub fn put_buffer(&self, s: &[u8]) -> std::io::Result<()> {
        self.cursor_reset()?;
        stdout_write(s)
    }

    pub fn put_utf8_buffer(&self, s: &[char]) -> std::io::Result<()> {
        let str_buffer: String = s.into_iter().collect();
        self.put_buffer(str_buffer.as_bytes())
    }

    pub fn put_partial_buffer(&self, s: &[u8]) -> std::io::Result<()> {
        stdout_write(s)
    }

    pub fn put_partial_utf8_buffer(&self, s: &[char]) -> std::io::Result<()> {
        let str_buffer: String = s.into_iter().collect();
        self.put_partial_buffer(str_buffer.as_bytes())
    }
}
