use std::any::TypeId;

pub trait Query<const L: usize> {
    const IDS: [TypeId; L];
    fn ids() -> &'static [TypeId] {
        &Self::IDS
    }
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! impl_query {
    ($($t:ident),*) => {
        impl<$($t),*> Query<{count_tts!($($t)*)}> for ($($t,)*)
        where
            $($t: 'static),*,
        {
            const IDS: [TypeId; count_tts!($($t)*)] = [$(TypeId::of::<$t>()),*];
        }
    };
}

impl_query!(C1);
impl_query!(C1, C2);
impl_query!(C1, C2, C4);
impl_query!(C1, C2, C4, C5);
impl_query!(C1, C2, C4, C5, C6);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn query() {
        assert_eq!(
            <(u8, bool, i32) as Query<3>>::ids(),
            &[
                TypeId::of::<u8>(),
                TypeId::of::<bool>(),
                TypeId::of::<i32>()
            ]
        );
    }
}
