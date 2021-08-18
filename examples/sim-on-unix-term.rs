use embedded_multi_page_hmi::*;

use std::io::{self, Read, Write, stdout};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

fn spawn_stdin_channel() -> Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
        let byte: u8 = io::stdin()
            .bytes()
            .next()
            .and_then(|byte| byte.ok())
            .unwrap();  // panic if stdin is closed on purpose
        tx.send(byte).unwrap();
    });
    rx
}

fn sleep_ms(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

struct Input {
    stdin_channel: Receiver<u8>,
    termios: Termios,
}

impl Input {
    fn new() -> Self {
        let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work
                       // on /dev/stdin or /dev/tty
        let termios = Termios::from_fd(stdin).unwrap();
        let mut new_termios = termios.clone();  // make a mutable copy of termios
                                                // that we will modify
        new_termios.c_lflag &= !ECHO; // no echo
        new_termios.c_lflag &= !ICANON; // no canonical mode
        tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

        Input { stdin_channel: spawn_stdin_channel(), termios, }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let stdin = 0;
        // restore original termios attributes
        tcsetattr(stdin, TCSANOW, &mut self.termios).unwrap();
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
                    'q' => Some(Interaction::SystemStop),
                    _ => None,
                },
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
    }
}

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
        stdout().flush();
    }
}

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

fn main() {
    let home = Page::new("Home message");
    let display = TerminalDisplay { len: 0 };
    let mut m = PageManager::new(display, home);
    let mut input = Input::new();

    // Optional cannot be reached by external action - called when entering async loop
    let startup = Page::new("Welcome message");
    m.register_startup(startup);

    // Optional cannot be reached by external action - called when leaving the async loop
    let shutdown = Page::new("Bye bye message");
    m.register_shutdown(shutdown);

    // Additional pages reachable by button
    let page_two = Page::new("Second Page");
    m.register(page_two);
    let page_three = Page::new("Third Page");
    m.register(page_three);

    // let clock = Page::new().toClockPage();
    // let temp = Page::new().toTemperaturePage(TempSensor::new());
    // m.register_action(&home, &clock);
    // m.register_action(&clock, &temp);
    // m.register_action(&temp, &home);

    m.dispatch(Interaction::SystemStart);
    sleep_ms(2000);
    m.dispatch(Interaction::Home);

    // input = Button::Action:todo();
    // async fn page_output() {
    //     m.startup().await;
    //     loop {
    //         let action = input().await;
    //         m.action(action).await;
    //     }
    //     m.shutdown().await;
    // }
    // nb(page_output().await);
    loop {
        match input.next() {
            None => (),
            Some(interaction) => {
                m.dispatch(interaction);
                if matches!(interaction, embedded_multi_page_hmi::Interaction::SystemStop) { break; }
            },
        };
        sleep_ms(200);
    }
    println!();

}
