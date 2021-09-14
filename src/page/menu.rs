use super::basic::BasicPage;

pub struct MenuPage<'a> {
    pub basic: BasicPage,
    selected: usize,
    max_items: usize,
    pub sub_titles: String, // is public to be accessed from outside implementation of PageInterface trait
    back: Option<&'a str>,  // the Back menu entry in language
}

impl<'a> MenuPage<'a> {
    pub fn new(basic: BasicPage, back: Option<&'a str>) -> Self {
        MenuPage {
            basic,
            selected: 1,
            max_items: 1,
            sub_titles: "".to_owned(),
            back,
        }
    }
}

use super::super::*;

impl PageInteractionInterface for MenuPage<'_> {
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        match interaction {
            Interaction::Action => match self.back {
                None => PageNavigation::NthSubpage(self.selected),
                Some(_) => {
                    // Back navigation is simulated and last in list
                    if self.selected == self.max_items {
                        PageNavigation::Up
                    } else {
                        PageNavigation::NthSubpage(self.selected)
                    }
                }
            },
            Interaction::Back => PageNavigation::Up,
            Interaction::Home => PageNavigation::Home,
            Interaction::Next => {
                self.selected += 1;
                if self.selected > self.max_items {
                    self.selected = 1;
                }
                PageNavigation::Update
            }
            // if previous interaction is not available, this implementation is never called
            // but it does not hurt
            Interaction::Previous => {
                self.selected -= 1;
                if self.selected == 0 {
                    self.selected = 1;
                }
                PageNavigation::Update
            }
        }
    }
}

impl PageBaseInterface for MenuPage<'_> {
    fn update<'a>(
        &mut self,
        title_of_subpages: Option<Box<dyn Iterator<Item = &'a str> + 'a>>,
    ) -> Result<PageNavigation, PageError> {
        if let Some(title_iterator) = title_of_subpages {
            self.max_items = 0;
            self.sub_titles = "".to_owned();

            for title in title_iterator {
                self.max_items += 1;
                if self.max_items == self.selected {
                    self.sub_titles.push_str("[ ");
                }
                self.sub_titles.push_str(title);
                if self.max_items == self.selected {
                    self.sub_titles.push_str(" ]");
                }
                self.sub_titles.push(' ');
            }

            // Optional back navigation menu entry is always placed at the end
            if let Some(back_text) = self.back {
                self.max_items += 1;
                if self.max_items == self.selected {
                    self.sub_titles.push_str("[ ");
                }
                self.sub_titles.push_str(back_text);
                if self.max_items == self.selected {
                    self.sub_titles.push_str(" ]");
                }
                self.sub_titles.push(' ');
            }
        }
        Ok(PageNavigation::Update)
    }

    fn title(&self) -> &str {
        self.basic.title
    }
}

#[cfg(test)]
mod tests;
