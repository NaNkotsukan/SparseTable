use crate::sparsetable::{SparseTableEntryTrait, SparseTableSrcTrait, SparseTable, ArchivedSparseTable};
use crate::block::Block;
use crate::compare::{CompareTrait, MinMaxTrait};

use rkyv::{Archive, Deserialize, Serialize};
use bytecheck::CheckBytes;

#[derive(Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SparseTableEntry<T: MinMaxTrait + Copy> {
    pub min: T,
    pub max: T,
}

macro_rules! impl_sparse_table_entry {
    ($t:ty $(, $tr:ident ),* ) => {
        impl<T: MinMaxTrait + Copy + Sized $( + $tr<Archived = T>)*> SparseTableEntryTrait for $t {
            type Ret = SparseTableEntry<T>;
            fn with_cmp(&self, b: &Self) -> Self::Ret {
                Self::Ret {
                    min: MinMaxTrait::min(self.min, b.min),
                    max: MinMaxTrait::max(self.max, b.max)
                }
            }
        }
    };
}

impl_sparse_table_entry!(SparseTableEntry<T>);
impl_sparse_table_entry!(ArchivedSparseTableEntry<T>, Archive);

impl<T: MinMaxTrait + std::marker::Copy + Sized> SparseTableSrcTrait<SparseTableEntry<T>> for Block<T> {
    fn get_sparse_table_entry(&self) -> SparseTableEntry<T> {
        let (min, max) = unsafe{ self.query_unsafe(0, 15) };
        SparseTableEntry {
            min,
            max
        }
    }
}

#[derive(Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct RMQ<T: MinMaxTrait + Copy + Sized> {
    pub blocks: Box<[Block<T>]>,
    pub table: SparseTable<SparseTableEntry<T>>,
}

impl<T: MinMaxTrait + Copy + std::default::Default + Sized> RMQ<T> {
    pub fn new(arr: &[T]) -> Self {
        let len = arr.len();
        let mut blocks = Vec::with_capacity((len + 15) / 16);
        let mut i = 0;
        for block in arr.chunks(16) {
            blocks.push(Block::new(block));
        }
        let table = SparseTable::new(&blocks);
        Self {
            blocks: blocks.into_boxed_slice(),
            table
        }
    }
}

macro_rules! impl_rmq {
    ($t:ty $(, $tr:ident )* ) => {
        impl <T: MinMaxTrait + Copy + Sized $( + $tr<Archived = T>),*> $t{
            pub fn query(&self, l: usize, r: usize) -> (T, T) {
                let (l_block, l_offset) = (l / 16, l % 16);
                let (r_block, r_offset) = (r / 16, r % 16);
                if l_block == r_block {
                    self.blocks[l_block].query(l_offset, r_offset)
                } else {
                    let (min1, max1) = self.blocks[l_block].query(l_offset, 15);
                    let (min2, max2) = self.blocks[r_block].query(0, r_offset);
                    let min = MinMaxTrait::min(min1, min2);
                    let max = MinMaxTrait::max(max1, max2);
                    let l_block = l_block + 1;
                    if l_block == r_block {
                        (min, max)
                    } else {
                        let min_max = self.table.query(l_block, r_block - 1);
                        let (min3, max3) = (min_max.min, min_max.max);
                        (MinMaxTrait::min(min, min3), MinMaxTrait::max(max, max3))
                    }
                }
            }

            pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> (T, T) {
                let (l_block, l_offset) = (l / 16, l % 16);
                let (r_block, r_offset) = (r / 16, r % 16);
                if l_block == r_block {
                    self.blocks.get_unchecked(l_block).query_unsafe(l_offset, r_offset)
                } else {
                    let (min1, max1) = self.blocks.get_unchecked(l_block).query_unsafe(l_offset, 15);
                    let (min2, max2) = self.blocks.get_unchecked(r_block).query_unsafe(0, r_offset);
                    let min = MinMaxTrait::min(min1, min2);
                    let max = MinMaxTrait::max(max1, max2);
                    let l_block = l_block + 1;
                    if l_block == r_block {
                        (min, max)
                    } else {
                        let min_max = self.table.query_unsafe(l_block, r_block - 1);
                        let (min3, max3) = (min_max.min, min_max.max);
                        (MinMaxTrait::min(min, min3), MinMaxTrait::max(max, max3))
                    }
                }
            }
        }

        impl<T: MinMaxTrait + Copy + Sized $( + $tr<Archived = T>),*> std::ops::Index<usize> for $t {
            type Output = T;
            fn index(&self, idx: usize) -> &Self::Output {
                &self.blocks[idx / 16][idx % 16]
            }
        }
    };
}

impl_rmq!(RMQ<T>);
impl_rmq!(ArchivedRMQ<T>, Archive);

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;


    #[test]
    fn rmq_rmq_unsafe_test() {
        type T = u16;
        let num = 3000;
        let mut rng = rand::thread_rng();
        let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(0, T::MAX)).take(num).collect::<Vec<_>>();

        let rmq = RMQ::new(&arr);
        for i in 0..num {
            assert_eq!(rmq[i], arr[i]);
        }
        for i in 0..num {
            println!("{}", i);
            for j in i..num {
                assert_eq!(unsafe{rmq.query_unsafe(i, j)}, (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
            }
        }
    }

    #[test]
    fn rmq_rmq_test() {
        type T = u16;
        let num = 3000;
        let mut rng = rand::thread_rng();
        let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(0, T::MAX)).take(num).collect::<Vec<_>>();

        let rmq = RMQ::new(&arr);
        for i in 0..num {
            println!("{}", i);
            for j in i..num {
                assert_eq!(rmq.query(i, j), (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
            }
        }
    }


}
