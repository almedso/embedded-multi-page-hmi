use std::time::Duration;

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode},
    execute, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;

use chrono::{DateTime, Utc};
use embedded_multi_page_hmi::{
    page::{BasicPage, ShutdownPage, StartupPage, TextPage},
    Interaction, PageBaseInterface, PageInteractionInterface, PageInterface, PageLifetime,
    PageManager, PageNavigation,
};

// ** Display implementation **

struct TerminalDisplay;

impl TerminalDisplay {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        TerminalDisplay {}
    }

    fn update(&mut self, title: &str, message: &str) {
        let mut stdout = stdout();

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0),
            style::SetForegroundColor(style::Color::Black),
            style::SetBackgroundColor(style::Color::Green),
            style::Print(title.to_string()),
            style::ResetColor,
            cursor::MoveTo(0, 1),
            style::Print(message.to_string())
        )
        .unwrap();
    }
}

impl Drop for TerminalDisplay {
    fn drop(&mut self) {
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
        )
        .unwrap();
        disable_raw_mode().unwrap();
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

impl PageInterface<TerminalDisplay> for HomePage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(self.0.basic.title, self.0.text);
    }
}

impl PageInterface<TerminalDisplay> for TextPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(self.title(), self.text);
    }
}

impl PageInterface<TerminalDisplay> for StartupPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(self.0.basic.title, self.0.text);
    }
}

impl PageInterface<TerminalDisplay> for ShutdownPage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        display_driver.update(self.0.basic.title, self.0.text);
    }
}

impl PageInterface<TerminalDisplay> for TimePage {
    fn display(&self, display_driver: &mut TerminalDisplay) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_time: String = now.format("%T").to_string();
        display_driver.update(self.title(), &formatted_time);
    }
}

fn map_interaction(event: Event) -> Option<Interaction> {
    if event == Event::Key(KeyCode::Char('n').into()) {
        return Some(Interaction::Next);
    };
    if event == Event::Key(KeyCode::Char('p').into()) {
        return Some(Interaction::Previous);
    };
    if event == Event::Key(KeyCode::Char(' ').into()) {
        return Some(Interaction::Action);
    };
    if event == Event::Key(KeyCode::Char('b').into()) {
        return Some(Interaction::Back);
    };
    if event == Event::Key(KeyCode::Char('h').into()) {
        return Some(Interaction::Home);
    };
    None
}

async fn print_events(m: &mut PageManager<'_, TerminalDisplay>) {
    let mut reader = EventStream::new();
    let mut navigation = m.dispatch(PageNavigation::SystemStart).unwrap();

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let input: Option<Interaction>;

        select! {
            _ = delay => input  = None ,
            maybe_event = event => {
                input = match maybe_event {
                    Some(Ok(event)) => map_interaction(event),
                    Some(Err(_e)) => None,
                    None => None,
                };

            },
        };
        let result = match input {
            None => m.dispatch(navigation),
            Some(interaction) => m.dispatch_interaction(interaction),
        };
        // in this example shutdown page returns PageError after it's lifetime is over
        // this is used for a clean exit
        match result {
            Err(_e) => break,
            Ok(nav) => navigation = nav,
        };
    }
}

fn main() {
    let display = TerminalDisplay::new();
    let home = HomePage::new("!!! This is the home page !!!");
    let mut m = PageManager::new(display, Box::new(home));

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

    async_std::task::block_on(print_events(&mut m));
}
