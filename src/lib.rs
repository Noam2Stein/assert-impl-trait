#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![doc = include_str!("../README.md")]

mod assert_impl;
mod parse;
mod token_iter;
mod types;

/// A simple macro for making compile-time assertions that a type implements a
/// given trait.
///
/// This macro supports generic parameters via `for<...>` syntax and trait
/// bounds via `where ...` syntax.
///
/// # Examples
///
/// ```
/// use assert_impl_trait::assert_impl;
///
/// // Assert that `u8` implements `Clone`.
/// assert_impl!(u8: Clone);
///
/// // Assert that for any type `T` that implements `Clone`, `Vec<T>` also
/// // implements `Clone`.
/// assert_impl!(
///     for<T: Clone> {
///         Vec<T>: Clone,
///     }
/// );
///
/// // Assert that for any type `T` and any integer `N`:
/// // - If `T` implements `Clone`, `[T; N]` also implements `Clone`.
/// // - If `T` implements `Copy`, `[T; N]` also implements `Copy`.
/// // - If `T` is valid for a lifetime, `[T; N]` is also valid for it.
/// assert_impl!(
///     for<T, const N: usize> {
///         where T: Clone {
///             [T; N]: Clone,
///         }
///         where T: Copy {
///             [T; N]: Copy,
///         }
///
///         for<'a> where T: 'a {
///             [T; N]: 'a,
///         }
///     }
/// );
///
/// // Assert that `Debug` is a dyn-compatible trait.
/// assert_impl!(dyn core::fmt::Debug:);
/// ```
#[proc_macro]
pub fn assert_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    assert_impl::main(input)
}
