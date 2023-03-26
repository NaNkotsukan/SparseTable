#![feature(core_intrinsics)]
mod block;
mod sparsetable;
mod common;
mod rmq;

use crate::rmq::{RMQ, ArchivedRMQ};
use crate::sparsetable::{SparseTable, ArchivedSparseTable};