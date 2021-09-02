use super::super::PageBaseInterface;

/// Basic pages at least have a title and a lifetime
pub struct BasicPage {
    pub title: &'static str,
    pub lifetime_in_ms: Option<u16>,
}

impl BasicPage {
    pub fn new(title: &'static str, lifetime_in_ms: Option<u16>) -> Self {
        BasicPage {
            title,
            lifetime_in_ms,
        }
    }
}

/// Basic pages at least have a title and a lifetime
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
    fn get_life_time_in_ms(&self) -> Option<u16> {
        self.basic.lifetime_in_ms
    }

    /// Every page has a title - default is empty &str
    fn title(&self) -> &str {
        self.basic.title
    }
}
