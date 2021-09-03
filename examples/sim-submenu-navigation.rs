use embedded_multi_page_hmi::{
    page::{
        basic::{BasicPage, TextPage},
        menu::MenuPage,
    },
    page_manager::PageManager,
    Interaction, PageInterface, PageNavigation,
};

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
/// * h - home page or exit if on home page
/// * SPACE - action = go to the selected page
///
/// h2. Output is a single terminal line
///
/// We overwrite the line everytime at page update using special terminal characters
///
/// On windows it requires to confirm every key press with enter

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

impl PageInterface<TerminalDisplay> for MenuPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.basic.title, &self.sub_titles);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay> for TextPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.basic.title, &self.text);
        display_driver.update(&output);
    }
}

// ** Arbitrary functions **

fn sleep_ms(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

fn main() {
    let home = MenuPage::new(BasicPage::new("Menu", None));
    let display = TerminalDisplay { len: 0 };
    let mut m = PageManager::new(display, Box::new(home));
    let mut input = Input::new();

    // Additional pages reachable by button
    let page_one = TextPage::new(BasicPage::new("First", None), "First Page");
    m.register_sub(Box::new(page_one));
    let page_two = TextPage::new(BasicPage::new("Second", None), "Second Page");
    m.register(Box::new(page_two));
    let page_three = TextPage::new(BasicPage::new("Third", None), "Third Page");
    m.register(Box::new(page_three));

    m.dispatch(PageNavigation::Home);

    // infinite loop use ctrl-c or space to exit

    loop {
        match input.next() {
            None => (),
            Some(interaction) => {
                if let Err(_e) = m.dispatch_interaction(interaction) {
                    break;
                };
            }
        };
        m.update();
        sleep_ms(200);
    }
    println!();
}
