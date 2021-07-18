use embedded_multi_page_hmi::*;
use rawkey::{KeyCode, RawKey};


struct Input;

impl Iterator for Input {
    type Item = Interaction;

    fn next(&mut self) -> Option<Self::Item> {
        let rawkey = RawKey::new();
        let mut selected_action: Option<Self::Item> =  None;
        let mut previous_selected_action: Option<Self::Item> = None;

        loop {
            if rawkey.is_pressed(KeyCode::Escape) {
                return None;
            }
            if rawkey.is_pressed(KeyCode::UpArrow) {
                selected_action = Some(Interaction::Previous);
            }
            if rawkey.is_pressed(KeyCode::DownArrow) {
                selected_action = Some(Interaction::Next);
            }
            if rawkey.is_pressed(KeyCode::LeftArrow) {
                selected_action = Some(Interaction::Back);
            }
            if rawkey.is_pressed(KeyCode::RightArrow) {
                selected_action = Some(Interaction::Action);
            }

            // previous_selected_action = selected_action;
            match selected_action {
                None => {
                    match previous_selected_action {
                        None => { continue; },
                        Some(x) =>  { previous_selected_action = None; return Some(x); },
                    }
                }
                Some(x) => { previous_selected_action = Some(x); selected_action = None; },
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
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
    let mut input = Input;


    let startup = Page::new("Welcome message");
    let shutdown = Page::new("Bye bye message");

    // Optional cannot be reached by external action - called when entering async loop
    m.register_startup(startup);
    // Optional cannot be reached by external action - called when leaving the async loop
    m.register_shutdown(shutdown);

    // Additional pages reachable by action button

    // let clock = Page::new().toClockPage();
    // let temp = Page::new().toTemperaturePage(TempSensor::new());
    // m.register_action(&home, &clock);
    // m.register_action(&clock, &temp);
    // m.register_action(&temp, &home);

    m.dispatch(Interaction::SystemStart);

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
    m.update();  // show home page
    loop {
        match input.next() {
            None => break,
            Some(interaction) => match interaction {
                Interaction::Action => m.update(),
                _ => print!("**"),
            },
        }
    }
    println!();

}
