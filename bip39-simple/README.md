### Wallet Backup and Recovery

Storing private keys in memory can be challenging due to their large size of 64 hex characters. Several backup and recovery mechanisms have been proposed such as `BIP39`, `Electrum v2`, `Aezeed`, `Muun` and `SLIP39`. In this article we will look at `Bitcoin Improvement Proposal 39 (BIP39)`.

BIP39 is a method of creating human readable mnemonic sequence of characters for wallet recovery. The mnemonic is generated from a wordlist with the following characteristics as specified in the proposal:

1. the wordlist is created in such a way that it's enough to type the first four letters to unambiguously identify the word

2. word pairs like "build" and "built", "woman and women", or "quick and quickly" not only make remembering the sentence difficult but are also more error prone and more difficult to guess

3. the wordlist is sorted which allows for efficient lookup of the code words (e.g using binary search instead of linear search) allowing trie (a prefix tree) to be used e.g for better compression.

The specification also requires that native characters used to be encoded in UTF-8 using Normalization Form Compatibility Decomposition (NFKD)

A user may choose to protect their mnemonic with a passphrase and the implementation requires an empty string "" to be used if a passphrase is not present.

To create a binary seed from the mnemonic with a passphrase:

1. create a password from a UTF-8 NFKD mnemonic sentence

2. run the password through a PBKDF2 function with the string "mnemonic" + a UTF-8 NFKD salt.

3. Set the iteration count of PBKDF2 to 2048 and HMAC to HMAC-SHA512 as the pseudorandom function which derives a 512 bit key as the seed.

The seed can be used to generate deterministic wallets using BIP39. Plausible deniability is guaranteed since all mnemonics produce a valid seed but only the correct seed can reconstruct the correct wallet. 

##### Lets write some code to show how to generate a mnemonic and then later we will run the mnemonic through a PBKDF2 function.

This tutorial assumes you have installed Rust Programming Language toolchain which comes bundled with cargo build and dependency management tool. 

1. Let's create a cargo project called `bip39-simple`
```sh
$ cargo new bip39-simple
```

2. Switch to that project directory
```sh
$ cd bip39-simple
```

3. Add dependencies to `Cargo.toml` file
```toml
[dependencies]
rand_chacha = "*"
rand_core = { version = "*", features = ["getrandom"] }
sha2 = "*"
```

We add the `getrandom` feature to `rand_core` in order to get random bytes using the operating system cryptographically secure random number generator. `sha2` crate will be used to create a checksum from the SHA256 hash of the random bytes generated.

First, we will create a struct `Entropy` to generate some random bytes
```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entropy<const N: usize>([u8; N]);
```

Here, we are using a `const` generic`N` to create a variable length array of bytes `(u8)` . The benefits of creating a generic constant is that we can create a variable length array instead of creating a method to generate arrays for each acceptable bit length eg `128`, `256` or `512` bits

Next we import our Rust crates for generating random numbers and then create a `generate()` method on our `Entropy` struct to generate random byte.

```rust
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

impl<const N: usize> Entropy<N> {
    pub fn generate() -> Self {
        // Use the `ChaCha` based algorithm for random byte generation
        let mut rng = ChaCha20Rng::from_entropy();
        // A mutable buffer to hold our random bytes.
        // Note that the array length is defined by the generic `N`
        let mut buffer = [0u8; N];
        // Fill the buffer with our randomly generated bytes
        rng.fill_bytes(&mut buffer);

        // Return self with the random bytes as it's field
        Self(buffer)
    }
}
```

Next we create a mnemonic generator struct `Bip39Generator`

```rust
#[derive(Debug, Default)]
pub struct Bip39Generator {
    // This will hold the decimal indexes for a bits
    // whereby each index will be the line number of the mnemonic word in the wordlist
    mnemonic_index: Vec<u16>,
    // This is the random bytes appended with the checksum byte
    appended: Vec<u8>,
    // This is the path to the wordlist to be chosen
    path: PathBuf,
}
```

Next we will implement 7 methods:

1. `new()` method to instantiate our `Bip39Generator` generator struct
2. `insecure_mnemonic()` which will generate our seed without using the PBKDF2 key derivation algorithm
3. `secure_mnemonic()`  which will generate our seed using the PBKDF2 key derivation algorithm
4. `load_wordlist()` which will load our wordlist into memory
5. `mnemonic()` which generate our mnemonic words
6. `generate_checksum()` which generates our checksum from the SHA256 hash of our random bytes
7. `compute()` which computes the decimal line number for selecting a mnemonic from our wordlist

Let's import some dependencies

```rust
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};
```

Next we create each of our methods

```rust
// Instatiate our struct by ensuring that there is a path to our wordlist
// using `impl AsRef<Path>` ensures any data type that can be converted to a `std::path::Path` can be used.
pub fn new(path_to_wordlist: impl AsRef<Path>) -> Self {
        Self {
            path: path_to_wordlist.as_ref().to_path_buf(),
            // Create default implementations of the rest of the struct fields
            ..Default::default()
        }
    }
```

Next, the `insecure_mnemonic` method
```rust
// This takes a generic constant `N` which defines the number of random bytes to generate
pub fn insecure_mnemonic<const N: usize>(&mut self) -> io::Result<Vec<String>> {
    // This calls the `mnemonic()` method since with no passphrase passed to the KDF
    self.mnemonic::<N>()
}
```

```rust
// This method selects all the mnemonics from the wordlist
pub fn mnemonic<const N: usize>(&mut self) -> io::Result<Vec<String>> {
    // Generate array of random bytes with length `N` 
    let entropy = Entropy::<{ N }>::generate();

    // Generate a checksum from the entropy
    self.generate_checksum::<N>(entropy.0);

    // Get the decimal values of the 11bit groups from combination
    // of the checksum and the random bytes
    self.compute();

    // Load our wordlist into memory
    let wordlist = self.load_wordlist()?;

    // select all words from our wordlist based on the index from our compute function
    let mnemonic = self
        .mnemonic_index
        .iter()
        .map(|index| (&wordlist[*index as usize]).clone())
        .collect::<Vec<String>>();

    // Return our mnemonic values
    Ok(mnemonic)
}
```

```rust
// This method loads wordlist from the path specified when instantiating our method
fn load_wordlist(&mut self) -> io::Result<Vec<String>> {
    // Open file
    let file = File::open(&self.path)?;
    // Create a buffer to read our file.
    // Using a buffer can be efficient 
    let reader: io::BufReader<File> = io::BufReader::new(file);

    // Where to store our words from the wordlist
    let mut wordlist = Vec::<String>::new();

    // Read each line in the file
    for line in reader.lines() {
        // Add each line to our `wordlist` vector
        wordlist.push(line?);
    }

    Ok(wordlist)
}
```

```rust
// This generates the checksum and appends the bytes to the random bytes generated
fn generate_checksum<const N: usize>(&mut self, entropy: [u8; N]) -> &mut Self {
    // Instatiate a new SHA256 hasher
    let mut hasher = Sha256::new();
    // Update our hasher with the random bytes
    hasher.update(entropy.as_slice());

    let entropy_hash = hasher.finalize();

    // 
    let bits_of_entropy = entropy.len() * 8;
    let bits_of_checksum = bits_of_entropy / 32;
    let significant = entropy_hash[0] >> bits_of_checksum;

    let mut appended = entropy.to_vec();
    appended.push(significant);

    self.appended = appended;

    self
}
```

```rust
// Compute our bytes from the concatenation of random bytes generated and the checksum bytes
// in groups of 11 bits each
fn compute(&mut self) -> &mut Self {
    let mut bits = vec![];
    for &byte in self.appended.iter() {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1u8 == 1);
        }
    }

    for chunk in bits.chunks(11) {
        if chunk.len() == 11 {
            let mut value: u16 = 0;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    value |= 1u16 << (10 - i);
                }
            }
            self.mnemonic_index.push(value);
        }
    }

    self
}
```
