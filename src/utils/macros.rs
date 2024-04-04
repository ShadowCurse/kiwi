macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}
pub(crate) use count;

#[allow(unused)]
macro_rules! static_assert {
    ($exp:expr) => {
        const __ASSERT: [usize; 0 - !{ $exp } as usize] = [];
    };
}
#[allow(unused)]
pub(crate) use static_assert;
