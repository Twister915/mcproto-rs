#![feature(impl_trait_in_bindings)]
#![feature(const_fn)]
#![feature(test)]

#[cfg(test)]
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

pub use deserialize::*;
pub use serialize::*;

#[cfg(test)]
mod test_macros;
