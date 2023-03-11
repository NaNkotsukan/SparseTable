#![feature(asm, core, lang_items, no_std)]
#![feature(core_intrinsics)]
mod block;
mod sparsetable;
mod common;
mod rmq;


fn main() {
    let arr = [3, 5, 8, 4, 10, 1, 2, 9];
    let rmq = rmq::RMQ::new(&arr);
    for i in 0..8 {
        for j in i..8 {
            println!("{:?}, {} {}", unsafe{rmq.query_unsafe(i, j)}, arr[i..j+1].iter().min().unwrap(), arr[i..j+1].iter().max().unwrap());
            assert_eq!(unsafe{rmq.query_unsafe(i, j)}, (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
        }
    }
}