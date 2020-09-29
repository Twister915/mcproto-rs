#![feature(impl_trait_in_bindings)]
#![feature(const_fn)]
#![feature(test)]

#[cfg(test)]
extern crate test;

mod serialize;
mod deserialize;
pub mod utils;
#[macro_export]
pub mod protocol;
pub mod uuid;
pub mod nbt;
pub mod types;
pub mod v1_15_2;
pub mod status;

pub use serialize::*;
pub use deserialize::*;

#[cfg(test)]
#[macro_export]
mod test_macros;