#![no_std]
#![feature(let_chains)]
#![feature(unsigned_is_multiple_of)]
#![allow(missing_docs)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate axlog;
extern crate alloc;

pub mod fs;
pub mod iomux;
pub mod ipc;
pub mod mm;
pub mod net;
pub mod sys;
pub mod task;

pub use {fs::*, iomux::*, ipc::*, mm::*, net::*, sys::*, task::*};
