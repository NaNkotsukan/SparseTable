#![feature(core_intrinsics)]
mod block;
mod sparsetable;
mod common;
mod rmq;
use crate::rmq::RMQ;
use rand::Rng;
use rkyv::ser::{Serializer, serializers::AllocSerializer};

type T = u16;

fn main() {
    let num = 3000;
    let mut rng = rand::thread_rng();
    let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(0, u16::MAX)).take(num).collect::<Vec<_>>();
    let rmq = crate::rmq::RMQ::<u16>::new(&arr);

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&rmq).unwrap();
    let bytes = serializer.into_serializer().into_inner();
    
    let archived = unsafe { rkyv::archived_root::<crate::rmq::RMQ::<u16>>(&bytes[..]) };
        
    for i in 0..num {
        println!("{} / {}", i, num);
        for j in i..num {
            assert_eq!(unsafe{rmq.query_unsafe(i, j)}, (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
            assert_eq!(unsafe{rmq.query_unsafe(i, j)}, unsafe{archived.query_unsafe(i, j)});
        }
    }
}