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
//!   * If the lifetime is over, automatically the home page is activated.
//!
//!
//! TODO: Solve update of dynamic page content like current time or temperature
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

//use core::mem;
use std::mem;

/// Possible Interactions derived from the input
#[derive(Debug, Clone, Copy)]
pub enum Interaction {
    SystemStart, // generic to start the HMI (not user selected)
    SystemStop,  // stop the HMI (not user selected)
    Action,
    Next,
    Previous,
    Back,
    Home,
}

/// Possible Interactions derived from the input
pub enum DispatchResult<P> {
    Handled,
    Ignored,
    Navigate(P),
}

/// Data structures that implement the Page trait are Pages and can be handled
/// by the PageManager type
pub trait PageInterface<D> {
    /// Force updating the page content on the display
    fn display(&self, display_driver: &mut D);

    /// Handle an interaction internally
    fn dispatch(&mut self, _interaction: Interaction) -> DispatchResult<Box<dyn PageInterface<D>>> {
        DispatchResult::Ignored
    }

    /// lifetime indication
    fn get_life_time_in_ms(&self) -> Option<u16> {
        Option::None
    }

    /// Every page has a title; default is empty String
    fn title(&self) -> String {
        "".to_string()
    }

    // /// Register the next page on the same level
    // fn register_next(&mut self, _page: Box<dyn PageInterface<D>>);

    // /// Register the a sub page there could be just one sub page
    // fn register_sub(&mut self, _page: Box<dyn PageInterface<D>>);
}

/// Implementation of the inter-page interaction model
pub struct PageManager<P: PageInterface<D>, D> {
    display: D,
    page: P,
    next: Link<P>,
    previous: Link<P>,
    startup: Option<P>,
    shutdown: Option<P>,
}

type Link<P> = Option<Box<Node<P>>>;

struct Node<P> {
    page: P,
    link: Link<P>,
}

impl<P> Node<P> {
    fn new(page: P) -> Self {
        Node::<P> { page, link: None }
    }
}

impl<P: PageInterface<D>, D> PageManager<P, D> {
    pub fn new(display: D, home: P) -> Self {
        PageManager::<P, D> {
            display,
            page: home,
            next: None,
            previous: None,
            startup: None,
            shutdown: None,
        }
    }

    pub fn update(&mut self) {
        self.page.display(&mut self.display);
    }

    pub fn register(&mut self, page: P) {
        self.push_next(page);
        self.activate_next();
    }

    pub fn register_startup(&mut self, page: P) {
        self.startup = Some(page);
    }

    pub fn register_shutdown(&mut self, page: P) {
        self.shutdown = Some(page);
    }

    fn push_next(&mut self, page: P) {
        let new_node = Box::new(Node {
            page: page,
            link: self.next.take(),
        });
        self.next = Some(new_node);
    }

    fn push_previous(&mut self, page: P) {
        let new_node = Box::new(Node {
            page: page,
            link: self.previous.take(),
        });
        self.previous = Some(new_node);
    }

    fn pop_next(&mut self) -> Option<P> {
        self.next.take().map(|node| {
            self.next = node.link;
            node.page
        })
    }

    fn pop_previous(&mut self) -> Option<P> {
        self.previous.take().map(|node| {
            self.previous = node.link;
            node.page
        })
    }

    fn activate_next(&mut self) -> bool {
        match self.pop_next() {
            None => false,
            Some(page) => {
                let page = mem::replace(&mut self.page, page);
                self.push_previous(page);
                true
            }
        }
    }

    fn activate_previous(&mut self) -> bool {
        match self.pop_previous() {
            None => false,
            Some(page) => {
                let page = mem::replace(&mut self.page, page);
                self.push_next(page);
                true
            }
        }
    }

    fn activate_most_previous(&mut self) {
        while self.activate_previous() {}
    }

    fn activate_home(&mut self) {
        self.activate_most_previous();
    }

    pub fn dispatch(&mut self, interaction: Interaction) {
        match interaction {
            Interaction::SystemStart => match &self.startup {
                Some(page) => page.display(&mut self.display),
                _ => (),
            },
            Interaction::SystemStop => match &self.shutdown {
                Some(page) => page.display(&mut self.display),
                _ => (),
            },
            Interaction::Next => {
                self.activate_next();
                self.page.display(&mut self.display);
            }
            Interaction::Previous => {
                self.activate_previous();
                self.page.display(&mut self.display);
            }
            Interaction::Home => {
                self.activate_home();
                self.page.display(&mut self.display);
            }
            Interaction::Action => {
                self.update();
            }
            _ => {}
        };
    }
}

impl<P: PageInterface<D>, D> Drop for PageManager<P, D> {
    fn drop(&mut self) {
        // forward list
        let mut cur_link = self.next.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.link.take();
        }
        // backward list
        let mut cur_link = self.previous.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.link.take();
        }
    }
}

pub struct Iter<'a, P> {
    next: Option<&'a Node<P>>,
}

impl<P: PageInterface<D>, D> PageManager<P, D> {
    pub fn forward_iter<'a>(&'a self) -> Iter<'a, P> {
        Iter {
            next: self.next.as_deref(),
        }
    }

    pub fn backward_iter<'a>(&'a self) -> Iter<'a, P> {
        Iter {
            next: self.previous.as_deref(),
        }
    }
}

impl<'a, P> Iterator for Iter<'a, P> {
    type Item = &'a P;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.link.as_deref();
            &node.page
        })
    }
}

#[cfg(test)]
mod tests;
