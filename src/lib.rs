#![feature(test)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(all(test, feature = "std"))]
extern crate test;

mod deserialize;
pub mod nbt;
pub mod protocol;
mod serialize;
pub mod status;
pub mod types;
pub mod utils;
pub mod uuid;
pub mod v1_15_2;

mod chat;

pub use deserialize::*;
pub use serialize::*;

#[cfg(all(test, feature = "std"))]
mod test_macros;