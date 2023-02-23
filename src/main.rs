#![feature(asm, core, lang_items, no_std)]
#![feature(core_intrinsics)]
mod block;
use crate::abs::hoge;


fn main() {
    let arr = [3, 5, 8, 4, 10, 1, 2, 9];
    let block = block::Block::new(&arr);
    for i in 0..8 {
        for j in i..8 {
            println!("{:?}, {} {}", block.query(i, j), arr[i..j+1].iter().min().unwrap(), arr[i..j+1].iter().max().unwrap());
            assert_eq!(block.query(i, j), (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
        }
    }
}