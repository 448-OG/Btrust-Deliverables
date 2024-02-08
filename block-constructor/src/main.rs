use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};

pub const MAX_BLOCK_TX_WEIGHT: u32 = 4_000_000;

fn main() {
    let mut mempool = Mempool::new("mempool.csv").load_mempool();
    dbg!(&mempool.blocks.len());
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Mempool {
    txs: Vec<Transaction>,
    uri: PathBuf,
    blocks: Vec<Vec<Transaction>>,
}

impl Mempool {
    pub fn new(uri: impl AsRef<Path>) -> Self {
        Self {
            uri: uri.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn load_mempool(mut self) -> Self {
        let file = File::open(&self.uri).unwrap();
        let buffer = BufReader::new(file);

        let mut prepare_genesis = VecDeque::<Transaction>::new();

        buffer.lines().into_iter().for_each(|line| {
            let line = line.unwrap();
            let tx = Transaction::parser(&line.trim());

            if tx.parent_txids.is_empty() {
                prepare_genesis.push_back(tx);
            } else {
                self.txs.push(tx);
            }
        });

        prepare_genesis.make_contiguous().sort();

        let mut weight_count = 0u32;
        let mut block_buffer = Vec::<Transaction>::new();

        while let Some(tx) = prepare_genesis.pop_front() {
            if tx.weight + weight_count > MAX_BLOCK_TX_WEIGHT {
                weight_count = 0;
                self.blocks.push(block_buffer.clone());
                block_buffer.clear();
            }
            weight_count += tx.weight;

            block_buffer.push(tx);
        }

        self.blocks.push(block_buffer.clone());

        block_buffer.clear();

        self
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Transaction {
    txid: String,
    fee: u64,
    weight: u32,
    parent_txids: Vec<String>,
}

impl Transaction {
    fn parser(value: &str) -> Self {
        let mut outcome = Self::default();
        let tx_data = value.split(",").collect::<Vec<&str>>();

        let txid = tx_data.get(0).unwrap().trim();
        let fee = tx_data.get(1).unwrap().trim();
        let weight = tx_data.get(2).unwrap().trim();
        let parents = tx_data.get(3);

        outcome.txid = txid.trim().to_owned();
        outcome.fee = fee.parse::<u64>().unwrap();
        outcome.weight = weight.parse::<u32>().unwrap();

        if let Some(parent_exists) = parents {
            parent_exists.trim().split(";").for_each(|parent| {
                if !parent.is_empty() {
                    outcome.parent_txids.push(parent.trim().to_owned());
                }
            });
        }

        outcome
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_fee_rate = self.fee / self.weight as u64;
        let other_fee_rate = other.fee / other.weight as u64;

        self_fee_rate.partial_cmp(&other_fee_rate)
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let self_fee_rate = self.fee / self.weight as u64;
        let other_fee_rate = other.fee / other.weight as u64;

        self_fee_rate.cmp(&other_fee_rate)
    }
}
