use std::str::FromStr;

use bdk::{
    bitcoin::{
        secp256k1::{rand::rngs::OsRng, Secp256k1, SecretKey},
        Address, Network, PrivateKey, PublicKey,
    },
    blockchain::{Blockchain, ElectrumBlockchain},
    database::MemoryDatabase,
    electrum_client::Client,
    template::P2Pkh,
    wallet::{AddressIndex, Wallet},
    SignOptions, SyncOptions,
};

mod load_data;
pub use load_data::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dbg!(random_wallet());

    let wallet_loader = WalletLoader::load("../data.toml")?;
    dbg!(&wallet_loader);

    let secret_key = SecretKey::from_slice(wallet_loader.secret_key_bytes())?;
    let network = Network::Testnet;
    let priv_key = PrivateKey {
        compressed: true,
        network,
        inner: secret_key,
    };

    let pub_key = P2Pkh(priv_key);

    let wallet = Wallet::new(pub_key, None, network, MemoryDatabase::new())?;
    assert_eq!(
        wallet.get_address(AddressIndex::New).unwrap().to_string(),
        wallet_loader.p2pkh_address()
    );
    let address = wallet.get_address(AddressIndex::New)?;
    dbg!(&address);

    let current_balance = 0.00067183f64;
    let miner_fee = 10_000f64;
    let satoshis_per_btc = 100_000_000f64;
    let send_balance = current_balance - (miner_fee / satoshis_per_btc);

    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();
    let blockchain = ElectrumBlockchain::from(client);
    wallet.sync(&blockchain, SyncOptions::default())?;

    let faucet_address = Address::from_str(wallet_loader.faucet_address())?;
    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(faucet_address.payload.script_pubkey(), send_balance as u64)
        .enable_rbf();
    let (mut psbt, tx_details) = tx_builder.finish()?;

    dbg!(&tx_details);
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    dbg!(&finalized);

    let raw_transaction = psbt.extract_tx();
    let txid = raw_transaction.txid();
    blockchain.broadcast(&raw_transaction)?;
    dbg!(&txid);

    Ok(())
}

pub fn random_wallet() -> (SecretKey, PublicKey, String) {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

    let btc_public_key = PublicKey::new(public_key);

    let network = Network::Testnet;
    let address = Address::p2pkh(&btc_public_key, network);

    (secret_key, btc_public_key, address.to_string())
}
