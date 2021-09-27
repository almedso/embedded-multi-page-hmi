pub mod basic;
pub mod enter_string;
pub mod menu;

// Re-exports
#[allow(unused_imports)]
pub use basic::{BasicPage, ShutdownPage, StartupPage, TextPage};
#[allow(unused_imports)]
pub use enter_string::EnterStringPage;
#[allow(unused_imports)]
pub use menu::MenuPage;
