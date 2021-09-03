mod text_page {

    use super::super::*;

    #[test]
    fn check_title_and_content() {
        let sut = TextPage::new(BasicPage::new("MyTitle", None), "MyContent");
        assert_eq!(sut.title(), "MyTitle");
        assert_eq!(sut.text, "MyContent");
    }

    #[test]
    fn check_lifetime() {
        let mut sut = TextPage::new(
            BasicPage::new("MyTitle", Some(PageLifetime::new(PageNavigation::Home, 2))),
            "MyContent",
        );
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Update);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Home);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Update);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Home);

        let mut sut = TextPage::new(BasicPage::new("MyTitle", None), "MyContent");
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Update);
    }
}

mod startup_page {

    use super::super::*;

    #[test]
    fn check_title_and_content() {
        let sut = StartupPage::new("MyContent", 2);
        assert_eq!(sut.title(), "Startup");
        assert_eq!(sut.0.text, "MyContent");
    }

    #[test]
    fn check_lifetime() {
        let mut sut = StartupPage::new("MyContent", 2);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::SystemStart);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::Home);
    }
}

mod shutdown_page {

    use super::super::*;

    #[test]
    fn check_title_and_content() {
        let sut = ShutdownPage::new("MyContent", 2);
        assert_eq!(sut.title(), "Shutdown");
        assert_eq!(sut.0.text, "MyContent");
    }

    #[test]
    fn check_lifetime() {
        let mut sut = ShutdownPage::new("MyContent", 2);
        assert_eq!(sut.update(None).unwrap(), PageNavigation::SystemStop);
        assert!(sut.update(None).is_err());
    }
}
