use embedded_multi_page_hmi::*;

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

    fn title(&self) -> &str {
        &self.message[0..self.message.len() - 5]
    }
}

struct ListPage {
    selected: usize,
    max_items: usize,
    sub_titles: String,
}

impl ListPage {
    fn new() -> Self {
        ListPage {
            selected: 1,
            max_items: 1,
            sub_titles: "".to_owned(),
        }
    }
}

impl PageInterface<TerminalDisplay> for ListPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(&self.sub_titles);
    }

    fn update<'a>(&mut self, title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>) {
        if let Some(title_iterator) = title_of_subpages {
            self.max_items = 0;
            self.sub_titles = "".to_owned();
            for title in title_iterator {
                self.max_items += 1;
                if self.max_items == self.selected {
                    self.sub_titles.push_str("[ ");
                }
                self.sub_titles.push_str(title);
                if self.max_items == self.selected {
                    self.sub_titles.push_str(" ]");
                }
                self.sub_titles.push_str(" ");
            }
        }
    }

    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        match interaction {
            Interaction::Action => PageNavigation::NthSubpage(self.selected),
            Interaction::Back => PageNavigation::Up,
            Interaction::Home => PageNavigation::SystemStop,
            Interaction::Next => {
                self.selected += 1;
                if self.selected > self.max_items {
                    self.selected = self.max_items;
                }
                PageNavigation::Update
            }
            Interaction::Previous => {
                self.selected -= 1;
                if self.selected == 0 {
                    self.selected = 1;
                }
                PageNavigation::Update
            }
        }
    }
}

// ** Arbitrary functions **

fn sleep_ms(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

fn main() {
    let home = ListPage::new();
    let display = TerminalDisplay { len: 0 };
    let mut m = PageManager::new(display, Box::new(home));
    let mut input = Input::new();

    // Additional pages reachable by button
    let page_one = Page::new("First Page");
    m.register_sub(Box::new(page_one));
    let page_two = Page::new("Second Page");
    m.register(Box::new(page_two));
    let page_three = Page::new("Third Page");
    m.register(Box::new(page_three));

    m.dispatch(PageNavigation::Home);

    // infinite loop use ctrl-c or space to exit

    loop {
        match input.next() {
            None => (),
            Some(interaction) => {
                if matches!(
                    m.dispatch_interaction(interaction),
                    embedded_multi_page_hmi::PageNavigation::SystemStop
                ) {
                    break;
                };
            }
        };
        m.update();
        sleep_ms(200);
    }
    println!();
}
