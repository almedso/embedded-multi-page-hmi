use embedded_multi_page_hmi::{
    lifetime::PageLifetime,
    page::{
        basic::{BasicPage, ShutdownPage, StartupPage, TextPage},
        menu::MenuPage,
    },
    page_manager::PageManager,
    Interaction, PageBaseInterface, PageInteractionInterface, PageInterface, PageNavigation,
};

use chrono::{DateTime, Utc};
use pancurses::{endwin, initscr, noecho, Input, Window};
use std::{thread, time};

/// h1. Showcase all capabilities on host via pancurses
///
/// pancurses supports  Windows, Linux and OS X. (mac)
///
/// h2. Input is the keyboard
///
/// Keys are:
/// * n - next page
/// * p - previous page
/// * h - home page or exit if on home page
/// * b - go back, i.e. the page up the hierarchy
/// * SPACE - action = go to the selected page
///
/// h2. Output is a terminal used as a fixed window
///

// ** Input implementation **

struct TerminalInput<'a> {
    window: &'a Window,
}

impl<'a> TerminalInput<'a> {
    pub fn new(window: &'a Window) -> Self {
        TerminalInput { window }
    }
}

impl Iterator for TerminalInput<'_> {
    type Item = Interaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.window.getch() {
            Some(Input::Character('n')) => Some(Interaction::Next),
            Some(Input::Character('p')) => Some(Interaction::Previous),
            Some(Input::Character(' ')) => Some(Interaction::Action),
            Some(Input::Character('b')) => Some(Interaction::Back),
            Some(Input::Character('h')) => Some(Interaction::Home),
            Some(_input) => None,
            None => None,
        }
    }
}

// ** Display implementation **

struct TerminalDisplay<'a> {
    window: &'a Window,
}

impl<'a> TerminalDisplay<'a> {
    pub fn new(window: &'a Window) -> Self {
        TerminalDisplay { window }
    }

    fn update(&mut self, message: &String) {
        self.window.erase();
        self.window.printw(message);
        self.window.refresh();
    }
}

// ** Page specifications **

// we overwrite the home page for the stake of allowing the showdown
// workflow when requesting up or left navigation on this page.
pub struct HomePage(pub TextPage);

impl HomePage {
    pub fn new(home_message: &'static str) -> Self {
        HomePage(TextPage {
            basic: BasicPage::new("Home", None),
            text: home_message,
        })
    }
}

impl PageBaseInterface for HomePage {
    fn title(&self) -> &str {
        self.0.basic.title
    }
}

// overwrite the default interaction model for the home page
impl PageInteractionInterface for HomePage {
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        match interaction {
            Interaction::Action => PageNavigation::NthSubpage(1),
            Interaction::Back => PageNavigation::SystemStop,
            Interaction::Home => PageNavigation::Home,
            Interaction::Next => PageNavigation::Left,
            Interaction::Previous => PageNavigation::SystemStart,
        }
    }
}

// we define our own page: - a page that shows the current and will life forever
struct TimePage(pub BasicPage);

impl PageBaseInterface for TimePage {
    fn title(&self) -> &str {
        self.0.title
    }
}

impl PageInteractionInterface for TimePage {}

// ** All pages need to implement a display functionality

impl PageInterface<TerminalDisplay<'_>> for HomePage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.0.basic.title, &self.0.text);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay<'_>> for MenuPage<'_> {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.basic.title, &self.sub_titles);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay<'_>> for TextPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.basic.title, &self.text);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay<'_>> for StartupPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.0.basic.title, &self.0.text);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay<'_>> for ShutdownPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let output = format!("{}: {}", &self.0.basic.title, &self.0.text);
        display_driver.update(&output);
    }
}

impl PageInterface<TerminalDisplay<'_>> for TimePage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_time: String = now.format("%T").to_string();
        let output = format!("{}: {}", self.title(), formatted_time);
        display_driver.update(&output);
    }
}

// ** Arbitrary functions **

fn sleep_ms(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

fn main() {
    // pancurses Initialization
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    window.nodelay(true); // read input non-blocking
    noecho();

    let display = TerminalDisplay::new(&window);
    let home = HomePage::new("!!! This is the home page !!!");
    let mut m = PageManager::new(display, Box::new(home));
    let mut input = TerminalInput::new(&window);

    // Optional cannot be reached by external action - called when entering async loop
    // Startup page has a mandatory lifetime.
    let startup = StartupPage::new("Welcome message", 8);
    m.register_startup(Box::new(startup));

    // Optional cannot be reached by external action - called when leaving the async loop
    // Shutdown page has a mandatory lifetime.
    let shutdown = ShutdownPage::new("Bye bye message", 10);
    m.register_shutdown(Box::new(shutdown));

    // Additional pages reachable by next button
    // A predefined Information text page with lifetime
    let page_one = TextPage::new(
        BasicPage::new("First", Some(PageLifetime::new(PageNavigation::Left, 6))),
        "First Information Page with 3 seconds lifetime; moving to next page",
    );
    m.register(Box::new(page_one));

    // A custom defined TimePage without a lifetime
    let page_two = TimePage(BasicPage::new("Time", None));
    m.register(Box::new(page_two));

    // The main menu below home page
    m.dispatch(PageNavigation::Home).unwrap();
    let menu = MenuPage::new(BasicPage::new("Menu", None), None);
    m.register_sub(Box::new(menu));

    let config_one = TextPage::new(BasicPage::new("Config-1", None), "First config Page");
    m.register_sub(Box::new(config_one));

    // A submenu
    let sub_menu = MenuPage::new(BasicPage::new("Sub-Menu", None), None);
    m.register(Box::new(sub_menu));

    let config_two = TextPage::new(BasicPage::new("Config-2", None), "Second config Page");
    m.register_sub(Box::new(config_two));
    let config_three = TextPage::new(BasicPage::new("Config-3", None), "Third config Page");
    m.register(Box::new(config_three));

    // Enter the event loop
    //
    // Note: For proper system startup and shotdown handling navigation events
    // need to be injected again
    let mut navigation = m.dispatch(PageNavigation::SystemStart).unwrap();
    loop {
        let result = match input.next() {
            None => m.dispatch(navigation),
            Some(interaction) => m.dispatch_interaction(interaction),
        };
        // in this example shutdown page returns PageError after it's lifetime is over
        // this is used for a clean exit
        match result {
            Err(_e) => break,
            Ok(nav) => navigation = nav,
        };

        // reading is unblocking, so we need some delay that page update
        // this delay impacts the page update frequency and impacts page aging.
        sleep_ms(500);
    }

    // cleanup and tear down pancurses
    endwin();
}
