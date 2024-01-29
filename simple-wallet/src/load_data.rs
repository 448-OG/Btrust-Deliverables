use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, prelude::*},
};
pub type Byte32Array = [u8; 32];

#[derive(Serialize, Deserialize)]
pub struct WalletLoader {
    p2pkh_address: String,
    secret_key_bytes: Byte32Array,
    faucet_address: String,
}

impl WalletLoader {
    pub fn load(uri: &str) -> io::Result<Self> {
        let mut file = File::open(uri)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(toml::from_str::<Self>(&contents).unwrap())
    }

    pub fn p2pkh_address(&self) -> &str {
        self.p2pkh_address.as_str()
    }

    pub fn secret_key_bytes(&self) -> &Byte32Array {
        &self.secret_key_bytes
    }

    pub fn faucet_address(&self) -> &str {
        self.faucet_address.as_str()
    }
}

impl fmt::Debug for WalletLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalletLoader")
            .field("p2pkh_address", &self.p2pkh_address)
            .field(
                "secret_key_bytes(Blake3Hash)",
                &blake3::hash(&self.secret_key_bytes).to_hex(),
            )
            .field("faucet_address", &self.faucet_address)
            .finish()
    }
}
