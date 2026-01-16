# `assert-impl-trait`

This crate provides a simple `assert_impl` macro for making compile-time
assertions that a type implements a given trait.

The macro supports generic parameters via `for<...>` syntax and trait bounds via
`where ...` syntax.

The macro is useful for crates that need tests to ensure their types implement
the correct traits, and is aspecially useful if those types contain generics.

The macro can also be used to assert that a trait is dyn-compatible.

## Examples

```rust
use assert_impl_trait::assert_impl;

// Assert that `u8` implements `Clone`.
assert_impl!(u8: Clone);

// Assert that for any type `T` that implements `Clone`, `Vec<T>` also
// implements `Clone`.
assert_impl!(
    for<T: Clone> {
        Vec<T>: Clone,
    }
);

// Assert that for any type `T` and any integer `N`:
// - If `T` implements `Clone`, `[T; N]` also implements `Clone`.
// - If `T` implements `Copy`, `[T; N]` also implements `Copy`.
// - If `T` is valid for a lifetime, `[T; N]` is also valid for it.
assert_impl!(
    for<T, const N: usize> {
        where T: Clone {
            [T; N]: Clone,
        }
        where T: Copy {
            [T; N]: Copy,
        }

        for<'a> where T: 'a {
            [T; N]: 'a,
        }
    }
);

// Assert that `Debug` is a dyn-compatible trait.
assert_impl!(dyn core::fmt::Debug:);
```

## License

Licensed under either Apache License Version 2.0 or MIT license at your option.
