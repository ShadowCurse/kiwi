#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ $crate::replace_expr!($tts 1usize))*};
}

#[macro_export]
macro_rules! tuple_from_array {
    ($total:ident, $array:ident, $head:tt,) => {
        (
            <$head as ComponentRef>::from_raw_ptr($array[$total - 1]),
        )
    };

    ($total:ident, $array:ident, $head:tt, $($tail:tt,)*) => {
            (
                <$head as ComponentRef>::from_raw_ptr($array[$total - 1 - $crate::count_tts!($($tail)*)]),
                $crate::tuple_from_array!($total, $array, $($tail,)*),
            )
    };
}

#[macro_export]
macro_rules! static_assert {
    ($exp:expr) => {
        const __ASSERT: [0; 0 - !{$exp as usize}] = [];
    };
}
