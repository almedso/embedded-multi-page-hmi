use embedded_multi_page_hmi::*;

use chrono::{DateTime, Utc};
use std::io::{self, stdout, Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};
#[cfg(target_family = "unix")]
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

/// h1. Simulate on unix terminal
///
/// h2. Input is the keyboard
///
/// Keys are:
/// * n - next page
/// * p - previous page
/// * h - home page
/// * SPACE - action
/// * q - quit
///
/// h2. Output is a single terminal line
///
/// We overwrite the line everytime at page update using special terminal characters
///

// ** Input implementation **

// run input in a dedicated thread to prevent blocking
fn spawn_stdin_channel() -> Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
        let byte: u8 = io::stdin()
            .bytes()
            .next()
            .and_then(|byte| byte.ok())
            .unwrap(); // panic if stdin is closed on purpose
        tx.send(byte).unwrap();
    });
    rx
}

// restore previous termcaps
#[cfg(target_family = "unix")]
impl Drop for Input {
    fn drop(&mut self) {
        let stdin = 0;
        // restore original termios attributes
        tcsetattr(stdin, TCSANOW, &mut self.termios).unwrap();
    }
}

struct Input {
    stdin_channel: Receiver<u8>,
    #[cfg(target_family = "unix")]
    termios: Termios,
}

// use termios to suppress echo of key press
impl Input {
    fn new() -> Self {
        #[cfg(target_family = "unix")]
        let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work
                       // on /dev/stdin or /dev/tty
        #[cfg(target_family = "unix")]
        let termios = Termios::from_fd(stdin).unwrap();

        #[cfg(target_family = "unix")]
        {
            let mut new_termios = termios.clone(); // make a mutable copy of termios
                                                   // that we will modify
            new_termios.c_lflag &= !ECHO; // no echo
            new_termios.c_lflag &= !ICANON; // no canonical mode
            tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
        }

        Input {
            stdin_channel: spawn_stdin_channel(),
            #[cfg(target_family = "unix")]
            termios,
        }
    }
}

impl Iterator for Input {
    type Item = Interaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stdin_channel.try_recv() {
            Ok(key) => match key as char {
                'n' => Some(Interaction::Next),
                'p' => Some(Interaction::Previous),
                ' ' => Some(Interaction::Action),
                'b' => Some(Interaction::Back),
                'h' => Some(Interaction::Home),
                _ => None,
            },
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}

// ** Display implementation **

struct TerminalDisplay {
    len: usize,
}

impl TerminalDisplay {
    fn update(&mut self, message: &String) {
        fn remove_previous_message(l: usize) -> String {
            let mut s: String = String::new();
            // move cursor back
            for _i in 0..l {
                s.push('\x08')
            }
            // overwrite with space
            for _i in 0..l {
                s.push(' ')
            }
            // move cursor back again
            for _i in 0..l {
                s.push('\x08')
            }
            s
        }
        print!("{}", remove_previous_message(self.len + 6));
        self.len = message.len();
        print!("** {} **", message);
        stdout().flush().unwrap();
    }
}

// ** Page specifications **

struct Page {
    message: String,
}

impl Page {
    fn new(message: &str) -> Self {
        Page {
            message: message.to_string(),
        }
    }
}

impl PageInterface<TerminalDisplay> for Page {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(&self.message);
    }
}

struct TimePage;

impl PageInterface<TerminalDisplay> for TimePage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_time: String = now.format("%T").to_string();
        display_driver.update(&formatted_time);
    }
}

// ** Arbitrary functions **

fn sleep_ms(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

fn main() {
    let home = Page::new("Home message");
    let display = TerminalDisplay { len: 0 };
    let mut m = PageManager::new(display, Box::new(home));
    let mut input = Input::new();

    // Optional cannot be reached by external action - called when entering async loop
    let startup = Page::new("Welcome message");
    m.register_startup(Box::new(startup));

    // Optional cannot be reached by external action - called when leaving the async loop
    let shutdown = Page::new("Bye bye message");
    m.register_shutdown(Box::new(shutdown));

    // Additional pages reachable by button
    let page_two = Page::new("Second Page");
    m.register(Box::new(page_two));
    let page_three = Page::new("Third Page");
    m.register(Box::new(page_three));

    let page_clock = TimePage {};
    m.register(Box::new(page_clock));

    m.dispatch(PageNavigation::SystemStart);
    sleep_ms(2000);
    m.dispatch(PageNavigation::Home);

    // infinite loop use ctrl-c to exit

    loop {
        match input.next() {
            None => (),
            Some(interaction) => {
                m.dispatch(map_interaction_to_navigation(interaction));
                if matches!(interaction, embedded_multi_page_hmi::Interaction::Action) {
                    break;
                }
            }
        };
        m.update();
        sleep_ms(200);
    }
}
