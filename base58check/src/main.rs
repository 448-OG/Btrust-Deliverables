use bitcoin::base58;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

fn main() {
    let private_key = Entropy::<4>::generate().0;
    let mut bytes = Base58Check::new()
        .add_prefix(&[0u8, 0, 0, 0])
        .add_payload(&private_key)
        .calc_checksum()
        .build();
    dbg!(base58::encode(&bytes).to_string());
    let custom_conversion = to_base58(&mut bytes);
    dbg!(&custom_conversion);

    println!("{:?}", base58::decode(&base58::encode(&bytes)));

    let to_custom_vec = from_base58(&custom_conversion);
    assert_eq!(
        to_custom_vec,
        base58::decode(&base58::encode(&bytes)).unwrap()
    );
}

#[derive(Debug, Default)]
pub struct Base58Check {
    prefix: Vec<u8>,
    payload: Vec<u8>,
    checksum: Vec<u8>,
}

impl Base58Check {
    fn new() -> Self {
        Self::default()
    }

    fn add_prefix(mut self, prefix: &[u8]) -> Self {
        self.prefix.extend_from_slice(prefix);

        self
    }

    fn add_payload(mut self, payload: &[u8]) -> Self {
        self.payload.extend_from_slice(payload);

        self
    }

    fn calc_checksum(mut self) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&self.prefix);
        hasher.update(&self.payload);

        let first_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(first_hash.as_slice());
        let double_hash = hasher.finalize();

        self.checksum.extend_from_slice(&double_hash[0..4]);

        self
    }

    fn build(mut self) -> Vec<u8> {
        let mut outcome = Vec::<u8>::new();
        outcome.extend(self.prefix.drain(..));
        outcome.extend(self.payload.drain(..));
        outcome.extend(self.checksum.drain(..));

        outcome
    }
}

fn to_base58(base58_bytes: &mut [u8]) -> String {
    let mut base58_char = VecDeque::<char>::new();
    let mut outcome = String::new();

    for _ in base58_bytes.iter().take_while(|&&x| x == 0) {
        outcome.push('1');
    }

    let mut decimal = 0usize;

    for value in base58_bytes {
        decimal = (decimal << 8) | *value as usize;
    }

    let index_alphabet = ALPHABET.chars().collect::<Vec<char>>();

    while decimal != 0 {
        let (quotient, remainder) = (decimal / 58, decimal % 58);

        base58_char.push_front(index_alphabet[remainder as usize]);
        decimal = quotient;
    }

    outcome += base58_char.iter().collect::<String>().as_str();

    outcome
}

fn from_base58(base58_str: &str) -> Vec<u8> {
    let mut decimal = 0usize;
    let index_alphabet: Vec<_> = ALPHABET.chars().collect();

    let mut leading_zeros_total = 0usize;
    let mut outcome = Vec::<u8>::new();
    let mut split_chars = base58_str.chars().collect::<Vec<char>>();

    for _ in split_chars.iter().take_while(|&x| x == &'1') {
        leading_zeros_total += 1;
        outcome.push(0);
    }

    split_chars.drain(0..leading_zeros_total);

    for current_char in split_chars {
        let value = index_alphabet
            .iter()
            .position(|&in_alphabet| in_alphabet == current_char)
            .unwrap();
        decimal = (decimal * 58) + value;
    }

    let mut bytes = Vec::<u8>::new();

    while decimal > 0 {
        let (quotient, remainder) = (decimal / 256, decimal % 256);
        bytes.push(remainder as u8);
        decimal = quotient;
    }

    bytes.reverse();

    outcome.extend_from_slice(&bytes);

    outcome
}

const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

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
