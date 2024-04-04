use self::types::TypeId;

// pub mod flat_tuple;
pub mod macros;
pub mod types;

#[inline]
pub const fn static_sort<const N: usize>(
    mut values: [TypeId; N],
    mut low: isize,
    mut high: isize,
) -> [TypeId; N] {
    let range = high - low;
    if range <= 0 || range >= values.len() as isize {
        return values;
    }
    loop {
        let mut i = low;
        let mut j = high;
        let p = values[(low + ((high - low) >> 1)) as usize].0;
        loop {
            while values[i as usize].0 < p {
                i += 1;
            }
            while values[j as usize].0 > p {
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
