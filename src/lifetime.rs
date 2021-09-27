use crate::PageNavigation;

/// PageLifetime enables a page to automatically switch to another page after a certain time.
///
/// The page lifetime is only applied while a page is presented.
/// Each page type is responsible to care if page lifetime is to be considered.
/// Page lifetime is measured in update events. I.e. an update event shall cause a call to
/// increase_age.
#[derive(Clone, Copy)]
pub struct PageLifetime {
    target: PageNavigation,
    lifetime_in_updates: u16,
    update_counter: u16,
}

impl PageLifetime {
    pub fn new(target: PageNavigation, lifetime_in_updates: u16) -> Self {
        PageLifetime {
            target,
            lifetime_in_updates,
            update_counter: 0,
        }
    }
    /// Check if lifetime is over
    pub fn is_over(&self) -> bool {
        self.update_counter >= self.lifetime_in_updates
    }

    /// Where to navigate to if lifetime is over
    pub fn get_target(&self) -> PageNavigation {
        self.target
    }

    /// Increase page age - to be called by page if it receives page update event.
    pub fn increase_age(&mut self) {
        self.update_counter += 1;
    }

    /// Rebirth of a page - to be called by page has just turned active.
    pub fn reset_age(&mut self) {
        self.update_counter = 0;
    }
}

#[cfg(test)]
mod tests;
