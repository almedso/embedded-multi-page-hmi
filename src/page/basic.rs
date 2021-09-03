use super::super::{
    Interaction, PageBaseInterface, PageError, PageInteractionInterface, PageLifetime,
    PageNavigation,
};

/// A Basic page has at least a title and an optional lifetime
pub struct BasicPage {
    pub title: &'static str,
    pub lifetime: Option<PageLifetime>,
}

impl BasicPage {
    pub fn new(title: &'static str, lifetime: Option<PageLifetime>) -> Self {
        BasicPage { title, lifetime }
    }
}

/// A text page holds a text and contains the behavior of a Basic page
pub struct TextPage {
    pub basic: BasicPage,
    pub text: &'static str,
}

impl TextPage {
    pub fn new(basic: BasicPage, text: &'static str) -> Self {
        TextPage { basic, text }
    }
}

impl PageBaseInterface for TextPage {
    fn title(&self) -> &str {
        self.basic.title
    }

    // Static page still need to take care for their own lifetime
    fn update<'a>(
        &mut self,
        _title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        match self.basic.lifetime {
            Some(mut lifetime) => {
                let mut result = PageNavigation::Update;
                lifetime.increase_age();
                if lifetime.is_over() {
                    lifetime.reset_age();
                    result = lifetime.get_target();
                }
                self.basic.lifetime = Some(lifetime);
                Ok(result)
            }
            None => Ok(PageNavigation::Update),
        }
    }
}

impl PageInteractionInterface for TextPage {}

/// A startup page
///
/// * Is a text page with title "Startup"
/// * Has a dedicated lifetime
/// * Any user interaction is suppressed
/// * If lifetime is over it turns to home page
pub struct StartupPage(pub TextPage);

impl StartupPage {
    pub fn new(startup_message: &'static str, lifetime_in_updates: u16) -> Self {
        let basic = BasicPage::new(
            "Startup",
            Some(PageLifetime::new(PageNavigation::Home, lifetime_in_updates)),
        );
        StartupPage(TextPage {
            basic,
            text: startup_message,
        })
    }
}

impl PageBaseInterface for StartupPage {
    fn title(&self) -> &str {
        self.0.title()
    }

    /// Update checks lifetime
    ///
    /// Return an error if lifetime is over
    fn update<'a>(
        &mut self,
        _title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        match self.0.basic.lifetime {
            Some(mut lifetime) => {
                let mut result = PageNavigation::SystemStart;
                lifetime.increase_age();
                if lifetime.is_over() {
                    lifetime.reset_age();
                    result = lifetime.get_target();
                }
                self.0.basic.lifetime = Some(lifetime);
                Ok(result)
            }
            None => Ok(PageNavigation::SystemStart),
        }
    }
}

impl PageInteractionInterface for StartupPage {
    /// Do not react on any interaction
    fn dispatch(&mut self, _interaction: Interaction) -> PageNavigation {
        PageNavigation::Update
    }
}

/// A startup page
///
/// * Is a text page with title "Shutdown"
/// * Has a dedicated lifetime
/// * Any user interaction is suppressed
/// * If lifetime is over it turns returns an PageError
pub struct ShutdownPage(pub TextPage);

impl ShutdownPage {
    pub fn new(shutdown_message: &'static str, lifetime_in_updates: u16) -> Self {
        let basic = BasicPage::new(
            "Shutdown",
            Some(PageLifetime::new(
                PageNavigation::SystemStop,
                lifetime_in_updates,
            )),
        );
        ShutdownPage(TextPage {
            basic,
            text: shutdown_message,
        })
    }
}

impl PageBaseInterface for ShutdownPage {
    fn title(&self) -> &str {
        self.0.title()
    }

    /// Update checks lifetime
    ///
    /// Return an error if lifetime is over
    fn update<'a>(
        &mut self,
        _title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        match self.0.basic.lifetime {
            Some(mut lifetime) => {
                lifetime.increase_age();
                self.0.basic.lifetime = Some(lifetime);
                if lifetime.is_over() {
                    lifetime.reset_age();
                    self.0.basic.lifetime = Some(lifetime);
                    Err(PageError)
                } else {
                    Ok(PageNavigation::SystemStop)
                }
            }
            None => Ok(PageNavigation::SystemStop),
        }
    }
}

impl PageInteractionInterface for ShutdownPage {
    /// Do not react on any interaction
    fn dispatch(&mut self, _interaction: Interaction) -> PageNavigation {
        PageNavigation::Update
    }
}

#[cfg(test)]
mod tests;
