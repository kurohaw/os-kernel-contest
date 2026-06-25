//! sd card driver for raspi4,
//! if you want to use it, change \[dev-dependencies\] in modules/axfs/Cargo.toml
//! from axdriver = { path = "../axdriver", features = \["block", "ramdisk"\] } to axdriver = { path = "../axdriver", features = \["block", "mmc"\] }
//! ans change \[features\] in  modules/axruntime/Cargo.toml
//! from "fs = \["alloc", "paging", "axdriver/virtio-blk", "dep:axfs"\]" to "fs = \["alloc", "paging", "axdriver/mmc", "dep:axfs"\]

#![no_std]

#[macro_use]
extern crate log;
pub mod Bcm2835SDhci;
pub mod emmc;
pub mod qa7_control;
pub mod timer;

pub enum SDHCIError {
    /// An entity already exists.
    AlreadyExists,
    /// Try again, for non-blocking APIs.
    Again,
    /// Bad internal state.
    BadState,
    /// Invalid parameter/argument.
    InvalidParam,
    /// Input/output error.
    Io,
    /// Not enough space/cannot allocate memory (DMA).
    NoMemory,
    /// Device or resource is busy.
    ResourceBusy,
    /// This operation is unsupported or unimplemented.
    Unsupported,
}

/// A specialized `Result` type for device operations.
pub type SDHCIResult<T = ()> = Result<T, SDHCIError>;
