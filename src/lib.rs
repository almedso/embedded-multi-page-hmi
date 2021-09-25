//! # Embedded Multi-Page HMI
//!
//! The embedded multi-page HMI combines a resource constraint display output
//! and a constraint fixed button input by an interaction model.
//!
//! ## Input
//!
//! Input is limited to a small set of buttons.
//!
//! Depending on the amount of buttons several interaction models are predominant.
//! The buttons get a semantic meaning, and allow for certain interaction
//! models. The more buttons there are the more convenient the interaction model
//! is.
//!
//! | Button | Semantics | Assigned activity |
//! | ------ | --------- |------------------ |
//! | first  | action    | activate, confirm, trigger, modify, ... |
//! | second | next      | select the next item |
//! | third  | previous  | select the previous item in list|
//! | fourth | back      | navigate to the previous position |
//! | fifth  | home      | go to home page, reset  |
//!
//! A rotary knop can be modelled as three buttons (action, next, previous).
//!
//! ## Output
//!
//! ### Display
//!
//! The output is on one single display. The display can be
//!
//! * alphanumerical,
//! * or graphical display.
//!
//! ### Pages
//!
//! The output is organized in pages.
//! Exactly one page is displayed at a time on the display.
//!
//! Every page has a lifetime.
//!
//! | Page     | Meaning |
//! | -------- | ------- |
//! | Home     | Is Mandatory; Is the fallback Page, Start point for all navigation |
//! | Startup  | Optional; Shown during init; no interaction; replaced by Home  |
//! | Shutdown | Optional; Shown during de-init; no interaction |
//!
//! Pages have the following properties:
//!
//! * Can handle input interactions.
//!   * this can be used to capture input e.g. of numbers, flags
//!     that get delegated to a data model underneath
//! * Have a lifetime: How long are they displayed w/o input
//!   * If the lifetime is over, automatically the home page or next page is activated.
//! * Pages can have dynamic content (like current time, temperature, etc) that is updated
//!   on a regular base
//!
//! ## Interaction Models
//!
//! Between pages can be navigated, triggered by an interaction or
//! automatically triggered by events (from timer or value change)
//!
//! ### One Button - Sequence of Pages
//!
//! * The `action` interaction activates the next page.
//! * Inside the pages no activity is possible
//!
//! ### Two Button/ Three Button - Sequence of Pages
//!
//! More than one button input allows inter-page interaction.
//!
//! Three button interaction is like two button interaction, except that
//! `previous` is a shortcut for iterating with `next` through a looped list of
//! items.
//!
//! There are information pages and setting pages.
//!
//! **Information pages**:
//!   * purely display (dynamic) information
//!   * do not allow for internal interaction
//!
//! **Setting pages**:
//!   * Allow to select items or enter values
//!
//! * The `next` interaction activates the next info page.
//! * The `action` interaction activates the setting page(s).
//! * Inside the info pages no activity is possible
//! * Inside the setting page(s) it is possible to
//!   * select items with `next` interaction
//!   * activate items with `action` interaction
//!   * *Go back to home (info) page* could be item to select and activate

// later on this should be a no_std to run on embedded - still we need a Box type
// that is not available easily  on no_std
// #![no_std]
// use heapless::pool::Box;
// #![feature(alloc)]  // only works with nightly compiler versions
// use alloc::boxed::Box;

#![allow(clippy::type_complexity)]

/// Possible Interactions derived from the input
#[derive(Debug, Clone, Copy)]
pub enum Interaction {
    /// Primary HMI event to trigger some action e.g. go to next page
    Action,
    /// Primary HMI event to e.g. go to next page
    Next,
    /// Primary HMI event to e.g. go to previous page
    Previous,
    /// Primary HMI event to e.g. go to one page up
    Back,
    /// Event to go to home page.
    /// Could be a primary HMI event or a generated event.
    Home,
}

/// Page navigation events dispatched by pagemanager
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageNavigation {
    /// Start the HMI.
    SystemStart,
    /// Stop the HMI.
    SystemStop,
    /// Stay at the active page and initiate an update.
    Update,
    /// Navigate to the left page.
    Left,
    /// Navigate to the right page.
    Right,
    /// Navigate one page up.
    Up,
    /// Navigate down the n-th subpage. Start counting with one.
    NthSubpage(usize),
    /// Event to go to home page.
    Home,
}

/// Map default 5-button interaction to navigation
pub fn map_interaction_to_navigation(interaction: Interaction) -> PageNavigation {
    match interaction {
        Interaction::Action => PageNavigation::Update,
        Interaction::Back => PageNavigation::Up,
        Interaction::Home => PageNavigation::Home,
        Interaction::Next => PageNavigation::Left,
        Interaction::Previous => PageNavigation::Right,
    }
}

/// Any error a page update my run into
#[derive(Debug, Clone)]
pub struct PageError;

/// Data structures that implement the Page trait are Pages and can be handled
/// by the PageManager type
///
/// Args
/// * `display_driver` - The display to render the page content
/// * `
pub trait PageInterface<D>: PageInteractionInterface {
    /// Force updating the page content on the display
    fn display(&self, display_driver: &mut D);
}

/// Data structures that implement the Page trait are Pages and can be handled
/// by the PageManager type
///
pub trait PageBaseInterface {
    /// Trigger a page-internal update and causes page-internal state modification
    ///
    /// Is called by `PageManager`.
    /// Handles Page Lifetime management
    ///
    /// Args:
    ///     title_of_subpages: Iterator to titles of subpages (Optional)
    ///
    /// Returns:
    ///     `Ok(<PageNavigation>)` - In case update is went well, to indicate the which page
    ///         to navigate to next.
    ///     `Error` - Indicate an error. (Note: Could be on purpose to force a controlled
    ///         gui process shutdown)
    fn update<'a>(
        &mut self,
        _title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        Ok(PageNavigation::Update)
    }

    /// Every page has a title - default is empty &str
    fn title(&self) -> &str {
        ""
    }
}

pub trait PageInteractionInterface: PageBaseInterface {
    /// Handle page interaction
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        map_interaction_to_navigation(interaction)
    }
}

pub mod lifetime;
#[allow(unused_imports)]
use lifetime::PageLifetime;

pub mod page;
pub mod page_manager;

pub mod setting;

// reexport the PageManager
#[allow(unused_imports)]
pub use page_manager::PageManager;
