use assert_impl_trait::assert_impl;

assert_impl!(u8: Copy);
assert_impl!(u8: Copy + Default, Vec<u8>: Clone,);

assert_impl!(for<'a, T: 'a> { &'a Vec<T>: IntoIterator<Item = &'a T> });

assert_impl!(
    u8: Copy + Default,
    std::marker::PhantomData<()>: std::any::Any,

    Vec<u8>: Clone,

    where u8: Copy {
        u16: Copy
    },
    for<T: ?Sized> where T: Clone {
        Vec<T>: Clone,
    }

    Vec<Vec<u8>>: Into<Vec<Vec<u8>>>,
    (Vec<Vec<u8>>,): Into<(Vec<Vec<u8>>,)>,

    for<const N: usize,> {
        [Vec<Vec<u8>>; N]: Into<[Vec<Vec<u8>>; N]>,

        for<const N2: usize,> {
            [Vec<Vec<u8>>; N2]: Into<[Vec<Vec<u8>>; N2]>,
            [Vec<Vec<u8>>; 3]: Into<[Vec<Vec<u8>>; 3]>,
        }
    },

    for<'a> where &'static Vec<u8>: Copy {}

    for<T: Clone> {
        T: Clone
    }

    dyn std::fmt::Debug:,
    dyn std::fmt::Debug + Send + 'static:,
    for<T> {
        dyn std::convert::AsRef<T>:,
    }

    for<T: ?Sized> {
        T: ?Sized,
    }

    [u8]: ?Sized + Send,

    u8: std::ops::Add,
    u8: std::ops::Add<Output = u8>,
);
