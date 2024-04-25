use pbkdf2::pbkdf2_hmac;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

fn main() {
    // Instantiate a mnemonic generator with the location of the wordlist
    let mut generator = Bip39Generator::new("english.txt");
    // Generate the mnemonic
    let mnemonic = generator.mnemonic::<16>().unwrap();

    println!("Your mnemonic is: {}", &mnemonic);

    // Recover a wallet by running the mnemonic through seed generator without a passphrase
    let insecure_seed = Bip39Generator::insecure_seed(&mnemonic);

    let passphrase = "BitCoin_iZ_Awesome";
    // Recover a wallet by running the mnemonic through seed generator without a passphrase
    let secure_seed = Bip39Generator::secure_seed(&mnemonic, passphrase);

    // test that the seed generation succeeds
    assert!(&secure_seed.is_ok(),);
    assert!(&insecure_seed.is_ok(),);
}

/// Number of iterations to be run by the PBKDF2 for key derivation
pub const ITERATION_COUNT: u32 = 2048;
/// The word used as a prefix for the salt for our key derivation function
pub const SALT_PREFIX: &str = "mnemonic";

#[derive(Debug, Default)]
pub struct Bip39Generator {
    // This holds all our indexes that we will use to fetch
    // our word from the word list
    // with each index corresponding to an index
    // from our wordlist contained in a Vec<word>
    mnemonic_index: Vec<u16>,
    // This field holds the random bytes with our checksum
    // bytes appended to the end
    appended: Vec<u8>,
    // This contains a path to our wordlist file
    path: PathBuf,
}

impl Bip39Generator {
    // This method takes an argument `path_to_wordlist` which
    // is a path to the wordlist we downloaded
    // where the path is anything that implements the trait
    // AsRef<Path> meaning we pass any data type as convert it
    // to a path using the `.as_ref()` method as long as that
    // data type implements the `AsRef<Path>` trait.
    pub fn new(path_to_wordlist: impl AsRef<Path>) -> Self {
        Self {
            // Convert `path_to_wordlist` argument to a path
            // using `.as_ref()` method and convert it
            // to a `std::path::PathBuf` using the `.to_path_buf()`
            path: path_to_wordlist.as_ref().to_path_buf(),
            // All other fields can hold default values
            // and we can call this method since
            // we derived `Default` values using `#[derive(Default)]`
            // on our struct
            ..Default::default()
        }
    }

    /// Generates a seed without a passphrase
    pub fn insecure_seed(mnemonic: &str) -> io::Result<Vec<u8>> {
        Self::seed(mnemonic, None)
    }

    /// Generates a seed with a passphrase
    pub fn secure_seed(mnemonic: &str, passphrase: &str) -> io::Result<Vec<u8>> {
        Self::seed(mnemonic, Some(passphrase))
    }

    // We pass our mnemonic and an optional passphrase
    fn seed(mnemonic: &str, passphrase: Option<&str>) -> io::Result<Vec<u8>> {
        // We check if there is a passphrase provided.
        // if there is one we prefix our salt with the passphrase
        let salt = if let Some(passphrase_required) = passphrase {
            String::new() + SALT_PREFIX + passphrase_required
        } else {
            String::from(SALT_PREFIX)
        };

        // We want to generate a 512bit seed
        // so we create a buffer to hold this.
        let mut wallet_seed = [0u8; 64]; // 512 bits == 64 bytes

        // We generate a key and push all the bytes to the `wallet_seed` buffer
        pbkdf2_hmac::<Sha512>(
            mnemonic.as_bytes(),
            salt.as_bytes(),
            ITERATION_COUNT,
            &mut wallet_seed,
        );

        // We return our seed
        Ok(wallet_seed.to_vec())
    }

    pub fn mnemonic<const N: usize>(&mut self) -> io::Result<String> {
        // This generates the number of random bits we need
        let entropy = Entropy::<{ N }>::generate();

        // Next, let's generate our checksum
        self.generate_checksum::<N>(entropy.0);

        // Next we compute the decimal numbers we will use
        // to get our wordlist
        self.compute();

        // Load the wordlist into memory
        let wordlist = self.load_wordlist()?;

        // Iterate through the decimal numbers
        // and for each decimal number get the word
        // in it's index in the wordlist (wordlist[index from decimal number]
        let mnemonic = self
            .mnemonic_index
            .iter()
            // Enumerate to get the current count in our interation
            .enumerate()
            .map(|(index, line_number)| {
                // Convert our decimal index (line_numer) to
                // a usize since Rust is very strict in that
                // you can only index an array using a usize
                // so we dereference and cast using `as usize`
                let word = (&wordlist[*line_number as usize]).clone() + " "; // Add a space in each word
                                                                             // Since indexes start at zero we add `1`
                                                                             // to make them human readable (humans mostly count from 1)
                let index = index + 1;

                // Check if we have our index is less than
                // 10 so we add a padding to make printing
                // to console neat
                let indexed = if index < 10 {
                    String::new() + " " + index.to_string().as_str()
                } else {
                    index.to_string()
                };

                // Print our index and each word. This
                // will show the user the words in each
                // line but with a number. eg
                //  9. foo
                // 10. bar
                println!("{}. {}", indexed, &word);

                // Return the word
                word
            })
            .collect::<String>(); // Combine all strings into one

        // Trim the last space in the and return the mnemonic
        Ok(mnemonic.trim().to_owned())
    }

    fn load_wordlist(&mut self) -> io::Result<Vec<String>> {
        // open the file using the path we passed
        // when instantiating our struct
        // using `Bip39Generator::new()`
        let file = File::open(&self.path)?;
        // Create a buffer so that we can efficiently readd
        // our file
        let reader: io::BufReader<File> = io::BufReader::new(file);

        // Create a Vector to hold our wordlist
        let mut wordlist = Vec::<String>::new();

        // Read each line
        for line in reader.lines() {
            // Push each word to our `wordlist` vector
            // handling any I/O errors using `?`
            wordlist.push(line?);
        }

        // Return our vector of word list
        Ok(wordlist)
    }

    // Here we pass our generated random bytes as `entropy` argument
    fn generate_checksum<const N: usize>(&mut self, entropy: [u8; N]) -> &mut Self {
        // BIP39 spec requires a seed to be generated
        // using a SHA256 Psuedo Random Function (PRF)
        // so we instantiate a SHA256 hashing function.
        let mut hasher = Sha256::new();

        // We now pass our random bytes into our SHA256 PRF
        hasher.update(entropy.as_slice());

        // We now get our finalized value. Using
        // SHA256 always ensures that despite being
        // able to use variable length of random bytes
        // we always get back a 256 bit (32 byte) value.
        let entropy_hash = hasher.finalize();

        // Since we get a 32 byte value we multiply by
        // `8` to get number of bits since 1 byte == 8 bits
        let bits_of_entropy = entropy.len() * 8;
        // We get our `n` bits for our checksum from the
        // length of the random bits (entropy)
        // where `n` is calculated as the
        // `length of our random bits / 32`
        let bits_of_checksum = bits_of_entropy / 32;
        // We then use bit shifting to get
        // bits of checksum from our
        // 256 bit hash in variable `entropy_hash`
        let significant = entropy_hash[0] >> bits_of_checksum;

        let mut appended = entropy.to_vec();
        // We then append our checksum to our random
        appended.push(significant);

        // We now assign our appended bytes to the `appended`
        // field of our `Bip39Generator` struct which is `Self`
        self.appended = appended;

        self
    }

    // We pass a mutable to self since we want to
    // add the result of this computation to `Self`
    fn compute(&mut self) -> &mut Self {
        // This vector will hold the binary
        // representation of each byte in the `appended` vector.
        let mut bits = vec![];

        // This line starts a loop that iterates over each byte in the `self.appended` vector.
        for &byte in self.appended.iter() {
            // This line starts a nested loop that
            // counts backwards from 7 to 0.
            // The variable `i` represents the position of
            // the bit we're interested in within
            // the current byte.
            for i in (0..8).rev() {
                /*
                This line does three things:
                 - `byte >> i`: This is a right bitwise shift operation.
                                It moves the bits in `byte` `i` places to the right.
                                The bit at position `i` is now at position 0.
                 - `(byte >> i) & 1u8`: This is a bitwise AND operation with `1u8` (which is `1` in binary).
                                        This operation effectively masks all the bits in `byte` except for the one at position 0.
                 - `bits.push((byte >> i) & 1u8 == 1);`: This pushes `true` if the bit at position 0 is `1`
                                                          and `false` otherwise into the `bits` vector.
                */
                bits.push((byte >> i) & 1u8 == 1);
            }
        }

        // This line starts a loop that iterates over
        // the `bits` vector in chunks of 11 bits.
        for chunk in bits.chunks(11) {
            // This line checks if the current chunk has
            // exactly 11 bits. If it does, the code inside
            // the if statement is executed.
            if chunk.len() == 11 {
                // This line initializes a mutable
                // variable named `value` and sets it to 0.
                // This variable will hold the decimal
                // representation of the current 11-bit chunk.
                let mut value: u16 = 0;

                // This line starts a nested loop that iterates
                // over each bit in the current chunk.
                // The variable `i` is the index of the current
                //  bit, and `bit` is the value of the current bit.
                for (i, &bit) in chunk.iter().enumerate() {
                    // This line checks if the current bit
                    // is `1` (true). If it is, it shifts `1`
                    // to the left by `(10 - i)` places
                    // (this effectively gives `1` a value of `2^(10 - i)`)
                    // and then performs a bitwise OR operation with `value`.
                    // This has the effect of adding `2^(10 - i)` to `value`.
                    if bit {
                        value |= 1u16 << (10 - i);
                    }
                }
                // This line pushes the decimal
                // representation of the current 11-bit chunk
                // into the `self.mnemonic_index` vector.
                self.mnemonic_index.push(value);
            }
        }

        self
    }
}

// This struct takes a constant `N` as a generic
// enabling one to specify a variable length for the bytes generated
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entropy<const N: usize>([u8; N]);

impl<const N: usize> Entropy<N> {
    // This method generates the bytes
    pub fn generate() -> Self {
        // Instantiate our cryptographically secure random byte generation algorithm
        let mut rng = ChaCha20Rng::from_entropy();
        // Create a zero filled buffer to hold our bytes
        let mut buffer = [0u8; N];
        // Fill our buffer with random bytes
        rng.fill_bytes(&mut buffer);

        // Return our buffer
        Self(buffer)
    }
}
