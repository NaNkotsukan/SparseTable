#![feature(core_intrinsics)]
mod compare;
mod block;
mod sparsetable;
mod common;
mod rmq;

pub use crate::rmq::{RMQ, ArchivedRMQ};
pub use crate::sparsetable::{SparseTable, ArchivedSparseTable, SparseTableEntryTrait};
pub use crate::compare::{CompareTrait, MinMaxTrait};