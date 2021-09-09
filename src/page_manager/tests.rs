use crate::*;

mod mocks {

    use super::super::SubPageIterator;
    use super::PageInterface;
    use super::*;

    pub struct DisplayDriverStub;

    pub struct DisplayDriverMock {
        expected_updates: Vec<String>,
        collected_updates: Vec<String>,
        context: String,
    }

    pub fn expect(s: &str) -> Vec<String> {
        s.split(' ').map(|s| s.to_string()).collect()
    }

    impl DisplayDriverMock {
        pub fn default(context: &str) -> Self {
            DisplayDriverMock {
                expected_updates: Vec::new(),
                collected_updates: Vec::new(),
                context: String::from(context),
            }
        }

        pub fn new(context: &str, expected: Vec<String>) -> Self {
            DisplayDriverMock {
                expected_updates: expected,
                collected_updates: Vec::new(),
                context: String::from(context),
            }
        }

        pub fn expect(&mut self, s: &str) {
            self.expected_updates.push(s.to_string());
        }

        pub fn update(&mut self, s: &String) {
            self.collected_updates.push(s.clone());
        }
    }

    impl Drop for DisplayDriverMock {
        fn drop(&mut self) {
            if self.expected_updates.len() > 0 {
                assert_eq!(
                    self.expected_updates, self.collected_updates,
                    "Testing {}",
                    self.context
                );
            }
        }
    }

    pub struct PageMock {
        message: String,
    }

    impl PageMock {
        pub fn new(s: &str) -> Self {
            PageMock {
                message: s.to_string(),
            }
        }
    }

    impl PageBaseInterface for PageMock {}

    impl PageInteractionInterface for PageMock {}

    impl PageInterface<DisplayDriverMock> for PageMock {
        fn display(&self, display_driver: &mut DisplayDriverMock) {
            display_driver.update(&self.message);
        }
    }

    impl PageInterface<DisplayDriverStub> for PageMock {
        fn display(&self, _display_driver: &mut DisplayDriverStub) {}
    }

    pub fn check_page_iteration<'a>(
        context: &str,
        expected: Vec<String>,
        iterator: SubPageIterator<'a, Box<dyn PageInterface<DisplayDriverMock>>>,
    ) {
        let mut d = DisplayDriverMock::new(context, expected);
        for i in iterator {
            i.display(&mut d);
        }
    }
}

use super::*;
use mocks::*;

#[test]
fn update_page_manager() {
    let foo = PageMock::new("Foo");
    let mut d = DisplayDriverMock::default("update");
    d.expect("Foo");
    let mut m = PageManager::new(d, Box::new(foo));
    m.update().unwrap();
}

#[test]
fn two_pages_navigation() {
    let foo = PageMock::new("foo");
    let bar = PageMock::new("bar");
    let mut d = DisplayDriverMock::default("two_page_navigation");
    d.expect("bar");
    d.expect("foo");
    d.expect("bar");
    d.expect("foo");
    d.expect("bar");
    d.expect("foo");
    d.expect("foo");
    d.expect("bar");
    d.expect("foo");
    d.expect("bar");
    d.expect("foo");
    d.expect("foo");
    let mut m = PageManager::new(d, Box::new(foo));
    m.register(Box::new(bar));
    m.update().unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
}

#[test]
fn four_pages_navigation() {
    let home = PageMock::new("Home");
    let foo = PageMock::new("foo");
    let bar = PageMock::new("bar");
    let baz = PageMock::new("baz");
    let mut d = DisplayDriverMock::default("four_pages");
    d.expect("baz");
    d.expect("Home");

    d.expect("foo");
    d.expect("bar");
    d.expect("baz");

    d.expect("bar");
    d.expect("foo");
    d.expect("Home");
    d.expect("Home");

    d.expect("foo");
    d.expect("bar");
    d.expect("baz");
    d.expect("Home");
    d.expect("foo");

    let mut m = PageManager::new(d, Box::new(home));
    m.register(Box::new(foo));
    m.register(Box::new(bar));
    m.register(Box::new(baz));
    m.update().unwrap();
    m.dispatch(PageNavigation::Home).unwrap();

    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();

    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();

    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
}

#[test]
fn four_pages_registration_mix() {
    let home = PageMock::new("Home");
    let foo = PageMock::new("foo");
    let bar = PageMock::new("bar");
    let baz = PageMock::new("baz");
    let mut d = DisplayDriverMock::default("four_pages_register_mix");
    d.expect("Home");
    d.expect("foo");
    d.expect("foo");
    d.expect("foo");
    d.expect("baz");

    let mut m = PageManager::new(d, Box::new(home));

    m.update().unwrap();
    m.register(Box::new(foo));
    m.update().unwrap();
    m.register(Box::new(bar));
    m.dispatch(PageNavigation::Right).unwrap();
    m.register(Box::new(baz));
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
}

#[test]
fn sub_pages_iterator_no_subpages() {
    let home = PageMock::new("Home");
    let d = DisplayDriverMock::new("Update check", expect("Home"));
    let mut m = PageManager::new(d, Box::new(home));
    m.dispatch(PageNavigation::Home).unwrap();
    check_page_iteration("empty iterator", vec![], m.sub_iter());
}

#[test]
fn sub_pages_iterator_three_subpages() {
    let home = PageMock::new("Home");
    let foo = PageMock::new("foo");
    let bar = PageMock::new("bar");
    let baz = PageMock::new("baz");
    let d = DisplayDriverMock::new("Update check", expect("Home"));
    let mut m = PageManager::new(d, Box::new(home));
    m.register_sub(Box::new(foo));
    m.register(Box::new(bar));
    m.register(Box::new(baz));
    m.dispatch(PageNavigation::Home).unwrap();
    check_page_iteration("sub list", expect("foo bar baz"), m.sub_iter());
}

#[test]
fn three_pages_navigation_bool_returns() {
    let foo = PageMock::new("Foo");
    let bar = PageMock::new("Bar");
    let baz = PageMock::new("Baz");
    let d = DisplayDriverStub {};
    let mut m = PageManager::new(d, Box::new(foo));
    m.register(Box::new(bar));
    m.register(Box::new(baz));
    m.activate_home();
    assert!(m.activate_left(), "expected move to bar");
    assert!(m.activate_left(), "expected move to baz");
    assert!(m.activate_right(), "expected stay baz");
    assert!(m.activate_right(), "expected stay baz");
    assert!(!m.activate_right(), "expected move to bar");
    assert!(!m.activate_right(), "expected move to foo");
    assert!(m.activate_left(), "expected stay foo");
    assert!(m.activate_left(), "expected stay foo");
    assert!(!m.activate_left(), "expected stay bar");
    assert!(!m.activate_left(), "expected move to foo");
}

#[test]
fn startup_navigation() {
    let foo = PageMock::new("Foo");
    let startup = PageMock::new("Startup");
    let mut d = DisplayDriverMock::default("Start Navigation");
    d.expect("Startup");
    d.expect("Foo");
    let mut m = PageManager::new(d, Box::new(foo));
    m.register_startup(Box::new(startup));
    m.dispatch(PageNavigation::SystemStart).unwrap();
    m.dispatch(PageNavigation::Update).unwrap();
    m.dispatch(PageNavigation::SystemStop).unwrap();
}
#[test]
fn shutdown_navigation() {
    let foo = PageMock::new("Foo");
    let shutdown = PageMock::new("Shutdown");
    let mut d = DisplayDriverMock::default("Shutdown Navigation");
    d.expect("Foo");
    d.expect("Shutdown");
    let mut m = PageManager::new(d, Box::new(foo));
    m.register_shutdown(Box::new(shutdown));
    m.dispatch(PageNavigation::SystemStart).unwrap();
    m.dispatch(PageNavigation::Update).unwrap();
    m.dispatch(PageNavigation::SystemStop).unwrap();
}

#[test]
fn home_two_pages_and_two_subpages_and_two_subsubpages_navigation() {
    let home = PageMock::new("Home");
    let level_1_second = PageMock::new("level_1_second");
    let level_2_first = PageMock::new("level_2_first");
    let level_2_second = PageMock::new("level_2_second");
    let level_3_first = PageMock::new("level_3_first");
    let level_3_second = PageMock::new("level_3_second");

    let mut d = DisplayDriverMock::default("multi-level");
    d.expect("level_3_second");
    d.expect("Home");
    d.expect("level_1_second");
    d.expect("Home");
    d.expect("Home"); // try a subpage which is not below home
    d.expect("level_1_second");
    d.expect("level_2_first");
    d.expect("level_2_first"); // try a subpage which is not below
    d.expect("level_2_second");
    d.expect("level_3_second");
    d.expect("level_3_first");
    d.expect("level_3_first");
    d.expect("level_3_second");
    d.expect("level_2_second");
    d.expect("level_3_second"); // we have just two sub pages so we end up at the second

    let mut m = PageManager::new(d, Box::new(home));
    m.register(Box::new(level_1_second));
    m.register_sub(Box::new(level_2_first));
    m.register(Box::new(level_2_second));
    m.register_sub(Box::new(level_3_first));
    m.register(Box::new(level_3_second));

    m.update().unwrap();
    m.dispatch(PageNavigation::Home).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::NthSubpage(1)).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();

    m.dispatch(PageNavigation::NthSubpage(1)).unwrap();
    m.dispatch(PageNavigation::NthSubpage(1)).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::NthSubpage(2)).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
    m.dispatch(PageNavigation::NthSubpage(4)).unwrap();
}

#[test]
fn home_and_two_subpages_and_two_x_two_subsubpages_navigation() {
    let home = PageMock::new("Home");
    let level_2_first = PageMock::new("level_2_first");
    let level_2_second = PageMock::new("level_2_second");
    let level_31_first = PageMock::new("level_31_first");
    let level_31_second = PageMock::new("level_31_second");
    let level_32_first = PageMock::new("level_32_first");
    let level_32_second = PageMock::new("level_32_second");

    let mut d = DisplayDriverMock::default("multi-level");
    d.expect("Home");
    d.expect("level_2_first");
    d.expect("level_31_first");
    d.expect("level_31_second");
    d.expect("level_2_first");
    d.expect("level_2_second");
    d.expect("level_32_second");
    d.expect("level_32_first");
    d.expect("level_2_second");
    d.expect("Home");

    let mut m = PageManager::new(d, Box::new(home));
    m.register_sub(Box::new(level_2_first));
    m.register_sub(Box::new(level_31_first));
    m.register(Box::new(level_31_second));
    m.activate_up();
    m.register(Box::new(level_2_second));
    m.register_sub(Box::new(level_32_first));
    m.register(Box::new(level_32_second));

    m.dispatch(PageNavigation::Home).unwrap();
    m.dispatch(PageNavigation::NthSubpage(1)).unwrap();
    m.dispatch(PageNavigation::NthSubpage(1)).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
    m.dispatch(PageNavigation::Left).unwrap();
    m.dispatch(PageNavigation::NthSubpage(2)).unwrap();
    m.dispatch(PageNavigation::Right).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
}
#[test]
fn home_and_three_subpages() {
    let home = PageMock::new("Home");
    let level_2_first = PageMock::new("level_2_first");
    let level_3_first = PageMock::new("level_3_first");
    let level_4_first = PageMock::new("level_4_first");

    let mut d = DisplayDriverMock::default("multi-level");
    d.expect("Home");
    d.expect("level_2_first");
    d.expect("level_3_first");
    d.expect("level_4_first");
    d.expect("level_3_first");
    d.expect("level_2_first");
    d.expect("level_3_first");
    d.expect("Home");
    let mut m = PageManager::new(d, Box::new(home));
    m.register_sub(Box::new(level_2_first));
    m.register_sub(Box::new(level_3_first));
    m.register_sub(Box::new(level_4_first));

    m.dispatch(PageNavigation::Home).unwrap();
    m.dispatch(PageNavigation::NthSubpage(0)).unwrap();
    m.dispatch(PageNavigation::NthSubpage(0)).unwrap();
    m.dispatch(PageNavigation::NthSubpage(0)).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
    m.dispatch(PageNavigation::Up).unwrap();
    m.dispatch(PageNavigation::NthSubpage(0)).unwrap();
    m.dispatch(PageNavigation::Home).unwrap();
}
