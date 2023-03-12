use std::{any::TypeId, alloc::Layout};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub name: &'static str,
    pub drop: Option<fn(*mut ())>,
}

impl TypeInfo {
    pub const fn new<T: TypeDrop + 'static>() -> Self {
        let drop = if std::mem::needs_drop::<Self>() {
            Some(unsafe { std::mem::transmute(&<Self as TypeDrop>::type_drop) })
        } else {
            None
        };
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            name: std::any::type_name::<Self>(),
            drop,
        }
    }
}

pub trait TypeDrop: Sized {
    /// # Safety
    /// The pointer should point to the instance of the correct type.
    unsafe fn type_drop(component: *mut ()) {
        component.cast::<Self>().drop_in_place();
    }
}

impl<T> TypeDrop for T {}


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
