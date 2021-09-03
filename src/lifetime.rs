use crate::PageNavigation;

#[derive(Clone, Copy)]
pub struct PageLifetime {
    /// If lifetime is over where to navigate to next
    target: PageNavigation,
    /// How many update calls the page shall survive before deactivation
    lifetime_in_updates: u16,
    /// Internal counter
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
    pub fn is_over(&self) -> bool {
        self.update_counter >= self.lifetime_in_updates
    }

    pub fn get_target(&self) -> PageNavigation {
        self.target
    }

    pub fn increase_age(&mut self) {
        self.update_counter += 1;
    }

    pub fn reset_age(&mut self) {
        self.update_counter = 0;
    }
}

#[cfg(test)]
mod tests;
