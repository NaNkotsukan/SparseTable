#[cfg(target_arch = "x86_64")]
use smallvec::{SmallVec, smallvec};
use std::{arch::x86_64::*, marker::PhantomData, default};

pub struct Min;
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
        
        println!("{:?}", g);
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
        println!("{:?}", l);

        Self {
            val,
            _cmp: PhantomData
        }
    }

    #[inline(always)]
    pub fn query(&self, l: usize, r: usize) -> usize {
        const MASK: u16 = (1 << 15) - 1;
        let mask = MASK >> 15 - (r - l);
        let idx = if r >= 8 { 15 - r } else { r };
        let shift = if r >= 8 { 15 - r } else { 0 };
        unsafe {
            let bits = (self.val.get_unchecked(idx) >> shift & mask) as u64;
            let mut ret = 0u64;
            // _lzcnt_u32(bits as u32) as usize - (32 - r)
            // _lzcnt_u64(bits as u64) as usize - (64 - r)
            // std::intrinsics::ctlz(bits) as usize - (16 - r)
            std::arch::asm!("lzcnt {ret:r}, {bits:r}",
                             bits = in(reg) bits,
                             ret = out(reg) ret);
            ret as usize - (64 - r)
        }
    }
}

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

    #[inline(never)]
    pub fn query(&self, l: usize, r: usize) -> (T, T) {
        let min = self.min.query(l, r);
        let max = self.max.query(l, r);
        unsafe { (*self.val.get_unchecked(min), *self.val.get_unchecked(max)) }
    }

    pub fn get(&self, idx: usize) -> T {
        unsafe { *self.val.get_unchecked(idx) }
    }
}
