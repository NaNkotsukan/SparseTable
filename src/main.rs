#![feature(core_intrinsics)]
mod block;
mod sparsetable;
mod common;
mod rmq;
use crate::rmq::RMQ;
use rand::Rng;
use rkyv::ser::{Serializer, serializers::AllocSerializer};
use rkyv::{Archive, Deserialize, Serialize, Archived};
mod compare;

use compare::{CompareTrait, MinMaxTrait};

#[derive(Archive, Serialize)]
#[archive_attr(derive(Clone, Copy))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Hoge {
    a: std::num::NonZeroU16,
}

impl Default for Hoge {
    fn default() -> Self {
        Self {
            a: unsafe { std::mem::transmute(0u16) }
        }
    }
}

type T = Hoge;

fn main() {
    impl CompareTrait for T {
        fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
            a.a.cmp(&b.a)
        }
    }
    impl MinMaxTrait for T {}
    impl CompareTrait for Archived<T> {
        fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
            a.a.cmp(&b.a)
        }
    }
    impl MinMaxTrait for Archived<T> {}

    let num = 3000;
    let mut rng = rand::thread_rng();
    let arr = (&mut rng).sample_iter(rand::distributions::Uniform::new(1, u16::MAX))
        .take(num)
        .map(|x| Hoge{a: std::num::NonZeroU16::new(x).unwrap()})
        .collect::<Vec<_>>();
    let rmq: RMQ<Hoge> = crate::rmq::RMQ::<T>::new(&arr);

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&rmq).unwrap();
    let bytes = serializer.into_serializer().into_inner();
    
    let archived: &rmq::ArchivedRMQ<Hoge> = unsafe { rkyv::archived_root::<crate::rmq::RMQ::<T>>(&bytes[..]) };
        
    for i in 0..num {
        println!("{} / {}", i, num);
        for j in i..num {
            assert_eq!(unsafe{rmq.query_unsafe(i, j)}, (*arr[i..j+1].iter().min().unwrap(), *arr[i..j+1].iter().max().unwrap()));
            assert_eq!(unsafe{rmq.query_unsafe(i, j)}, unsafe{archived.query_unsafe(i, j)});
        }
    }
}