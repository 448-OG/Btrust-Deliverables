### DESIGN DOCUMENT

##### PROJECT
rust-bitcoin - https://github.com/rust-bitcoin/rust-bitcoin


##### Issue
https://github.com/rust-bitcoin/rust-bitcoin/issues/2505

This issue requires formatting of transaction IDs in pretty printing not to contain an `0x` prefix just like formatting only using std::fmt::Display.

This issue is dependent on the integration of `hex-conservative` crate version `0.2.0` which introduces breaking changes to parts of the `hash` crate of Rust bitcoin.

Solving this issue will require modifying crates in rust-bitcoin in order to make them compatible with the breaking changes introduced by `hex-conservative` crate version `0.2.0`

I feel confident to work on this crate because I possess a solid understanding on Rust language and have understood significant parts of the Bitcoin transaction structures and SHA256 algorithm.


#### Workflow
1. Fork repository
2. Create new branch
3. Change the pretty-printing code in the `hex-conservative` version `0.2.0`
4. Commit this changes
5. Check if this tied to macros that were introduced for easier code handling
6. Create fixes for breaking changes in each dependency crate for rust-bitcoin
7. Make atomic commits for each fix
8. Push data to current branch on Github
9. Check if Github actions pass
10. Make fixes if actions don't meet requirements
11. Create a draft PR on rust-bitcoin repository
12. Engage authors on current state of fixes
13. Mark PR as ready for review if all requirements are met.