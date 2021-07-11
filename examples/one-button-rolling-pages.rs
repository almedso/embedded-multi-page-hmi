use embedded_multi_page_hmi::*;

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
        display_driver.update(&"--> Finish".to_string());
    }
}

fn main() {
    let home = Page::new("Home message");
    let display = TerminalDisplay { len: 0 };
    let mut m = PageManager::new(display, home);
    m.update();
    println!();

    // let startup = Page::new_message("Welcome message");
    // let shutdown = Page::new_message("Bye bye message");

    // // Optional cannot be reached by external action - called when entering async loop
    // m.register_startup(startup);
    // // Optional cannot be reached by external action - called when leaving the async loop
    // m.register_shutdown(shutdown);

    // Additional pages reachable by action button

    // let clock = Page::new().toClockPage();
    // let temp = Page::new().toTemperaturePage(TempSensor::new());
    // m.register_action(&home, &clock);
    // m.register_action(&clock, &temp);
    // m.register_action(&temp, &home);

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
}
