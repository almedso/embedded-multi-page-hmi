/// The PageManager is responsible for switching among pages while
/// pages do not know about other pages.
/// The PageManager also dispatches events and updates the current page.
///
/// Only a "home" page is mandatory. Any other pages are optional.
/// Startup and Shutdown pages are purely information pages and not
/// activated only by SystemStartup and SystemShutdown events.
///
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
// * `a`- is root page
// * `b, c` - are pages on the same level like `a` reachable via right link of `a`
// * `d, e, f`- are sub-pages of `a` reachable via down link of `a`
//
use core::mem;

use super::*;

pub struct PageManager<D> {
    display: D,
    page: Box<dyn PageInterface<D>>,
    left: Link<Box<dyn PageInterface<D>>>,
    right: Link<Box<dyn PageInterface<D>>>,
    up: Link<Box<dyn PageInterface<D>>>,
    down: Link<Box<dyn PageInterface<D>>>,
    startup: Option<Box<dyn PageInterface<D>>>,
    shutdown: Option<Box<dyn PageInterface<D>>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    page: T,
    left: Link<T>,
    right: Link<T>,
    down: Link<T>,
    up: Link<T>,
}

impl<'a, D> PageManager<D> {
    /// PageManager Constructor
    ///
    /// Arguments
    ///
    /// * `display`: The display data structure where all output is rendered to
    ///   The display data structure and the logic attached makes the rendered
    ///   output appear on some output facility viewable by a human.
    /// * `home`: The "home" page. There must be at least one page. Other pages
    ///    are added by register_* calls.
    pub fn new(display: D, home: Box<dyn PageInterface<D>>) -> Self {
        PageManager::<D> {
            display,
            page: home,
            left: None,
            right: None,
            up: None,
            down: None,
            startup: None,
            shutdown: None,
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
    pub fn register(&mut self, page: Box<dyn PageInterface<D>>) {
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
    pub fn register_sub(&mut self, page: Box<dyn PageInterface<D>>) {
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
    pub fn register_startup(&mut self, page: Box<dyn PageInterface<D>>) {
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
    pub fn register_shutdown(&mut self, page: Box<dyn PageInterface<D>>) {
        self.shutdown = Some(page);
    }

    fn push_left(
        &mut self,
        page: Box<dyn PageInterface<D>>,
        up: Link<Box<dyn PageInterface<D>>>,
        down: Link<Box<dyn PageInterface<D>>>,
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
        page: Box<dyn PageInterface<D>>,
        up: Link<Box<dyn PageInterface<D>>>,
        down: Link<Box<dyn PageInterface<D>>>,
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
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
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
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
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
        page: Box<dyn PageInterface<D>>,
        left: Link<Box<dyn PageInterface<D>>>,
        right: Link<Box<dyn PageInterface<D>>>,
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
        page: Box<dyn PageInterface<D>>,
        left: Link<Box<dyn PageInterface<D>>>,
        right: Link<Box<dyn PageInterface<D>>>,
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
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
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
        Box<dyn PageInterface<D>>,
        Link<Box<dyn PageInterface<D>>>,
        Link<Box<dyn PageInterface<D>>>,
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
        let navigation = self.page.dispatch(interaction);
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
            PageNavigation::SystemStart => match &mut self.startup {
                Some(page) => {
                    navigation = page.update(None)?;
                    page.display(&mut self.display);
                }
                None => (),
            },
            PageNavigation::SystemStop => match &mut self.shutdown {
                Some(page) => {
                    navigation = page.update(None)?;
                    page.display(&mut self.display);
                }
                None => (),
            },
            PageNavigation::Left => {
                self.activate_left();
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
        Ok(navigation)
    }
}

impl<D> Drop for PageManager<D> {
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

impl<D> PageManager<D> {
    pub fn sub_iter(&self) -> SubPageIterator<Box<dyn PageInterface<D>>> {
        SubPageIterator {
            left: self.down.as_deref(),
        }
    }
}

impl<'a, D> Iterator for SubPageIterator<'a, Box<dyn PageInterface<D>>> {
    type Item = &'a Box<dyn PageInterface<D>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.left.map(|node| {
            self.left = node.left.as_deref();
            &node.page
        })
    }
}

#[cfg(test)]
mod tests;
