use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};

fn main() {
    let entropy = Entropy::<32>::generate().0;
    let mut byte_slice = Base58VersionByte::BitcoinExtendedPublicKey
        .as_bytes()
        .to_vec();
    byte_slice.extend_from_slice(&entropy);

    let mut hasher = Sha256::new();
    hasher.update(byte_slice.as_slice());
    let finalize_hasher1 = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(finalize_hasher1.as_slice());
    let finalized = hasher.finalize().to_vec();

    let checksum = &finalized[0..=3];
    byte_slice.extend_from_slice(&checksum);

    dbg!(hex::encode(&byte_slice));

    let custom_base58 = to_base58(&byte_slice);
    let crate_base58 = bs58::encode(&byte_slice).into_string();
    dbg!(&custom_base58);
    dbg!(&crate_base58);
    dbg!(&custom_base58.len());
    dbg!(&crate_base58.len());

    assert_eq!(custom_base58.as_str(), crate_base58.as_str());

    let from_base = from_base58(&custom_base58);
    assert_eq!(&byte_slice, from_base.as_slice());
    let from_crate_base58 = bs58::decode(&custom_base58).into_vec().unwrap();
    assert_eq!(&byte_slice, from_crate_base58.as_slice());
}

/*
This function works by iterating over each byte in the input array, and for each byte,
it iterates over each character in the result string (which is initially empty).
It multiplies the character by 256 (using bit shifting), adds the byte,
and then divides by 58 to get the new character and the carry.
This is essentially performing the division and remainder operations in base58.

Please note that this function does not handle leading zeros in the input bytes.
If you need to handle leading zeros (which should be converted to '1’s in the Base58 string),
you’ll need to add some additional code at the beginning of the function to count the number of
leading zeros and add the corresponding number of '1’s to the start of the output string.
*/

fn to_base58(bytes: &[u8]) -> String {
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = Vec::new();

    // Count the leading zeros
    let leading_zeros = bytes.iter().take_while(|&&byte| byte == 0).count();

    for &byte in bytes {
        let mut carry = byte as usize;
        for ch in result.iter_mut() {
            let temp = (*ch as usize) << 8 | carry;
            let (quotient, remainder) = (temp / 58, temp % 58);

            carry = quotient;
            *ch = remainder as u8;
        }
        while carry > 0 {
            let (quotient, remainder) = (carry / 58, carry % 58);

            result.push(remainder as u8);
            carry = quotient;
        }
    }

    result.reverse();
    let mut s = String::new();
    // Add '1' for each leading zero
    for _ in 0..leading_zeros {
        s.push('1');
    }
    for &index in result.iter() {
        s.push(alphabet.chars().nth(index as usize).unwrap());
    }

    s
}

/*fn to_base58(bytes: &[u8]) -> String {
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = Vec::new();

    for &byte in bytes {
        let mut carry = byte as usize;
        for ch in result.iter_mut() {
            let temp = (*ch as usize) << 8 | carry;
            let (quotient, remainder) = (temp / 58, temp % 58);

            carry = quotient;
            *ch = remainder as u8;
        }
        while carry > 0 {
            let (quotient, remainder) = (carry / 58, carry % 58);

            result.push(remainder as u8);
            carry = quotient;
        }
    }

    result.reverse();
    let mut s = String::new();
    for &index in result.iter() {
        s.push(alphabet.chars().nth(index as usize).unwrap());
    }

    s
}*/

/*
This version of the function correctly handles the conversion from Base58 back to bytes.
It multiplies each byte in the result by 58 (the base of Base58), adds the value of the current character,
and then divides by 256 (the base of bytes) to get the new byte and the carry.
The carry is then added to the next byte in the next iteration.
*/
fn from_base58(s: &str) -> Vec<u8> {
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = vec![0u8; s.len()]; // Allocate enough space for the result.

    for c in s.chars() {
        if let Some(pos) = alphabet.find(c) {
            let mut carry = pos as u32;
            for byte in result.iter_mut().rev() {
                let temp = (*byte as u32) * 58 + carry;
                *byte = temp as u8;
                carry = temp >> 8;
            }
            assert_eq!(carry, 0);
        } else {
            panic!("Invalid Base58 character: {}", c);
        }
    }

    // Trim leading zeros.
    let leading_zeros = result.iter().take_while(|&&x| x == 0).count();
    result.drain(..leading_zeros);

    result
}

//********************************************** */
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

#[derive(Debug, Default)]
pub enum Base58VersionByte {
    #[default]
    BitcoinP2PKH,
    BitcoinP2SH,
    BitcoinWifPrivateKey,
    BitcoinExtendedPrivateKey,
    BitcoinExtendedPublicKey,

    TestnetP2PKH,
    TestnetP2SH,
    TestnetWifPrivateKey,
    TestnetExtendedPrivateKey,
    TestnetExtendedPublicKey,

    Unsupported,
}

impl Base58VersionByte {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Self::BitcoinP2PKH => [0u8].as_slice(),
            Self::BitcoinP2SH => [5u8].as_slice(),
            Self::BitcoinWifPrivateKey => [128u8].as_slice(),
            Self::BitcoinExtendedPrivateKey => [4, 136, 173, 228].as_slice(),
            Self::BitcoinExtendedPublicKey => [4u8, 136, 178, 30].as_slice(),
            Self::TestnetP2PKH => [111u8].as_slice(),
            Self::TestnetP2SH => [196u8].as_slice(),
            Self::TestnetWifPrivateKey => [239u8].as_slice(),
            Self::TestnetExtendedPrivateKey => [4, 53, 131, 148].as_slice(),
            Self::TestnetExtendedPublicKey => [4, 53, 135, 207].as_slice(),
            Self::Unsupported => panic!(),
        }
    }

    pub const fn to_hex(&self) -> &str {
        match self {
            Self::BitcoinP2PKH => "00",
            Self::BitcoinP2SH => "05",
            Self::BitcoinWifPrivateKey => "80",
            Self::BitcoinExtendedPrivateKey => "0488ADE4",
            Self::BitcoinExtendedPublicKey => "0488B21E",
            Self::TestnetP2PKH => "6F",
            Self::TestnetP2SH => "C4",
            Self::TestnetWifPrivateKey => "EF",
            Self::TestnetExtendedPrivateKey => "04358394",
            Self::TestnetExtendedPublicKey => "043587CF",
            Self::Unsupported => panic!(),
        }
    }

    pub const fn from_bytes(value: &[u8]) -> Self {
        match value {
            &[0u8] => Self::BitcoinP2PKH,
            &[5u8] => Self::BitcoinP2SH,
            &[128u8] => Self::BitcoinWifPrivateKey,
            &[4, 136, 173, 228] => Self::BitcoinExtendedPrivateKey,
            &[4u8, 136, 178, 30] => Self::BitcoinExtendedPublicKey,
            &[111u8] => Self::TestnetP2PKH,
            &[196u8] => Self::TestnetP2SH,
            &[239u8] => Self::TestnetWifPrivateKey,
            &[4, 53, 131, 148] => Self::TestnetExtendedPrivateKey,
            &[4, 53, 135, 207] => Self::TestnetExtendedPublicKey,
            _ => panic!("This can be returned as an error"),
        }
    }

    pub fn to_type(value: &str) -> Self {
        match value {
            "1" => Self::BitcoinP2PKH,
            "3" => Self::BitcoinP2SH,
            "K" | "L" | "5" => Self::BitcoinWifPrivateKey,
            "xprv" => Self::BitcoinExtendedPrivateKey,
            "xpub" => Self::BitcoinExtendedPublicKey,
            "m" | "n" => Self::TestnetP2PKH,
            "2" => Self::TestnetP2SH,
            "c" | "9" => Self::TestnetWifPrivateKey,
            "tprv" => Self::TestnetExtendedPrivateKey,
            "tpub" => Self::TestnetExtendedPublicKey,
            _ => Self::Unsupported,
        }
    }
}

/*
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
*/
