use super::basic::BasicPage;
use arrayvec::ArrayString;

/// Page that allows to enter an ascii string composed of fixed set of characters.
///
/// Interaction is as follows:
/// * entering the page -> previous collected characters are shown
/// * entering the page first time -> factory default collected characters are shown
///
/// * next - selects the next character from the set of allowed characters
/// * previous - selects the previous character from the set of allowed characthers
/// * action - appends the selected character to the so far collected characters
/// * back - removes (most right character from the collected characters)
/// * home - leaves the page with UP-navigation
///
/// If previous button does not exist, next starts at the beginning after reaching
/// the end.
///
/// Back can be emulated with next and action if not available.
/// Home can be emulated with next and action if not available.
pub struct EnterStringPage {
    pub basic: BasicPage,
    allowed_characters: &'static str,
    current_char: usize,
    max_chars: usize,
    pub buffer: ArrayString<20>,

    back: Option<&'static str>, // the Back menu entry in language
    up: Option<&'static str>,   // the OK/Up/leave menu entry in language
}

impl EnterStringPage {
    pub fn new(
        basic: BasicPage,
        allowed_characters: &'static str,
        back: Option<&'static str>,
        up: Option<&'static str>,
    ) -> Self {
        let mut max_chars = allowed_characters.len();
        if back.is_some() {
            max_chars += 1;
        }
        if up.is_some() {
            max_chars += 1;
        }
        EnterStringPage {
            basic,
            allowed_characters,
            current_char: 0,
            buffer: ArrayString::<20>::new(),
            back,
            up,
            max_chars,
        }
    }

    /// Determine if finish action is presented and selected
    fn is_finish(&self) -> bool {
        match self.up {
            None => false,
            Some(_) => {
                if self.current_char >= self.allowed_characters.len() {
                    return match self.back {
                        None => true,
                        Some(_) => self.current_char > self.allowed_characters.len(),
                    };
                }
                false
            }
        }
    }

    /// Determine if back action is presented and selected
    fn is_back(&self) -> bool {
        match self.back {
            None => false,
            Some(_) => self.current_char == self.allowed_characters.len(),
        }
    }

    pub fn action_string(&self) -> &'static str {
        if self.is_back() {
            if let Some(back) = self.back {
                return back;
            }
        }
        if self.is_finish() {
            if let Some(up) = self.up {
                return up;
            }
        }
        &self.allowed_characters[self.current_char..self.current_char + 1]
    }
}

use super::super::*;

impl PageInteractionInterface for EnterStringPage {
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        match interaction {
            Interaction::Action => {
                if self.is_back() {
                    self.buffer.pop();
                    return PageNavigation::Update;
                }
                if self.is_finish() {
                    return PageNavigation::Up;
                }
                self.buffer.push(
                    self.allowed_characters
                        .chars()
                        .nth(self.current_char)
                        .unwrap(),
                );
                PageNavigation::Update
            }
            Interaction::Back => {
                self.buffer.pop();
                PageNavigation::Update
            }
            Interaction::Home => PageNavigation::Up,
            Interaction::Next => {
                self.current_char += 1;
                if self.current_char >= self.max_chars {
                    self.current_char = 0;
                }
                PageNavigation::Update
            }
            // if previous interaction is not available, this implementation is never called
            // but it does not hurt
            Interaction::Previous => {
                if self.current_char == 0 {
                    self.current_char = self.max_chars - 1;
                } else {
                    self.current_char -= 1;
                }
                PageNavigation::Update
            }
        }
    }
}

impl PageBaseInterface for EnterStringPage {
    fn update<'a>(
        &mut self,
        _title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        // print the current text
        Ok(PageNavigation::Update)
    }

    fn title(&self) -> &str {
        self.basic.title
    }
}

#[cfg(test)]
mod tests;
