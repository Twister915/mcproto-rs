#![cfg_attr(feature = "bench", feature(test))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(all(test, feature = "std", feature = "bench"))]
extern crate test;

mod deserialize;
mod serialize;
pub mod nbt;
pub mod protocol;
pub mod status;
pub mod types;
pub mod utils;
pub mod uuid;
mod chat;
pub mod byte_order;

#[cfg(feature = "v1_15_2")]
pub mod v1_15_2;
#[cfg(feature = "v1_16_3")]
pub mod v1_16_3;

pub use deserialize::*;
pub use serialize::*;

#[cfg(all(test, feature = "std"))]
mod test_macros;
