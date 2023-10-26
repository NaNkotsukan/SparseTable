pub trait CompareTrait {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering;
}

pub trait MinMaxTrait : std::marker::Sized + CompareTrait {
    fn min(a: Self, b: Self) -> Self {
        match Self::cmp(&a, &b) {
            std::cmp::Ordering::Greater => b,
            _ => a
        }
    }
    fn max(a: Self, b: Self) -> Self {
        match Self::cmp(&a, &b) {
            std::cmp::Ordering::Less => b,
            _ => a
        }
    }
}

// impl CompareTrait for u64 {
//     fn min(a: &Self, b: &Self) -> bool {
//         a < b
//     }
//     fn max(a: &Self, b: &Self) -> bool {
//         a > b
//     }
// }