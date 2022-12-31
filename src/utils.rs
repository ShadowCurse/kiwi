#[inline]
pub const fn static_sort<const N: usize>(
    mut values: [u64; N],
    mut low: isize,
    mut high: isize,
) -> [u64; N] {
    let range = high - low;
    if range <= 0 || range >= values.len() as isize {
        return values;
    }
    loop {
        let mut i = low;
        let mut j = high;
        let p = values[(low + ((high - low) >> 1)) as usize];
        loop {
            while values[i as usize] < p {
                i += 1;
            }
            while values[j as usize] > p {
                j -= 1;
            }
            if i <= j {
                if i != j {
                    let q = values[i as usize];
                    values[i as usize] = values[j as usize];
                    values[j as usize] = q;
                }
                i += 1;
                j -= 1;
            }
            if i > j {
                break;
            }
        }
        if j - low < high - i {
            if low < j {
                values = static_sort(values, low, j);
            }
            low = i;
        } else {
            if i < high {
                values = static_sort(values, i, high)
            }
            high = j;
        }
        if low >= high {
            break;
        }
    }
    values
}

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
            std::mem::transmute($array[$total - 1]),
        )
    };

    ($total:ident, $array:ident, $head:tt, $($tail:tt,)*) => {
            (
                std::mem::transmute($array[$total - $crate::count_tts!($($tail)*)]),
                $crate::tuple_from_array!($total, $array, $($tail,)*),
            )
    };
}
