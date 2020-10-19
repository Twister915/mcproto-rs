# mcproto-rs

This is an implementation of serialization and deserialization of the minecraft protocol.

This crate can be used to implement any version of the minecraft protocol, and has an example implementation of version 
1.15.2 included as module `v1_15_2`.

To implement your own protocol, consult this example, and use the macros to define a protocol to your heart's content!

More documentation to come, just dumping the code since I finished it.

## `#![no_std]`

You can use this crate without the standard library (but requiring `alloc`) by setting `default-features = false` in 
your Cargo.toml. This will only disable the `UUID4::random()` function, which requires `OsRandom` to generate a random UUID.