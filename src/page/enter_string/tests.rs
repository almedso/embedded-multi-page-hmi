// #[allow(unused_imports)]
// use super::super::super::setting::Setting;  // need to import the trait
//use super::super::super::setting::CellSetting;
use super::*;

#[test]
fn check_title_and_init() {
    let value: CellSetting<i32> = Default::default();
    value.set(123);
    let sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    assert_eq!(sut.title(), "MyTitle");
    assert_eq!(sut.allowed_characters, "0123");
    assert_eq!(sut.current_char, 0);
    assert_eq!(sut.back, Some("Back"));
    assert_eq!(sut.up, Some("Ok"));
    assert_eq!(&sut.buffer[..], "123");
    assert_eq!(sut.max_chars, 6);

    let sut: EnterStringPage<i32> =
        EnterStringPage::<i32>::new(BasicPage::new("MyTitle", None), "0123", None, None, &value);
    assert_eq!(sut.max_chars, 4);
}

#[test]
fn check_is_finish_with_back_and_finish_emulation() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    sut.current_char = 3;
    assert!(!sut.is_finish());
    sut.current_char = 4;
    assert!(!sut.is_finish());
    sut.current_char = 5;
    assert!(sut.is_finish());
}

#[test]
fn check_is_finish_with_finish_emulation() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        None,
        Some("Ok"),
        &value,
    );
    sut.current_char = 3;
    assert!(!sut.is_finish());
    sut.current_char = 4;
    assert!(sut.is_finish());
}

#[test]
fn check_is_finish_without_finish_emulation() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("back"),
        None,
        &value,
    );
    sut.current_char = 3;
    assert!(!sut.is_finish());
    sut.current_char = 4;
    assert!(!sut.is_finish());
}

#[test]
fn check_is_back_without_finish_emulation() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("back"),
        None,
        &value,
    );
    sut.current_char = 3;
    assert!(!sut.is_back());
    sut.current_char = 4;
    assert!(sut.is_back());
    sut.current_char = 5;
    assert!(!sut.is_back());
}

#[test]
fn dispatch_next() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 1);
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 2);
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 3);
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 4);
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 5);
    assert_eq!(sut.dispatch(Interaction::Next), PageNavigation::Update);
    assert_eq!(sut.current_char, 0);
}

#[test]
fn dispatch_previous() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 5);
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 4);
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 3);
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 2);
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 1);
    assert_eq!(sut.dispatch(Interaction::Previous), PageNavigation::Update);
    assert_eq!(sut.current_char, 0);
}

#[test]
fn dispatch_action_back_and_up() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    // add an ordinary first allowed
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "00");

    // Simulate back action even at empty buffer
    sut.current_char = 4;
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "0");
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "");
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "");

    // Add something multiple times
    sut.current_char = 3;
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    sut.current_char = 2;
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "332");

    // Simulate and real back action
    assert_eq!(sut.dispatch(Interaction::Back), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "33");
    sut.current_char = 4;
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Update);
    assert_eq!(&sut.buffer[..], "3");

    // Go home up
    assert_eq!(sut.dispatch(Interaction::Home), PageNavigation::Up);
    assert_eq!(&sut.buffer[..], "3");
    // Go home up simulated
    sut.current_char = 5;
    assert_eq!(sut.dispatch(Interaction::Action), PageNavigation::Up);
    assert_eq!(&sut.buffer[..], "3");
}

#[test]
fn action_string() {
    let value: CellSetting<i32> = Default::default();
    let mut sut: EnterStringPage<i32> = EnterStringPage::<i32>::new(
        BasicPage::new("MyTitle", None),
        "0123",
        Some("Back"),
        Some("Ok"),
        &value,
    );
    // Simulate back action even at empty buffer
    assert_eq!(sut.action_string(), "0");
    sut.current_char = 3;
    assert_eq!(sut.action_string(), "3");
    sut.current_char = 4;
    assert_eq!(sut.action_string(), "Back");
    sut.current_char = 5;
    assert_eq!(sut.action_string(), "Ok");
}
