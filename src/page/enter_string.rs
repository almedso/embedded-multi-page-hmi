#[allow(unused_imports)]
use super::super::setting::Setting;

use super::super::setting::CellSetting;
use super::basic::BasicPage;

use std::fmt::{Debug, Display};
#[allow(unused_imports)]
use std::str::FromStr;
#[allow(unused_imports)]
use std::string::String;

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
pub struct EnterStringPage<'a, T> {
    pub basic: BasicPage,
    allowed_characters: &'static str,
    current_char: usize,
    max_chars: usize,
    pub buffer: String,

    back: Option<&'static str>, // the Back menu entry in language
    up: Option<&'static str>,   // the OK/Up/leave menu entry in language
    value: &'a CellSetting<T>,  // the value to store
}

impl<'a, T: Copy + FromStr + Display> EnterStringPage<'a, T>
where
    <T as FromStr>::Err: Debug,
{
    pub fn new(
        basic: BasicPage,
        allowed_characters: &'static str,
        back: Option<&'static str>,
        up: Option<&'static str>,
        value: &'a CellSetting<T>,
    ) -> Self {
        let mut max_chars = allowed_characters.len();
        if back.is_some() {
            max_chars += 1;
        }
        if up.is_some() {
            max_chars += 1;
        }
        let buffer = format!("{}", value.get());
        EnterStringPage {
            basic,
            allowed_characters,
            current_char: 0,
            buffer,
            back,
            up,
            max_chars,
            value,
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

    /// Process the action input
    ///
    /// Action is one of:
    ///
    /// * Add the selected character to internal buffer
    /// * Remove last from internal buffer
    /// * Finish the page and return to upper page.
    ///   * Side effect: Update the value it page cares for
    ///
    /// h2. Args
    ///

    pub fn action_string(&self) -> &'static str {
        if self.is_back() {
            if let Some(back) = self.back {
                return back;
            }
        }
        if self.is_finish() {
            if let Some(up) = self.up {
                self.value.set_string(&self.buffer[..]);
                return up;
            }
        }
        &self.allowed_characters[self.current_char..self.current_char + 1]
    }
}

use super::super::*;

impl<T: Copy + FromStr + Display> PageInteractionInterface for EnterStringPage<'_, T>
where
    <T as FromStr>::Err: Debug,
{
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

impl<T> PageBaseInterface for EnterStringPage<'_, T> {
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
