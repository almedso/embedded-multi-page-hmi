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

/// Possible Interactions derived from the input
pub enum Interaction {
    Action,
    Next,
    Previous,
    Back,
    Home,
}

/// Possible Interactions derived from the input
pub enum DispatchResult<T> {
    Handled,
    Ignored,
    Navigate(Box<dyn PageInterface<T>>),
}

/// Data structures that implement the Page trait are Pages and can be handled
/// by the PageManager type
pub trait PageInterface<D> {
    /// Force updating the page content on the display
    fn display(&self, display_driver: &mut D);

    /// Handle an interaction internally
    fn dispatch(&mut self, _interaction: Interaction) -> DispatchResult<D> {
        DispatchResult::Ignored
    }

    /// lifetime indication
    fn get_life_time_in_ms(&self) -> Option<u16> {
        Option::None
    }
}

/// Implementation of the inter-page interaction model
pub struct PageManager<P: PageInterface<D>, D> {
    display: D,
    home: P,
}

impl<P: PageInterface<D>, D> PageManager<P, D> {
    pub fn new(display: D, home: P) -> Self {
        PageManager { display, home }
    }

    pub fn update(&mut self) {
        self.home.display(&mut self.display);
    }
}

#[cfg(test)]
mod tests;
