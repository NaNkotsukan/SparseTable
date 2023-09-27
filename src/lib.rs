#![feature(core_intrinsics)]
mod block;
mod sparsetable;
mod common;
mod rmq;

pub use crate::rmq::{RMQ, ArchivedRMQ};
pub use crate::sparsetable::{SparseTable, ArchivedSparseTable};