use bitcoin::{consensus::encode::deserialize, Transaction};

fn main() {
    let transaction_bytes =  hex_literal::hex!("020000000001010ccc140e766b5dbc884ea2d780c5e91e4eb77597ae64288a42575228b79e234900000000000000000002bd37060000000000225120245091249f4f29d30820e5f36e1e5d477dc3386144220bd6f35839e94de4b9cae81c00000000000016001416d31d7632aa17b3b316b813c0a3177f5b6150200140838a1f0f1ee607b54abf0a3f55792f6f8d09c3eb7a9fa46cd4976f2137ca2e3f4a901e314e1b827c3332d7e1865ffe1d7ff5f5d7576a9000f354487a09de44cd00000000");

    let decoded_tx = TxDecoder::new(&transaction_bytes);
    decoded_tx
        .version()
        .inputs()
        .outputs()
        .locktime()
        .base_size()
        .coinbase()
        .rbf()
        .nomalized()
        .segwit_id()
        .total_size()
        .weight();
}

pub struct TxDecoder(Transaction);

impl TxDecoder {
    pub fn new(transaction_bytes: &[u8]) -> Self {
        let decoded: Transaction = deserialize(&transaction_bytes)
            .expect("Error Decoding Transaction. Check your hex characters are correct");

        Self(decoded)
    }

    pub fn get(&self) -> &Transaction {
        &self.0
    }

    pub fn version(&self) -> &Self {
        println!("Version: {:?}", self.0.version.0);

        self
    }

    pub fn inputs(&self) -> &Self {
        println!("Inputs:");
        for tx_input in &self.0.input {
            println!("- Transaction ID: {}", tx_input.previous_output.txid);
            println!("- Transaction index: {}", tx_input.previous_output.vout);
        }

        self
    }

    pub fn outputs(&self) -> &Self {
        println!("Outputs:");
        for tx_output in &self.0.output {
            println!("- Value: {}", tx_output.value);
            println!("- Script: {}", tx_output.script_pubkey);
        }

        self
    }

    pub fn locktime(&self) -> &Self {
        println!("Locktime: {}", self.0.lock_time);

        self
    }

    pub fn base_size(&self) -> &Self {
        println!("Base Size: {:?}", self.0.base_size());

        self
    }

    pub fn coinbase(&self) -> &Self {
        if self.0.is_coinbase() {
            println!("This is a Coin Base Ttransaction",);
        } else {
            println!("This is Not a Coin Base Ttransaction",);
        }

        self
    }

    pub fn rbf(&self) -> &Self {
        println!("Opted for RBF: {:?}", self.0.is_explicitly_rbf());

        self
    }

    pub fn nomalized(&self) -> &Self {
        println!("Nomalized Transaction ID: {:?}", self.0.ntxid());

        self
    }

    pub fn segwit_id(&self) -> &Self {
        let segwit_txid = self.0.wtxid();

        if segwit_txid.as_raw_hash() != self.0.txid().as_raw_hash() {
            println!("Segwit Transaction ID: {:?}", segwit_txid);
        } else {
            println!("Transaction is not Segwit");
        }

        self
    }

    pub fn total_size(&self) -> &Self {
        println!("Transaction Total Size: {:?}", self.0.total_size());

        self
    }

    pub fn weight(&self) -> &Self {
        println!("Transaction Weight: {}", self.0.weight().to_string());

        self
    }
}

#[cfg(test)]
mod correctness_test {
    use crate::TxDecoder;
    use bitcoin::{absolute::LockTime, transaction::Version, Txid, Weight};
    use std::str::FromStr;

    #[test]
    fn do_test() {
        let transaction_bytes =  hex_literal::hex!("020000000001010ccc140e766b5dbc884ea2d780c5e91e4eb77597ae64288a42575228b79e234900000000000000000002bd37060000000000225120245091249f4f29d30820e5f36e1e5d477dc3386144220bd6f35839e94de4b9cae81c00000000000016001416d31d7632aa17b3b316b813c0a3177f5b6150200140838a1f0f1ee607b54abf0a3f55792f6f8d09c3eb7a9fa46cd4976f2137ca2e3f4a901e314e1b827c3332d7e1865ffe1d7ff5f5d7576a9000f354487a09de44cd00000000");

        let decoded_tx = TxDecoder::new(&transaction_bytes);
        decoded_tx
            .version()
            .inputs()
            .outputs()
            .locktime()
            .base_size()
            .coinbase()
            .rbf()
            .nomalized()
            .segwit_id()
            .total_size()
            .weight();

        {
            // Script Tests
            assert_eq!(Version(2), decoded_tx.get().version);
            for tx_input in &decoded_tx.get().input {
                assert_eq!(
                    Txid::from_str(
                        "49239eb7285257428a2864ae9775b74e1ee9c580d7a24e88bc5d6b760e14cc0c"
                    )
                    .unwrap(),
                    tx_input.previous_output.txid
                );
                assert_eq!(0, tx_input.previous_output.vout);
            }

            assert_eq!(LockTime::ZERO, decoded_tx.get().lock_time);
            assert_eq!(125usize, decoded_tx.get().base_size());
            assert_eq!(false, decoded_tx.get().is_coinbase());
            assert_eq!(true, decoded_tx.get().is_explicitly_rbf());
            assert_eq!(
                "6d9da35544e87a88279c5bfc66e08a873f3d456b4d6112620e2c41555863f920",
                decoded_tx.get().ntxid().to_string().as_str()
            );
            let is_segwit = if decoded_tx.get().wtxid().as_raw_hash()
                != decoded_tx.get().txid().as_raw_hash()
            {
                true
            } else {
                false
            };
            assert!(is_segwit);
            assert_eq!(193usize, decoded_tx.get().total_size());
            assert_eq!(Weight::from_wu(568u64), decoded_tx.get().weight());
        }
    }
}
