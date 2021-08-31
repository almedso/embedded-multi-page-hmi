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
//#![no_std]
// use alloc::boxed::Box;

#![allow(clippy::type_complexity)]
use core::mem;

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
#[derive(Debug, Clone, Copy)]
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
/// Data structures that implement the Page trait are Pages and can be handled
/// by the PageManager type
pub trait PageInterface<D> {
    /// Force updating the page content on the display
    fn display(&self, display_driver: &mut D);

    /// Handle an interaction internally
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        map_interaction_to_navigation(interaction)
    }

    /// lifetime indication
    fn get_life_time_in_ms(&self) -> Option<u16> {
        Option::None
    }

    /// Every page has a title; default is empty String
    fn title(&self) -> &str {
        ""
    }
}

/// Implementation of the inter-page interaction model
///
/// The PageManager is responsible for switching among pages while
/// pages do not know about other pages.
/// The PageManager also dispatches events and updates the current page.
///
/// Only a "home" page is mandatory. Any other pages are optional.
/// Startup and Shutdown pages are purely information pages and not
/// activated only by SystemStartup and SystemShutdown events.
///
/// h2. Implementation Note
///
/// There is only one page active at a time, that dispatches events
/// (stored in page variable). other pages are activate by updating links
/// (in respective directions).
///
/// * Tree structures of pages are modeled by left, right, up and down links.
/// * Tree root is where both up and left links are empty.
/// * From the active page, all other pages can be navigated to
///
/// h3. Example Structure
///
/// ```ignore
///  a-------------------b-----------c
///  |                   |
///  d-----e-----f       o-p
///  |     |     |
///  g-h-i j-k-l m-n
/// ```
/// * `a`- is root page
/// * `b, c` - are pages on the same level like `a` reachable via right link of `a`
/// * `d, e, f`- are sub-pages of `a` reachable via down link of `a`
///
pub struct PageManager<D> {
    display: D,
    page: Box<dyn PageInterface<D>>,
    left: Link<Box<dyn PageInterface<D>>>,
    right: Link<Box<dyn PageInterface<D>>>,
    up: Link<Box<dyn PageInterface<D>>>,
    down: Link<Box<dyn PageInterface<D>>>,
    startup: Option<Box<dyn PageInterface<D>>>,
    shutdown: Option<Box<dyn PageInterface<D>>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    page: T,
    left: Link<T>,
    right: Link<T>,
    down: Link<T>,
    up: Link<T>,
}

impl<D> PageManager<D> {
    /// PageManager Constructor
    ///
    /// * `display`: The display data structure where all output is rendered to
    ///   The display data structure and the logic attached makes the rendered
    ///   output appear on some output facility viewable by a human.
    /// * `home`: The "home" page. There must be at least one page. Other pages
    ///    are added by register_* calls.
    pub fn new(display: D, home: Box<dyn PageInterface<D>>) -> Self {
        PageManager::<D> {
            display,
            page: home,
            left: None,
            right: None,
            up: None,
            down: None,
            startup: None,
            shutdown: None,
        }
    }

    /// Update the content of the active page on the display
    pub fn update(&mut self) {
        self.page.display(&mut self.display);
    }

    /// Register a new page
    ///
    /// The page is registered in the "left" direction of the
    /// active page. The registered page will be the new active page.
    ///
    /// * `page` - The page to be registered and activated.
    pub fn register(&mut self, page: Box<dyn PageInterface<D>>) {
        self.push_left(page, None, None);
        self.activate_left();
    }

    /// Register a new sub page
    ///
    /// The page is registered in the "down" direction of the
    /// active page. The registered page will be the new active page.
    ///
    /// * `page`: - The page to be registered and activated.
    pub fn register_sub(&mut self, page: Box<dyn PageInterface<D>>) {
        self.push_down(page, None, None);
        self.activate_down();
    }

    /// Register a startup page
    ///
    /// There can be just one startup page. Multiple calls to this function
    /// overwrite the previously set startup page.
    ///
    /// * `page`: - The page that should serve for startup.
    pub fn register_startup(&mut self, page: Box<dyn PageInterface<D>>) {
        self.startup = Some(page);
    }

    /// Register a shutdown page
    ///
    /// There can be just one shutdown page. Multiple calls to this function
    /// overwrite the previously set shutdwon page.
    ///
    /// * `page`: - The page that should serve for startup.
    pub fn register_shutdown(&mut self, page: Box<dyn PageInterface<D>>) {
        self.shutdown = Some(page);
    }

    fn push_left(
        &mut self,
        page: Box<dyn PageInterface<D>>,
        up: Link<Box<dyn PageInterface<D>>>,
        down: Link<Box<dyn PageInterface<D>>>,
    ) {
        let new_node = Box::new(Node {
            page,
            left: self.left.take(),
            right: None,
            down,
            up,
        });
        self.left = Some(new_node);
    }

    fn push_right(
        &mut self,
        page: Box<dyn PageInterface<D>>,
        up: Link<Box<dyn PageInterface<D>>>,
        down: Link<Box<dyn PageInterface<D>>>,
    ) {
        let new_node = Box::new(Node {
            page,
            left: None,
            right: self.right.take(),
            up,
            down,
        });
        self.right = Some(new_node);
    }

    fn pop_left(
        &mut self,
    ) -> Option<(
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
    )> {
        self.left.take().map(|node| {
            let mut node = node;
            self.left = node.left;
            (node.page, node.up.take(), node.down.take())
        })
    }

    fn pop_right(
        &mut self,
    ) -> Option<(
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
    )> {
        self.right.take().map(|node| {
            let mut node = node;
            self.right = node.right;
            (node.page, node.up.take(), node.down.take())
        })
    }

    /// Navigate to the left page
    /// If there is no left page it returns false and activate page is unchanged
    fn activate_left(&mut self) -> bool {
        match self.pop_left() {
            None => false,
            Some((page, up, down)) => {
                let page = mem::replace(&mut self.page, page);
                let new_up = self.up.take();
                let new_down = self.down.take();
                self.push_right(page, new_up, new_down);
                self.up = up;
                self.down = down;
                true
            }
        }
    }

    /// Navigate to the right page
    /// If there is no right page it returns false and activate page is unchanged
    fn activate_right(&mut self) -> bool {
        match self.pop_right() {
            None => false,
            Some((page, up, down)) => {
                let page = mem::replace(&mut self.page, page);
                let new_up = self.up.take();
                let new_down = self.down.take();
                self.push_left(page, new_up, new_down);
                self.up = up;
                self.down = down;
                true
            }
        }
    }

    fn activate_most_right(&mut self) {
        while self.activate_right() {}
    }

    fn push_down(
        &mut self,
        page: Box<dyn PageInterface<D>>,
        left: Link<Box<dyn PageInterface<D>>>,
        right: Link<Box<dyn PageInterface<D>>>,
    ) {
        let new_node = Box::new(Node {
            page,
            up: None,
            down: self.down.take(),
            left,
            right,
        });
        self.down = Some(new_node);
    }

    fn push_up(
        &mut self,
        page: Box<dyn PageInterface<D>>,
        left: Link<Box<dyn PageInterface<D>>>,
        right: Link<Box<dyn PageInterface<D>>>,
    ) {
        let new_node = Box::new(Node {
            page,
            up: self.up.take(),
            down: None,
            left,
            right,
        });
        self.up = Some(new_node);
    }

    fn pop_down(
        &mut self,
    ) -> Option<(
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
    )> {
        self.down.take().map(|node| {
            let mut node = node;
            self.down = node.down;
            (node.page, node.left.take(), node.right.take())
        })
    }

    fn pop_up(
        &mut self,
    ) -> Option<(
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
    )> {
        self.up.take().map(|node| {
            let mut node = node;
            self.up = node.up;
            (node.page, node.left.take(), node.right.take())
        })
    }

    fn activate_down(&mut self) -> bool {
        match self.pop_down() {
            None => false,
            Some((page, left, right)) => {
                let page = mem::replace(&mut self.page, page);
                let new_left = self.left.take();
                let new_right = self.right.take();
                self.push_up(page, new_left, new_right);
                self.left = left;
                self.right = right;
                true
            }
        }
    }

    fn activate_up(&mut self) -> bool {
        self.activate_most_right();
        match self.pop_up() {
            None => false,
            Some((page, left, right)) => {
                let page = mem::replace(&mut self.page, page);
                let new_left = self.left.take();
                let new_right = self.right.take();
                self.push_down(page, new_left, new_right);
                self.left = left;
                self.right = right;
                true
            }
        }
    }

    fn activate_home(&mut self) {
        while self.activate_up() {}
        self.activate_most_right();
    }

    /// Dispatch an event
    ///
    /// The event can cause a change of the active page, can
    /// lead to an update of the active page content and can also be
    /// ignored.
    ///
    /// At first the event is delegated to the active page to handle it.
    /// The active page can return a different event that is then dispatched
    /// by the page manager.
    ///
    /// * `navigation`: - The navigation event to dispatch
    pub fn dispatch(&mut self, navigation: PageNavigation) {
        match navigation {
            PageNavigation::SystemStart => {
                if let Some(page) = &self.startup {
                    page.display(&mut self.display)
                }
            }
            PageNavigation::SystemStop => {
                if let Some(page) = &self.shutdown {
                    page.display(&mut self.display)
                }
            }
            PageNavigation::Left => {
                self.activate_left();
                self.update();
            }
            PageNavigation::Right => {
                self.activate_right();
                self.update();
            }
            PageNavigation::Home => {
                self.activate_home();
                self.update();
            }
            PageNavigation::Up => {
                self.activate_up();
                self.update();
            }
            PageNavigation::NthSubpage(index) => {
                self.activate_down();
                let mut index: usize = index;
                while index > 1 {
                    self.activate_left();
                    index -= 1;
                }
                self.update();
            }
            PageNavigation::Update => {
                self.update();
            }
        };
    }
}

impl<D> Drop for PageManager<D> {
    fn drop(&mut self) {
        // forward list
        let mut cur_horizontal = self.left.take();
        while let Some(mut boxed_node) = cur_horizontal {
            cur_horizontal = boxed_node.left.take();
        }
        // backward list
        let mut cur_horizontal = self.right.take();
        while let Some(mut boxed_node) = cur_horizontal {
            cur_horizontal = boxed_node.left.take();
        }
    }
}

pub struct Iter<'a, P> {
    left: Option<&'a Node<P>>,
}

impl<D> PageManager<D> {
    pub fn sub_iter(&self) -> Iter<Box<dyn PageInterface<D>>> {
        Iter {
            left: self.down.as_deref(),
        }
    }
}

impl<'a, D> Iterator for Iter<'a, Box<dyn PageInterface<D>>> {
    type Item = &'a Box<dyn PageInterface<D>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.left.map(|node| {
            self.left = node.left.as_deref();
            &node.page
        })
    }
}

#[cfg(test)]
mod tests;
