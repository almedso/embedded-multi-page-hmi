use super::basic::BasicPage;

pub struct MenuPage {
    pub basic: BasicPage,
    selected: usize,
    max_items: usize,
    pub sub_titles: String, // is public to be accessed from outside implementation of PageInterface trait
}

impl MenuPage {
    pub fn new(basic: BasicPage) -> Self {
        MenuPage {
            basic,
            selected: 1,
            max_items: 1,
            sub_titles: "".to_owned(),
        }
    }
}

use super::super::*;

impl PageInteractionInterface for MenuPage {
    fn dispatch(&mut self, interaction: Interaction) -> PageNavigation {
        match interaction {
            Interaction::Action => PageNavigation::NthSubpage(self.selected),
            Interaction::Back => PageNavigation::Up,
            Interaction::Home => PageNavigation::SystemStop,
            Interaction::Next => {
                self.selected += 1;
                if self.selected > self.max_items {
                    self.selected = self.max_items;
                }
                PageNavigation::Update
            }
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

impl PageBaseInterface for MenuPage {
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
        }
        Ok(PageNavigation::Update)
    }

    fn title(&self) -> &str {
        self.basic.title
    }
}
