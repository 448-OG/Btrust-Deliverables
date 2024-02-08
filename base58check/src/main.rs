use std::str::FromStr;

use hex_literal::hex;
use sha2::{Digest, Sha256};
fn main() {
    let hashed = Base58Ops::sha256(b"foobar");
    let hashed_hex = hex::encode(&hashed);
    dbg!(&hashed_hex);

    let mut foo = b"Cat".to_vec();
    let mut foo = b"28a".to_vec();
    let mut foo = hashed.to_vec();
    foo.reverse();

    let mut add_outcome = 0u64;

    for (index, value) in foo.iter().enumerate() {
        let mul_index = index * 8;
        let index_pow = 2u8.pow(mul_index as u32) as u64;
        let outcome = index_pow * *value as u64;

        add_outcome += outcome;
    }

    dbg!(&add_outcome);
    let mut values = Vec::<u64>::new();

    while add_outcome != 0 {
        let (quotient, remainder) = (add_outcome / 58, add_outcome % 58);

        add_outcome = quotient;

        values.push(remainder);
    }

    values.reverse();

    let base58_string = values
        .into_iter()
        .map(|value| BASE58_ALPHABET[value as usize])
        .collect::<String>();

    dbg!(&base58_string);
}
const BASE58_ALPHABET: [char; 58] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z',
];

pub struct Base58Ops {
    data: Vec<u8>,
    version: Base58VersionByte,
    checksum: [u8; 4],
}

impl Base58Ops {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            version: Base58VersionByte::default(),
            checksum: [0u8; 4],
        }
    }

    fn checksum(&mut self) -> &mut Self {
        let mut concat = Vec::<u8>::new();
        concat.extend_from_slice(self.version.as_bytes());

        let mut first_hash = Base58Ops::sha256(&concat);

        let mut second = Base58Ops::sha256(&first_hash);

        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&second.as_slice()[0..=3]);

        self.checksum = checksum;

        self
    }

    pub fn sha256(bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);

        hasher.finalize().into()
    }

    fn to_hash_160(&self) {}
}

#[derive(Debug, Default)]
pub enum Base58VersionByte {
    #[default]
    BitcoinP2PKH,
    BitcoinP2SH,
    TestnetP2PKH,
    TestnetP2SH,
    WifPrivateKey,
    Bip32ExtendedPublicKey,
}

impl Base58VersionByte {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Self::BitcoinP2PKH => [0u8].as_slice(),
            Self::BitcoinP2SH => [5u8].as_slice(),
            Self::TestnetP2PKH => [111u8].as_slice(),
            Self::TestnetP2SH => [196u8].as_slice(),
            Self::WifPrivateKey => [128u8].as_slice(),
            Self::Bip32ExtendedPublicKey => [4u8, 136, 178, 30].as_slice(),
        }
    }

    pub const fn to_hex(&self) -> &str {
        match self {
            Self::BitcoinP2PKH => "00",
            Self::BitcoinP2SH => "05",
            Self::TestnetP2PKH => "6F",
            Self::TestnetP2SH => "C4",
            Self::WifPrivateKey => "80",
            Self::Bip32ExtendedPublicKey => "0488B21E",
        }
    }

    pub const fn from_bytes(value: &[u8]) -> Self {
        match value {
            &[0u8] => Self::BitcoinP2PKH,
            &[5u8] => Self::BitcoinP2SH,
            &[111u8] => Self::TestnetP2PKH,
            &[196u8] => Self::TestnetP2SH,
            &[128u8] => Self::WifPrivateKey,
            &[4u8, 136, 178, 30] => Self::Bip32ExtendedPublicKey,
            _ => panic!("This can be returned as an error"),
        }
    }
}

#[cfg(test)]
mod prefix_sanity_checks {
    use crate::Base58VersionByte;
    use hex_literal::hex;
    #[test]
    fn run_test() {
        assert_eq!(Base58VersionByte::BitcoinP2PKH.as_bytes(), hex!("00"));
        assert_eq!(Base58VersionByte::BitcoinP2SH.as_bytes(), hex!("05"));
        assert_eq!(Base58VersionByte::TestnetP2PKH.as_bytes(), hex!("6F"));
        assert_eq!(Base58VersionByte::TestnetP2SH.as_bytes(), hex!("C4"));
        assert_eq!(Base58VersionByte::WifPrivateKey.as_bytes(), hex!("80"));
        assert_eq!(
            Base58VersionByte::Bip32ExtendedPublicKey.as_bytes(),
            hex!("0488B21E")
        );
    }
}

pub struct Mapping;

impl Mapping {
    pub fn to_character(value: u8) -> char {
        if value > 57 {
            panic!("DECIMAL VALUES CANNOT BE GREATER THAN 57")
        }

        BASE58_ALPHABET[value as usize]
    }
}
