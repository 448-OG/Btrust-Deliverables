### DESIGN DOCUMENT

##### PROJECT
rust-bitcoin - https://github.com/rust-bitcoin/rust-bitcoin


##### Issue
https://github.com/rust-bitcoin/rust-bitcoin/issues/2505

This issue requires formatting of transaction IDs in pretty printing not to contain an `0x` prefix just like formatting only using std::fmt::Display.

This issue is dependent on the integration of `hex-conservative` crate version `0.2.0` which introduces breaking changes to parts of the `hash` crate of Rust bitcoin.

Solving this issue will require modifying crates in rust-bitcoin in order to make them compatible with the breaking changes introduced by `hex-conservative` crate version `0.2.0`

I feel confident to work on this crate because I possess a solid understanding on Rust language and have understood significant parts of the Bitcoin transaction structures and SHA256 algorithm.
