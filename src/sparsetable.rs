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

// pub trait SparseTableTrait<T: SparseTableEntryTrait> {
//     fn query(&self, l: usize, r: usize) -> T;
//     unsafe fn query_unsafe(&self, l: usize, r: usize) -> T;    
// }
#[macro_export]
macro_rules! impl_sparse_table_wrap {
    ($t:ty, $cmp:ty, $out:ty $(, $tr:ident)?) => {
        // impl<T: SparseTableEntryTrait $( + $tr<Archived = T>)* + Sized> $t {
        //     pub fn query(&self, l: usize, r: usize) -> T::Ret {
        //         let len = r - l + 1;
        //         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
        //         let head = self.heads[d] as usize;
        //         let a = &self.entries[head + l];
        //         let b = &self.entries[head + r + 1 - (1 << d)];
        //         T::Ret::with_cmp(a, b)
        //     }

        //     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T::Ret {
        //         let len = r - l + 1;
        //         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
        //         let head = *self.heads.get_unchecked(d) as usize;
        //         let a = self.entries.get_unchecked(head + l);
        //         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
        //         T::with_cmp(a, b)
        //     }
        // }
        // impl<T: SparseTableEntryTrait $( + $tr<Archived = rkyv::Archived<$out>>)?> $t {
        //     pub fn query(&self, l: usize, r: usize) -> $out {
        //         let len = r - l + 1;
        //         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
        //         let head = self.heads[d] as usize;
        //         let a = &self.entries[head + l];
        //         let b = &self.entries[head + r + 1 - (1 << d)];
        //         a.with_cmp(b)
        //     }
        
        //     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> $out {
        //         let len = r - l + 1;
        //         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
        //         let head = *self.heads.get_unchecked(d) as usize;
        //         let a = self.entries.get_unchecked(head + l);
        //         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
        //         a.with_cmp(b)
        //     }
        // }
        impl $t {
            pub fn query(&self, l: usize, r: usize) -> $out {
                let len = r - l + 1;
                let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
                let head = self.heads[d] as usize;
                let a = &self.entries[head + l];
                let b = &self.entries[head + r + 1 - (1 << d)];
                <$cmp>::with_cmp(a, b)
            }
        
            pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> $out {
                let len = r - l + 1;
                let d = crate::common::get_msb_pos(len as u64) as usize - 1;
                let head = *self.heads.get_unchecked(d) as usize;
                let a = self.entries.get_unchecked(head + l);
                let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
                <$cmp>::with_cmp(a, b)
            }
        }
        
    };
}

#[macro_export]
macro_rules! impl_sparse_table {
    ($out:ty) => {
        use crate::sparsetable::SparseTableEntryTrait;
        crate::impl_sparse_table_wrap!(crate::sparsetable::SparseTable<$out>, $out, $out);
        crate::impl_sparse_table_wrap!(crate::sparsetable::ArchivedSparseTable<$out>, rkyv::Archived<$out>, $out, Archive);
    };
}

// impl SparseTableEntryTrait for SparseTableEntry {
//     type Ret = SparseTableEntry;
//     fn with_cmp(&self, b: &Self) -> Self::Ret {
//         Self::Ret {
//             min: std::cmp::min(self.min, b.min),
//             max: std::cmp::max(self.max, b.max)
//         }
//     }
// }

// impl SparseTableEntryTrait for Archived::<SparseTableEntry> {
//     type Ret = SparseTableEntry;
//     fn with_cmp(&self, b: &Self) -> Self::Ret {
//         Self::Ret {
//             min: std::cmp::min(self.min, b.min),
//             max: std::cmp::max(self.max, b.max)
//         }
//     }
// }

// // fn with_cmp<T: SparseTableEntryTrait + Archive>(a: &Archived::<T>, b: &Archived::<T>) -> SparseTableEntry {
// //     SparseTableEntry {
// //         min: 0,
// //         max: 0,
// //     }
// // }

// // impl ArchivedSparseTable<SparseTableEntry> {
// //     pub fn query(&self, l: usize, r: usize) -> SparseTableEntry {
// //         let len = r - l + 1;
// //         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
// //         let head = self.heads[d] as usize;
// //         let a = &self.entries[head + l];
// //         let b = &self.entries[head + r + 1 - (1 << d)];
// //         ArchivedSparseTableEntry::with_cmp(a, b)
// //     }

// //     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> SparseTableEntry {
// //         let len = r - l + 1;
// //         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
// //         let head = *self.heads.get_unchecked(d) as usize;
// //         let a = self.entries.get_unchecked(head + l);
// //         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
// //         ArchivedSparseTableEntry::with_cmp(a, b)
// //     }
// // }

// // impl SparseTable<SparseTableEntry> {
// //     pub fn query(&self, l: usize, r: usize) -> SparseTableEntry {
// //         let len = r - l + 1;
// //         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
// //         let head = self.heads[d] as usize;
// //         let a = &self.entries[head + l];
// //         let b = &self.entries[head + r + 1 - (1 << d)];
// //         SparseTableEntry::with_cmp(a, b)
// //     }

// //     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> SparseTableEntry {
// //         let len = r - l + 1;
// //         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
// //         let head = *self.heads.get_unchecked(d) as usize;
// //         let a = self.entries.get_unchecked(head + l);
// //         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
// //         SparseTableEntry::with_cmp(a, b)
// //     }
// // }

// impl<T: SparseTableEntryTrait + Archive<Archived = T>> ArchivedSparseTable<T> {
//     pub fn query(&self, l: usize, r: usize) -> T::Ret {
//         let len = r - l + 1;
//         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
//         let head = self.heads[d] as usize;
//         let a = &self.entries[head + l];
//         let b = &self.entries[head + r + 1 - (1 << d)];
//         a.with_cmp(b)
//         // <T as Archived<SparseTableEntry>>::with_cmp(a, b)
//         // a.with_cmp(b)
//     }

//     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T::Ret {
//         let len = r - l + 1;
//         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
//         let head = *self.heads.get_unchecked(d) as usize;
//         let a = self.entries.get_unchecked(head + l);
//         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
//         a.with_cmp(b)
//     }
// }

// impl<T: SparseTableEntryTrait<Ret = T>> SparseTable<T> {
//     pub fn query(&self, l: usize, r: usize) -> T {
//         let len = r - l + 1;
//         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
//         let head = self.heads[d] as usize;
//         let a = &self.entries[head + l];
//         let b = &self.entries[head + r + 1 - (1 << d)];
//         T::with_cmp(a, b)
//     }

//     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T {
//         let len = r - l + 1;
//         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
//         let head = *self.heads.get_unchecked(d) as usize;
//         let a = self.entries.get_unchecked(head + l);
//         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
//         a.with_cmp(b)
//     }
// }

// impl<T: SparseTableEntryTrait + Archive<Archived = ArchivedSparseTableEntry>> ArchivedSparseTable<T> {
//     pub fn query(&self, l: usize, r: usize) -> T::Ret {
//         let len = r - l + 1;
//         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
//         let head = self.heads[d] as usize;
//         let a = &self.entries[head + l];
//         let b = &self.entries[head + r + 1 - (1 << d)];
//         <T as ArchivedSparseTableEntry>::with_cmp(a, b)
//     }

//     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> SparseTableEntry {
//         let len = r - l + 1;
//         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
//         let head = *self.heads.get_unchecked(d) as usize;
//         let a = self.entries.get_unchecked(head + l);
//         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
//         a::with_cmp(b)
//     }
// }
// impl_sparse_table!(SparseTableEntry);
// // where
// //     // T: Archive<T>,
// //     <T as Archive>::Archived: SparseTableEntryTrait,
// // // `<sparsetable::SparseTableEntry as Archive>::Archived = sparsetable::SparseTableEntry
// //     T::Ret: Archive<Archived = T::Ret>,
// //     // T: Archive
// // {
// //     pub fn query(&self, l: usize, r: usize) -> T::Ret {
// //         let len = r - l + 1;
// //         let d = 64 - std::intrinsics::ctlz(len) as usize - 1;
// //         let head = self.heads[d] as usize;
// //         let a = &self.entries[head + l];
// //         let b = &self.entries[head + r + 1 - (1 << d)];
// //         T::with_cmp(a, b)
// //     }

// //     pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> T::Ret {
// //         let len = r - l + 1;
// //         let d = crate::common::get_msb_pos(len as u64) as usize - 1;
// //         let head = *self.heads.get_unchecked(d) as usize;
// //         let a = self.entries.get_unchecked(head + l);
// //         let b = self.entries.get_unchecked(head + r + 1 - (1 << d));
// //         T::with_cmp(a, b)
// //     }
// // }

// #[derive(Archive, Serialize)]
// #[archive_attr(derive(CheckBytes))]
// pub struct SparseTableEntry<T: std::cmp::Ord + Copy> {
//     min: T,
//     max: T,
// }

// macro_rules! impl_sparse_table_entry {
//     ($t:ty $(, $tr:ident ),* ) => {
//         impl<T: std::cmp::Ord + Copy + Sized $( + $tr<Archived = T>)*> SparseTableEntryTrait for $t {
//             type Ret = SparseTableEntry<T>;
//             fn with_cmp(&self, b: &Self) -> Self::Ret {
//                 Self::Ret {
//                     min: std::cmp::min(self.min, b.min),
//                     max: std::cmp::max(self.max, b.max)
//                 }
//             }
//         }
//     };
// }

// impl_sparse_table_entry!(SparseTableEntry<T>);
// impl_sparse_table_entry!(ArchivedSparseTableEntry<T>, Archive);
// crate::impl_sparse_table!(SparseTableEntry<u16>);

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn rmq_sparsetable_test() {
        // impl_sparse_table!(SparseTable<T>);
        // impl_sparse_table!(ArchivedSparseTable<T>, Archive);


        #[derive(Archive, Serialize)]
        #[archive_attr(derive(CheckBytes))]
        // #[archive_attr(derive(Clone))]
        // #[archive_attr(derive(Copy))]
        pub struct SparseTableEntry {
            min: u16,
            max: u16
        }
        macro_rules! impl_sparse_table_entry {
            ($t:ty $(, $tr:ident ),* ) => {
                impl SparseTableEntryTrait for $t {
                    type Ret = SparseTableEntry;
                    fn with_cmp(&self, b: &Self) -> Self::Ret {
                        Self::Ret {
                            min: std::cmp::min(self.min, b.min),
                            max: std::cmp::max(self.max, b.max)
                        }
                    }
                }
            };
        }
        impl_sparse_table_entry!(SparseTableEntry);
        // impl_sparse_table_entry!(ArchivedSparseTableEntry, Archive);

        impl SparseTableEntryTrait for Archived<SparseTableEntry> {
            type Ret = SparseTableEntry;
            fn with_cmp(&self, b: &Self) -> Self::Ret {
                Self::Ret {
                    min: std::cmp::min(self.min, b.min),
                    max: std::cmp::max(self.max, b.max)
                }
            }
        }

        // SparseTable::<SparseTableEntry>::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        // ArchivedSparseTable::<ArchivedSparseTableEntry>;
        type T = u16;
        let num = 3000;
        let mut rng = rand::thread_rng();
        let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(0, T::MAX)).take(num).collect::<Vec<_>>();


        // impl SparseTableEntryTrait for SparseTableEntry {
        //     fn with_cmp(a: &Self, b: &Self) -> Self {
        //         Self {
        //             min: std::cmp::min(a.min, b.min),
        //             max: std::cmp::max(a.max, b.max)
        //         }
        //     }
        // }
        impl SparseTableSrcTrait<SparseTableEntry> for T {
            fn get_sparse_table_entry(&self) -> SparseTableEntry {
                SparseTableEntry {
                    min: *self,
                    max: *self
                }
            }
        }

        impl_sparse_table!(SparseTableEntry);
                
        let table = SparseTable::<SparseTableEntry>::new(&arr);
        // table.query(0, 1);

        let mut serializer = rkyv::ser::serializers::AllocSerializer::<0>::default();
        rkyv::ser::Serializer::serialize_value(&mut serializer, &table).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<SparseTable::<SparseTableEntry>>(&bytes[..]) };
        // ArchivedSparseTable::<Archived<SparseTableEntry>>::query(&self, l, r);
        archived.query(0, 1);
        // Archived::<SparseTableEntry<u16>>::with_cmp(&archived.entries[0], &archived.entries[1]);
                // for i in 0..num {
        //     println!("{}", i);
        //     for j in i..num {
        //         assert_eq!(unsafe{table.query_unsafe(i, j).min}, *arr[i..j+1].iter().min().unwrap());
        //         assert_eq!(unsafe{table.query_unsafe(i, j).max}, *arr[i..j+1].iter().max().unwrap());
        //     }
        // }
    }

}

// impl_sparse_table!(ArchivedSparseTable<ArchivedSparseTableEntry>, Archive);