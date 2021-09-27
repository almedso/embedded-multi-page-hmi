//! Home of several general purpose page implementations

mod basic;
mod enter_string;
mod menu;

// Re-exports
#[allow(unused_imports)]
pub use basic::{BasicPage, ShutdownPage, StartupPage, TextPage};
#[allow(unused_imports)]
pub use enter_string::EnterStringPage;
#[allow(unused_imports)]
pub use menu::MenuPage;
