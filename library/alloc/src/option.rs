#[doc(inline)]
pub use core::option::Option;

use crate::borrow::ToOwned;

impl<T: ToOwned> Option<T> {
    /// Maps an `Option<T>` to an `Option<T>` by cloning the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = "rust";
    /// let opt_x = Some(x);
    /// let owned = opt_x.owned();
    /// assert_eq!(owned, Some(String::from("rust")));
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[unstable(feature = "option_owned", issue = "none")]
    pub fn owned(self) -> Option<T> {
        self.map(|t| t.to_owned())
    }
}
