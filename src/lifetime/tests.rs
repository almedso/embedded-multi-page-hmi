use super::*;

#[test]
fn create_and_is_over() {
    let sut = PageLifetime::new(PageNavigation::Home, 0);
    assert!(sut.is_over());
    let sut = PageLifetime::new(PageNavigation::Home, 1);
    assert!(!sut.is_over());
    let sut = PageLifetime::new(PageNavigation::Home, 2);
    assert!(!sut.is_over());
}

#[test]
fn increase_age_and_reset_age() {
    let mut sut = PageLifetime::new(PageNavigation::Home, 2);
    assert!(!sut.is_over());
    sut.increase_age();
    assert!(!sut.is_over());
    sut.increase_age();
    assert!(sut.is_over());
    sut.increase_age();
    assert!(sut.is_over());

    sut.reset_age();
    assert!(!sut.is_over());
    sut.increase_age();
    assert!(!sut.is_over());
    sut.increase_age();
    assert!(sut.is_over());
}
