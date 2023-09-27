use rkyv::{Archive, Deserialize, Serialize, Archived};
use bytecheck::CheckBytes;

pub trait SparseTableEntryTrait {
    type Ret : SparseTableEntryTrait;
    fn with_cmp(&self, b: &Self) -> Self::Ret ;
}

pub trait SparseTableSrcTrait<T: SparseTableEntryTrait> {
    fn get_sparse_table_entry(&self) -> T;
}

#[derive(Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SparseTable<T: SparseTableEntryTrait> {
    pub entries: Box<[T]>,
    pub heads: Box<[usize]>,
}

impl<T: SparseTableEntryTrait<Ret = T>> SparseTable<T> {
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
}


impl<T: SparseTableEntryTrait<Ret = T>> SparseTable<T> {
    pub fn query(&self, l: usize, r: usize) -> T::Ret {
        let len = r - l + 1;
        let d = 64 - std::intrinsics::ctlz(len) - 1;
        let head = self.heads[d];
        let a = &self.entries[head + l];
        let b = &self.entries[head + r + 1 - (1 << d)];
        a.with_cmp(b)
    }

    pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T::Ret {
        let len = r - l + 1;
        let d = crate::common::get_msb_pos(len as u64) as usize - 1;
        let head = *self.heads.get_unchecked(d);
        let a = self.entries.get_unchecked(head + l);
        let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
        a.with_cmp(b)
    }
}

impl<T: SparseTableEntryTrait + Archive<Archived = impl SparseTableEntryTrait> + Sized> ArchivedSparseTable<T> {
    pub fn query(&self, l: usize, r: usize) -> <<T as Archive>::Archived as SparseTableEntryTrait>::Ret {
        let len = r - l + 1;
        let d = 64 - std::intrinsics::ctlz(len) - 1;
        let head = self.heads[d] as usize;
        let a = &self.entries[head + l];
        let b = &self.entries[head + r + 1 - (1 << d)];
        a.with_cmp(b)
    }

    pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> <<T as Archive>::Archived as SparseTableEntryTrait>::Ret {
        let len = r - l + 1;
        let d = crate::common::get_msb_pos(len as u64) as usize - 1;
        let head = *self.heads.get_unchecked(d) as usize;
        let a = self.entries.get_unchecked(head + l);
        let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
        a.with_cmp(b)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn rmq_sparsetable_test() {
        type T = u16;
        let num = 3000;
        let mut rng = rand::thread_rng();
        let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(0, T::MAX)).take(num).collect::<Vec<_>>();

        struct SparseTableEntry {
            min: T,
            max: T
        }
        impl SparseTableEntryTrait for SparseTableEntry {
            type Ret = Self;
            fn with_cmp(&self, b: &Self) -> Self {
                Self {
                    min: std::cmp::min(self.min, b.min),
                    max: std::cmp::max(self.max, b.max)
                }
            }
        }
        impl SparseTableSrcTrait<SparseTableEntry> for T {
            fn get_sparse_table_entry(&self) -> SparseTableEntry {
                SparseTableEntry {
                    min: *self,
                    max: *self
                }
            }
        }
                
        let table = SparseTable::<SparseTableEntry>::new(&arr);
        for i in 0..num {
            println!("{}", i);
            for j in i..num {
                assert_eq!(unsafe{table.query_unsafe(i, j).min}, *arr[i..j+1].iter().min().unwrap());
                assert_eq!(unsafe{table.query_unsafe(i, j).max}, *arr[i..j+1].iter().max().unwrap());
            }
        }
    }

}
