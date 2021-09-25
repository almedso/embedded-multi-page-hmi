use std::fmt::Debug;
use std::{cell::Cell, default::Default, str::FromStr};

/// A setting can be set and get
///
/// A setting can be multiple borrowed since the update function does not require a
/// &mut self mutating reference
///
/// # Example
///
/// ```
///     use embedded_multi_page_hmi::setting::{CellSetting, Setting};
///     let setting: CellSetting<f32> = Default::default();
///     let s1 = &setting;
///     let s2 = &setting;
///     assert_eq!(0.0f32, s1.get());
///     assert_eq!(0.0f32, s2.get());
///     s1.set(32.0);
///     assert_eq!(32.0f32, s1.get());
///     assert_eq!(32.0f32, s2.get());
/// ```
pub trait Setting {
    type Item: Copy;

    /// Set the value of the setting
    ///
    /// The set function does not require a `&mut self` parameter on purpose
    fn set(&self, value: Self::Item);

    /// Set the value of the setting obtained from string slice
    ///
    /// The set function does not require a `&mut self` parameter on purpose
    fn set_string(&self, value: &str);

    /// Get the value of the setting into a string slice
    fn get(&self) -> Self::Item;
}

#[derive(Default)]
pub struct CellSetting<T>(Cell<T>);

impl<T: Copy + FromStr> Setting for CellSetting<T>
where
    <T as FromStr>::Err: Debug,
{
    type Item = T;

    fn set(&self, value: Self::Item) {
        self.0.set(value);
    }

    fn get(&self) -> Self::Item {
        self.0.get()
    }

    fn set_string(&self, value: &str) {
        let v = T::from_str(value).unwrap();
        self.0.set(v);
    }
}
