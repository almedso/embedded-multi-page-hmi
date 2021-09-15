use super::*;

#[test]
fn check_title_and_init() {
    let sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    assert_eq!(sut.title(), "MyTitle");
    assert_eq!(sut.selected, 1);
    assert_eq!(&sut.sub_titles[..], "");
    assert_eq!(sut.max_items, 1);
    assert_eq!(sut.back, Some("Back"));
}

#[test]
fn update_sub_titles_without_back_simulation() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), None);
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    assert_eq!(&sut.sub_titles[..], "[ foo ] bar baz ");
    assert_eq!(sut.max_items, 3);
}

#[test]
fn update_sub_titles_with_back_simulation() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    assert_eq!(&sut.sub_titles[..], "[ foo ] bar baz Back ");
    assert_eq!(sut.max_items, 4);
}

#[test]
fn interaction_next() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    assert_eq!(sut.selected, 1);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Next));
    assert_eq!(sut.selected, 2);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Next));
    assert_eq!(sut.selected, 3);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Next));
    assert_eq!(sut.selected, 4);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Next));
    assert_eq!(sut.selected, 1);
}

#[test]
fn interaction_previous() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    sut.selected = 4;
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Previous));
    assert_eq!(sut.selected, 3);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Previous));
    assert_eq!(sut.selected, 2);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Previous));
    assert_eq!(sut.selected, 1);
    assert_eq!(PageNavigation::Update, sut.dispatch(Interaction::Previous));
    assert_eq!(sut.selected, 1);
}

#[test]
fn interaction_home() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    assert_eq!(PageNavigation::Home, sut.dispatch(Interaction::Home));
}

#[test]
fn interaction_action_with_back_navigation() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    sut.selected = 4;
    assert_eq!(PageNavigation::Up, sut.dispatch(Interaction::Action));
}

#[test]
fn interaction_action_without_back_navigation() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), None);
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    sut.selected = 1;
    assert_eq!(
        PageNavigation::NthSubpage(1),
        sut.dispatch(Interaction::Action)
    );
    sut.selected = 2;
    assert_eq!(
        PageNavigation::NthSubpage(2),
        sut.dispatch(Interaction::Action)
    );
    sut.selected = 3;
    assert_eq!(
        PageNavigation::NthSubpage(3),
        sut.dispatch(Interaction::Action)
    );
}

#[test]
fn interaction_up() {
    let sub_titles = ["foo", "bar", "baz"];
    let mut sut = MenuPage::new(BasicPage::new("MyTitle", None), Some("Back"));
    sut.update(Some(Box::new(sub_titles.iter().map(|p| *p))))
        .unwrap();
    assert_eq!(PageNavigation::Up, sut.dispatch(Interaction::Back));
}
