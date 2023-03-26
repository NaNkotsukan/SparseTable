#[cfg(target_arch = "x86_64")]
use smallvec::{SmallVec, smallvec};
use std::{arch::x86_64::*, marker::PhantomData, default};
use rkyv::{Archive, Deserialize, Serialize};
use bytecheck::CheckBytes;

#[derive(Archive, Serialize)]
pub struct Min;

#[derive(Archive, Serialize)]
pub struct Max;

pub trait RMQType {
    fn cmp<T: std::cmp::PartialOrd>(l: &T, r: &T) -> bool;
}

impl RMQType for Min {
    fn cmp<T: std::cmp::PartialOrd>(l: &T, r: &T) -> bool {
        l < r
    }
}
impl RMQType for Max {
    fn cmp<T: std::cmp::PartialOrd>(l: &T, r: &T) -> bool {
        l > r
    }
}

#[derive(Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct RMQBlock<C: RMQType> {
    val: [u16; 8],
    _cmp: PhantomData<C>
}

impl<C: RMQType> RMQBlock<C> {
    pub fn new<T: std::cmp::PartialOrd>(arr:&[T]) -> Self {
        assert!(arr.len() <= 16);
        let len = arr.len();
        let mut st = SmallVec::<[i8; 16]>::new();
        let mut g: [i8; 16] = [0; 16];
        for i in 0..len {
            loop {
                let last = st.last();
                match last {
                    Some(x) => {
                        if C::cmp(&arr[*x as usize], &arr[i]) {
                            g[i] = *x;
                            break;
                        } else {
                            st.pop();
                        }
                    }
                    None => {
                        g[i] = -1i8;
                        break;
                    }
                };
            };
            st.push(i as i8);
        }
        for i in len..16 {
            g[i] = len as i8 - 1;
        }
        
        let mut l: [u16; 16] = [0; 16];
        for i in 1..16 {
            let g_i = g[i];
            if g_i == -1 {
                l[i] = 0;
            } else {
                l[i] = l[g_i as usize] | (1 << g_i);
            }
        }
        let mut val = [0; 8];
        for i in 0..8 {
            let x = l[i] << 15 - i | l[15 - i];
            val[i] = x.reverse_bits() >> 1;
        }

        Self {
            val,
            _cmp: PhantomData
        }
    }
}

pub trait RMQBlockTrait<C: RMQType> {
    fn query(&self, l: usize, r: usize) -> usize;
    unsafe fn query_unsafe(&self, l: usize, r: usize) -> usize;
}

macro_rules! impl_rmq_block {
    ($t:ty $(, $tr:ident )* ) => {
        impl<C: RMQType $( + $tr),*> RMQBlockTrait<C> for $t {
            fn query(&self, l: usize, r: usize) -> usize {
                const MASK: u16 = (1 << 15) - 1;
                let mask = MASK >> 15 - (r - l);
                let idx = if r >= 8 { 15 - r } else { r };
                let shift = if r >= 8 { 15 - r } else { 0 };
                let bits = (self.val[idx] >> shift & mask) as u64;
                let mut ret = 0u64;
                std::intrinsics::ctlz(bits) as usize - (64 - r)
            }
            
            unsafe fn query_unsafe(&self, l: usize, r: usize) -> usize {
                const MASK: u16 = (1 << 15) - 1;
                let mask = MASK >> 15 - (r - l);
                let idx = if r >= 8 { 15 - r } else { r };
                let shift = if r >= 8 { 15 - r } else { 0 };
                let bits = (self.val.get_unchecked(idx) >> shift & mask) as u64;
                let mut ret = 0u64;
                // std::intrinsics::ctlz(bits) as usize - (64 - r) // This was not replaced by lzcnt.
                // bits.leading_zeros() as usize - (64 - r) // This is the same as the above.
                // _lzcnt_u64(bits as u64) as usize - (64 - r) // This was not deployed inline.
                r - crate::common::get_msb_pos(bits) as usize
            }
        }
    };
}

impl_rmq_block!(RMQBlock<C>);
impl_rmq_block!(ArchivedRMQBlock<C>, Archive);

#[derive(Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
#[repr(align(64))]
pub struct Block<T: std::cmp::PartialOrd + std::default::Default> {
    min: RMQBlock<Min>,
    max: RMQBlock<Max>,
    val: [T; 16],
}

impl<T: std::cmp::PartialOrd + std::default::Default + std::marker::Copy> Block<T> {
    pub fn new(arr: &[T]) -> Self {
        let mut val = [T::default(); 16];
        val[..arr.len()].copy_from_slice(arr);
        Self {
            min: RMQBlock::new(&arr),
            max: RMQBlock::new(&arr),
            val,
        }
    }
}

macro_rules! impl_block {
    ($t:ty $(, $tr:ident ),* ) => {
        impl<T: std::cmp::PartialOrd + std::default::Default + std::marker::Copy $( + $tr<Archived = T>)*> $t {
            pub unsafe fn query_unsafe(&self, l: usize, r: usize) -> (T, T) {
                let min = self.min.query_unsafe(l, r);
                let max = self.max.query_unsafe(l, r);
                (*self.val.get_unchecked(min), *self.val.get_unchecked(max))
            }

            pub fn query(&self, l: usize, r: usize) -> (T, T) {
                let min = self.min.query(l, r);
                let max = self.max.query(l, r);
                (self.val[min], self.val[max])
            }

            pub fn get(&self, idx: usize) -> T {
                unsafe { *self.val.get_unchecked(idx) }
            }
        }   
    };
}

impl_block!(Block<T>);
impl_block!(ArchivedBlock<T>, Archive);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rmq_block_test() {
        let arr = [3, 5, 8, 4, 10, 1, 2, 9];
        let block = Block::new(&arr);
        for i in 0..8 {
            for j in i..8 {
                assert_eq!(unsafe{block.query_unsafe(i, j)}, (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
            }
        }
    }
}
