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

impl CompareTrait for u64 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for u64 {}

impl CompareTrait for u32 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for u32 {}

impl CompareTrait for u16 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for u16 {}

impl CompareTrait for u8 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for u8 {}

impl CompareTrait for i64 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for i64 {}

impl CompareTrait for i32 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for i32 {}

impl CompareTrait for i16 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for i16 {}

impl CompareTrait for i8 {
    fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
impl MinMaxTrait for i8 {}


