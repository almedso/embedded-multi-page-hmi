use crate::*;

mod mocks {

    use super::*;

    pub struct DisplayDriverMock {
        expected_updates: Vec<String>,
        collected_updates: Vec<String>,
    }
    impl DisplayDriverMock {
        pub fn new() -> Self {
            DisplayDriverMock {
                expected_updates: Vec::new(),
                collected_updates: Vec::new(),
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
            assert_eq!(self.expected_updates, self.collected_updates);
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
}

mod page_manager {

    use super::super::*;
    use super::mocks::*;

    #[test]
    fn update_page_manager() {
        let p = PageMock::new("Foo");
        let mut d = DisplayDriverMock::new();
        d.expect("Foo");
        let mut m = PageManager::new(d, p);
        m.update();
    }
}
