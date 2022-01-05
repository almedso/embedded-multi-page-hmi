use super::*;
use core::mem;
use core::cell::RefCell;
use core::rc::Rc;

/// The PageManager is responsible for switching among pages while
/// pages do not know about other pages.
/// The PageManager also dispatches events and updates the current page.
///
/// Only a "home" page is mandatory. Any other pages are optional.
/// Startup and Shutdown pages are purely information pages and not
/// activated only by SystemStartup and SystemShutdown events.
///
/// h2. Example
///
/// ```ignore
/// let mut input = SomeInput(); // artificial code
/// let display = SomeDisplay::new(); // artificial code
/// let home = HomePage::new("!!! This is the home page !!!");
///
/// let mut m = PageManager::new(display, Box::new(home));
/// // Optional startup page has a mandatory lifetime.
/// let startup = StartupPage::new("Welcome message", 8);
/// m.register_startup(Box::new(startup));
/// // Optional Shutdown page has a mandatory lifetime.
/// let shutdown = ShutdownPage::new("Bye bye message", 10);
/// m.register_shutdown(Box::new(shutdown));
/// // Additional pages reachable by next button
/// // A predefined Information text page with lifetime
/// let page_one = TextPage::new(
///     BasicPage::new("First", Some(PageLifetime::new(PageNavigation::Left, 6))),
///     "First Information Page with 3 seconds lifetime; moving to next page",
/// );
/// m.register(Box::new(page_one));
///
/// // Enter the event loop
/// let mut navigation = m.dispatch(PageNavigation::SystemStart).unwrap();
/// loop {
///     match input.next() {
///          None => m.dispatch(navigation),
///          Some(interaction) => m.dispatch_interaction(interaction),
///      }
/// }
/// ```

// h2. Implementation Note
//
// There is only one page active at a time, that dispatches events
// (stored in page variable). other pages are activate by updating links
// (in respective directions).
//
// * Tree structures of pages are modeled by left, right, up and down links.
// * Tree root is where both up and left links are empty.
// * From the active page, all other pages can be navigated to
//
// h3. Example Structure
//
// ```ignore
//  a-------------------b-----------c
//  |                   |
//  d-----e-----f       o-p
//  |     |     |
//  g-h-i j-k-l m-n
// ```
// * `a`- is root page (empty left node and empty up node)
// * `b, c` - are pages on the same level like `a` reachable via right link of `a`
// * `d, e, f`- are sub-pages of `a` reachable via down link of `a`
//
pub struct PageManager<'a, D> {
    display: D,
    page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
    left: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    right: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    up: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    down: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    startup: Option<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    shutdown: Option<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    state: PageManagerState,
}

unsafe impl<D> Send for PageManager<'_, D> {}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    page: T,
    left: Link<T>,
    right: Link<T>,
    down: Link<T>,
    up: Link<T>,
}

enum PageManagerState {
    Startup,
    Operational,
    Shutdown,
}

impl<'a, D> PageManager<'a, D> {
    /// PageManager Constructor
    ///
    /// Arguments
    ///
    /// * `display`: The display data structure where all output is rendered to
    ///   The display data structure and the logic attached makes the rendered
    ///   output appear on some output facility viewable by a human.
    /// * `home`: The "home" page. There must be at least one page. Other pages
    ///    are added by register_* calls.
    pub fn new(display: D, home: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>) -> Self {
        PageManager::<D> {
            display,
            page: home,
            left: None,
            right: None,
            up: None,
            down: None,
            startup: None,
            shutdown: None,
            state: PageManagerState::Startup,
        }
    }

    /// Update the content of the active page on the display
    ///
    /// Potentially initiate a page change before displaying, since the
    /// update responsibility is the responsibility of the specific active page
    pub fn update(&mut self) -> Result<(), PageError> {
        // menu pages need submenu titles
        let iter = Box::new(SubPageIterator {
            left: self.down.as_deref(),
        });
        let navigation = self.page.update(Some(Box::new(iter.map(|p| p.title()))))?;

        // in case the page requires another page to navigate this needs to be performed
        if navigation != PageNavigation::Update {
            self.dispatch(navigation)?;
        }

        self.page.display(&mut self.display);
        Ok(())
    }

    /// Register a new page
    ///
    /// The page is registered in the "left" direction of the
    /// active page. The registered page will be the new active page.
    ///
    /// Arguments
    ///
    /// * `page` - The page to be registered and activated.
    pub fn register(&mut self, page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>) {
        self.push_left(page, None, None);
        self.activate_left();
    }

    /// Register a new sub page
    ///
    /// The page is registered in the "down" direction of the
    /// active page. The registered page will be the new active page.
    ///
    /// Arguments
    ///
    /// * `page`: - The page to be registered and activated.
    pub fn register_sub(&mut self, page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>) {
        self.push_down(page, None, None);
        self.activate_down();
    }

    /// Register a startup page
    ///
    /// There can be just one startup page. Multiple calls to this function
    /// overwrite the previously set startup page.
    ///
    /// Arguments
    ///
    /// * `page`: - The page that should serve for startup.
    pub fn register_startup(&mut self, page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>) {
        self.startup = Some(page);
    }

    /// Register a shutdown page
    ///
    /// There can be just one shutdown page. Multiple calls to this function
    /// overwrite the previously set shutdown page.
    ///
    /// Arguments
    ///
    /// * `page`: - The page that should serve for startup.
    pub fn register_shutdown(&mut self, page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>) {
        self.shutdown = Some(page);
    }

    fn push_left(
        &mut self,
        page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        up: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        down: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    ) {
        let new_node = Box::new(Node {
            page,
            left: self.left.take(),
            right: None,
            down,
            up,
        });
        self.left = Some(new_node);
    }

    fn push_right(
        &mut self,
        page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        up: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        down: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    ) {
        let new_node = Box::new(Node {
            page,
            left: None,
            right: self.right.take(),
            up,
            down,
        });
        self.right = Some(new_node);
    }

    fn pop_left(
        &mut self,
    ) -> Option<(
        Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    )> {
        self.left.take().map(|node| {
            let mut node = node;
            self.left = node.left;
            (node.page, node.up.take(), node.down.take())
        })
    }

    fn pop_right(
        &mut self,
    ) -> Option<(
        Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    )> {
        self.right.take().map(|node| {
            let mut node = node;
            self.right = node.right;
            (node.page, node.up.take(), node.down.take())
        })
    }

    /// Navigate to the left page
    /// If there is no left page it returns false and activate page is unchanged
    fn activate_left(&mut self) -> bool {
        match self.pop_left() {
            None => false,
            Some((page, up, down)) => {
                let page = mem::replace(&mut self.page, page);
                let new_up = self.up.take();
                let new_down = self.down.take();
                self.push_right(page, new_up, new_down);
                self.up = up;
                self.down = down;
                true
            }
        }
    }

    /// Navigate to the right page
    /// If there is no right page it returns false and activate page is unchanged
    fn activate_right(&mut self) -> bool {
        match self.pop_right() {
            None => false,
            Some((page, up, down)) => {
                let page = mem::replace(&mut self.page, page);
                let new_up = self.up.take();
                let new_down = self.down.take();
                self.push_left(page, new_up, new_down);
                self.up = up;
                self.down = down;
                true
            }
        }
    }

    fn activate_most_right(&mut self) {
        while self.activate_right() {}
    }

    fn push_down(
        &mut self,
        page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        left: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        right: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    ) {
        let new_node = Box::new(Node {
            page,
            up: None,
            down: self.down.take(),
            left,
            right,
        });
        self.down = Some(new_node);
    }

    fn push_up(
        &mut self,
        page: Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        left: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        right: Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    ) {
        let new_node = Box::new(Node {
            page,
            up: self.up.take(),
            down: None,
            left,
            right,
        });
        self.up = Some(new_node);
    }

    fn pop_down(
        &mut self,
    ) -> Option<(
        Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    )> {
        self.down.take().map(|node| {
            let mut node = node;
            self.down = node.down;
            (node.page, node.left.take(), node.right.take())
        })
    }

    fn pop_up(
        &mut self,
    ) -> Option<(
        Rc<dyn RefCell<dyn PageInterface<D>> + 'a>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
        Link<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>>,
    )> {
        self.up.take().map(|node| {
            let mut node = node;
            self.up = node.up;
            (node.page, node.left.take(), node.right.take())
        })
    }

    fn activate_down(&mut self) -> bool {
        match self.pop_down() {
            None => false,
            Some((page, left, right)) => {
                let page = mem::replace(&mut self.page, page);
                let new_left = self.left.take();
                let new_right = self.right.take();
                self.push_up(page, new_left, new_right);
                self.left = left;
                self.right = right;
                true
            }
        }
    }

    fn activate_up(&mut self) -> bool {
        self.activate_most_right();
        match self.pop_up() {
            None => false,
            Some((page, left, right)) => {
                let page = mem::replace(&mut self.page, page);
                let new_left = self.left.take();
                let new_right = self.right.take();
                self.push_down(page, new_left, new_right);
                self.left = left;
                self.right = right;
                true
            }
        }
    }

    fn activate_home(&mut self) {
        while self.activate_up() {}
        self.activate_most_right();
    }

    /// Dispatch an interaction event
    ///
    /// Let the active page process the interaction event and eventually turn
    /// The interaction event into a executed page navigation
    ///
    /// Arguments
    ///
    /// * `interaction`: - The interaction event to dispatch

    pub fn dispatch_interaction(
        &mut self,
        interaction: Interaction,
    ) -> Result<PageNavigation, PageError> {
        let navigation = match self.state {
            PageManagerState::Startup => match &mut self.startup {
                None => self.page.dispatch(interaction),
                Some(x) => x.dispatch(interaction),
            },
            PageManagerState::Operational => self.page.dispatch(interaction),
            PageManagerState::Shutdown => match &mut self.shutdown {
                None => self.page.dispatch(interaction),
                Some(x) => x.dispatch(interaction),
            },
        };
        self.dispatch(navigation)
    }

    /// Dispatch a navigation event
    ///
    /// The event can cause a change of the active page or
    /// lead to an update of the active page content.
    ///
    /// Arguments
    ///
    /// * `navigation`: - The navigation event to dispatch
    pub fn dispatch(&mut self, navigation: PageNavigation) -> Result<PageNavigation, PageError> {
        let mut navigation = navigation;
        match navigation {
            PageNavigation::SystemStart => {
                self.activate_home(); // reset the ordinary page structure to home in case there is no startup page
                match &mut self.startup {
                    Some(page) => {
                        navigation = page.update(None)?;
                        page.display(&mut self.display);
                    }
                    None => (),
                }
            }
            PageNavigation::SystemStop => match &mut self.shutdown {
                Some(page) => {
                    navigation = page.update(None)?;
                    page.display(&mut self.display);
                }
                None => (),
            },
            PageNavigation::Left => {
                // when navigating left, we turn around at the end; in case there is no previous navigation
                if !self.activate_left() {
                    self.activate_most_right();
                }
                self.update()?;
                navigation = PageNavigation::Update;
            }
            PageNavigation::Right => {
                self.activate_right();
                self.update()?;
                navigation = PageNavigation::Update;
            }
            PageNavigation::Home => {
                self.activate_home();
                self.update()?;
                navigation = PageNavigation::Update;
            }
            PageNavigation::Up => {
                self.activate_up();
                self.update()?;
                navigation = PageNavigation::Update;
            }
            PageNavigation::NthSubpage(index) => {
                self.activate_down();
                let mut index: usize = index;
                while index > 1 {
                    self.activate_left();
                    index -= 1;
                }
                self.update()?;
                navigation = PageNavigation::Update;
            }
            PageNavigation::Update => {
                self.update()?;
            }
        };

        // update the internal state for Correct HMI interaction update
        match navigation {
            PageNavigation::SystemStart => self.state = PageManagerState::Startup,
            PageNavigation::SystemStop => self.state = PageManagerState::Shutdown,
            _ => self.state = PageManagerState::Operational,
        }

        Ok(navigation)
    }
}

impl<'a, D> Drop for PageManager<'a, D> {
    /// TODO - update to remove everything
    fn drop(&mut self) {
        // forward list
        let mut cur_horizontal = self.left.take();
        while let Some(mut boxed_node) = cur_horizontal {
            cur_horizontal = boxed_node.left.take();
        }
        // backward list
        let mut cur_horizontal = self.right.take();
        while let Some(mut boxed_node) = cur_horizontal {
            cur_horizontal = boxed_node.left.take();
        }
    }
}

pub struct SubPageIterator<'a, P> {
    left: Option<&'a Node<P>>,
}

impl<'a, D> PageManager<'a, D> {
    pub fn sub_iter(&self) -> SubPageIterator<Rc<dyn RefCell<dyn PageInterface<D>> + 'a>> {
        SubPageIterator {
            left: self.down.as_deref(),
        }
    }
}

impl<'a, D> Iterator for SubPageIterator<'a, Rc<dyn RefCell<dyn PageInterface<D>> + 'a>> {
    type Item = &'a Rc<dyn RefCell<dyn PageInterface<D>> + 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.left.map(|node| {
            self.left = node.left.as_deref();
            &node.page
        })
    }
}

#[cfg(test)]
mod tests;
