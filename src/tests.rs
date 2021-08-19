use crate::*;

mod mocks {

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
        iterator: Iter<'a, Box<dyn PageInterface<DisplayDriverMock>>>,
    ) {
        let mut d = DisplayDriverMock::new(context, expected);
        for i in iterator {
            i.display(&mut d);
        }
    }
}

mod page_manager {

    use super::super::*;
    use super::mocks::*;

    #[test]
    fn update_page_manager() {
        let foo = PageMock::new("Foo");
        let mut d = DisplayDriverMock::default("update");
        d.expect("Foo");
        let mut m = PageManager::new(d, Box::new(foo));
        m.update();
    }

    #[test]
    fn two_pages_navigation() {
        let foo = PageMock::new("foo");
        let bar = PageMock::new("bar");
        let mut d = DisplayDriverMock::default("two_page_navigation");
        d.expect("bar");
        d.expect("bar");
        d.expect("foo");
        d.expect("bar");
        d.expect("foo");
        d.expect("bar");
        d.expect("foo");
        d.expect("foo");
        d.expect("bar");
        d.expect("bar");
        d.expect("foo");
        d.expect("foo");
        let mut m = PageManager::new(d, Box::new(foo));
        m.register(Box::new(bar));
        m.update();
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
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
        d.expect("baz");

        d.expect("bar");
        d.expect("foo");
        d.expect("Home");
        d.expect("Home");

        d.expect("foo");
        d.expect("bar");
        d.expect("baz");
        d.expect("baz");

        d.expect("bar");
        d.expect("foo");
        d.expect("Home");
        d.expect("Home");

        let mut m = PageManager::new(d, Box::new(home));
        m.register(Box::new(foo));
        m.register(Box::new(bar));
        m.register(Box::new(baz));
        m.update();
        m.dispatch(Interaction::Home);

        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);

        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);

        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);
        m.dispatch(Interaction::Next);

        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Previous);
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

        m.update();
        m.register(Box::new(foo));
        m.update();
        m.register(Box::new(bar));
        m.dispatch(Interaction::Previous);
        m.register(Box::new(baz));
        m.dispatch(Interaction::Previous);
        m.dispatch(Interaction::Next);
    }

    #[test]
    fn four_pages_iterator_init() {
        let home = PageMock::new("Home");
        let foo = PageMock::new("foo");
        let bar = PageMock::new("bar");
        let baz = PageMock::new("baz");
        let d = DisplayDriverMock::new("Update check", expect("baz"));
        let mut m = PageManager::new(d, Box::new(home));
        m.register(Box::new(foo));
        m.register(Box::new(bar));
        m.register(Box::new(baz));

        m.update();

        check_page_iteration("iterator_init-forward", vec![], m.forward_iter());
        check_page_iteration(
            "iterator_init-backward",
            expect("bar foo Home"),
            m.backward_iter(),
        );
    }

    #[test]
    fn four_pages_iterator_one_previous() {
        let home = PageMock::new("Home");
        let foo = PageMock::new("foo");
        let bar = PageMock::new("bar");
        let baz = PageMock::new("baz");
        let d = DisplayDriverMock::new("Update check", expect("baz bar"));
        let mut m = PageManager::new(d, Box::new(home));
        m.register(Box::new(foo));
        m.register(Box::new(bar));
        m.register(Box::new(baz));
        m.update();
        m.activate_previous();
        m.update();
        check_page_iteration("forward list", expect("baz"), m.forward_iter());
        check_page_iteration("backward list", expect("foo Home"), m.backward_iter());
    }

    #[test]
    fn four_pages_iterator_two_previous() {
        let home = PageMock::new("Home");
        let foo = PageMock::new("foo");
        let bar = PageMock::new("bar");
        let baz = PageMock::new("baz");
        let d = DisplayDriverMock::new("Update check", expect("baz bar foo"));
        let mut m = PageManager::new(d, Box::new(home));
        m.register(Box::new(foo));
        m.register(Box::new(bar));
        m.register(Box::new(baz));
        m.update();
        m.activate_previous();
        m.update();
        m.activate_previous();
        m.update();
        check_page_iteration("forward list", expect("bar baz"), m.forward_iter());
        check_page_iteration("backward list", expect("Home"), m.backward_iter());
    }

    #[test]
    fn four_pages_iterator_three_previous() {
        let home = PageMock::new("Home");
        let foo = PageMock::new("foo");
        let bar = PageMock::new("bar");
        let baz = PageMock::new("baz");
        let d = DisplayDriverMock::new("Update check", expect("baz bar foo Home"));
        let mut m = PageManager::new(d, Box::new(home));
        m.register(Box::new(foo));
        m.register(Box::new(bar));
        m.register(Box::new(baz));
        m.update();
        m.activate_previous();
        m.update();
        m.activate_previous();
        m.update();
        m.activate_previous();
        m.update();
        check_page_iteration("forward list", expect("foo bar baz"), m.forward_iter());
        check_page_iteration("backward list", vec![], m.backward_iter());
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
        assert!(m.activate_next(), "expected move to bar");
        assert!(m.activate_next(), "expected move to baz");
        assert!(!m.activate_next(), "expected stay baz");
        assert!(!m.activate_next(), "expected stay baz");
        assert!(m.activate_previous(), "expected move to bar");
        assert!(m.activate_previous(), "expected move to foo");
        assert!(!m.activate_previous(), "expected stay foo");
        assert!(!m.activate_previous(), "expected stay foo");
        assert!(m.activate_next(), "expected stay bar");
        assert!(m.activate_previous(), "expected move to foo");
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
        m.dispatch(Interaction::SystemStart);
        m.dispatch(Interaction::Action);
        m.dispatch(Interaction::SystemStop);
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
        m.dispatch(Interaction::SystemStart);
        m.dispatch(Interaction::Action);
        m.dispatch(Interaction::SystemStop);
    }
}
