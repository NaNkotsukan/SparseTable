pub trait SparseTableEntryTrait {
    fn with_cmp(a: &Self, b: &Self) -> Self;
}

pub trait SparseTableSrcTrait<T: SparseTableEntryTrait> {
    fn get_sparse_table_entry(&self) -> T;
}

pub struct SparseTable<T: SparseTableEntryTrait> {
    entries: Box<[T]>,
    heads: Box<[usize]>,
}

impl<T: SparseTableEntryTrait> SparseTable<T> {
    pub fn new<S: SparseTableSrcTrait<T>>(arr: &[S]) -> Self {
        let len = arr.len();
        let mut heads = Vec::new();
        let mut window = 1;
        let mut last = 0;
        while window <= len {
            heads.push(last);
            last += len - (window - 1);
            window *= 2;
        }
        
        let mut entries = Vec::with_capacity(last);
        for x in arr {
            entries.push(x.get_sparse_table_entry());
        }
        let mut window = 1;
        for i in 1..heads.len() {
            let start = &heads[i];
            let end = heads.get(i + 1).unwrap_or(&last);
            let len = end - start;
            let head = heads[i - 1];
            for j in head..head + len {
                let a = &entries[j];
                let b = &entries[j + window];
                entries.push(T::with_cmp(a, b));
            }
            window *= 2;
        }
        Self {
            entries: entries.into_boxed_slice(),
            heads: heads.into_boxed_slice(),
        }
    }

    pub fn query(&self, l: usize, r: usize) -> T {
        let len = r - l + 1;
        let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
        let head = self.heads[d];
        let a = &self.entries[head + l];
        let b = &self.entries[head + r + 1 - (1 << d)];
        T::with_cmp(a, b)
    }

    pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T {
        let len = r - l + 1;
        let d = crate::common::get_msb_pos(len as u64) as usize - 1;
        let head = self.heads.get_unchecked(d);
        let a = self.entries.get_unchecked(head + l);
        let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
        T::with_cmp(a, b)
    }
}
