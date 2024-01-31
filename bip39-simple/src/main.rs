use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

fn main() {
    let mut outcome = Bip39Generator::new("english.txt");

    dbg!(&outcome.insecure_mnemonic::<16>());
}

#[derive(Debug, Default)]
pub struct Bip39Generator {
    mnemonic_index: Vec<u16>,
    appended: Vec<u8>,
    path: PathBuf,
}

impl Bip39Generator {
    pub fn new(path_to_wordlist: impl AsRef<Path>) -> Self {
        Self {
            path: path_to_wordlist.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn insecure_mnemonic<const N: usize>(&mut self) -> io::Result<Vec<String>> {
        self.mnemonic::<N>()
    }

    pub fn mnemonic<const N: usize>(&mut self) -> io::Result<Vec<String>> {
        let entropy = Entropy::<{ N }>::generate();

        self.generate_checksum::<N>(entropy.0);

        self.compute();

        let wordlist = self.load_wordlist()?;

        let mnemonic = self
            .mnemonic_index
            .iter()
            .map(|index| (&wordlist[*index as usize]).clone())
            .collect::<Vec<String>>();

        Ok(mnemonic)
    }

    fn load_wordlist(&mut self) -> io::Result<Vec<String>> {
        let file = File::open(&self.path)?;
        let reader: io::BufReader<File> = io::BufReader::new(file);

        let mut wordlist = Vec::<String>::new();

        for line in reader.lines() {
            wordlist.push(line?);
        }

        Ok(wordlist)
    }

    fn generate_checksum<const N: usize>(&mut self, entropy: [u8; N]) -> &mut Self {
        let mut hasher = Sha256::new();
        hasher.update(entropy.as_slice());

        let entropy_hash = hasher.finalize();

        let bits_of_entropy = entropy.len() * 8;
        let bits_of_checksum = bits_of_entropy / 32;
        let significant = entropy_hash[0] >> bits_of_checksum;

        let mut appended = entropy.to_vec();
        appended.push(significant);

        self.appended = appended;

        self
    }

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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entropy<const N: usize>([u8; N]);

impl<const N: usize> Entropy<N> {
    pub fn generate() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut buffer = [0u8; N];
        rng.fill_bytes(&mut buffer);

        Self(buffer)
    }
}
