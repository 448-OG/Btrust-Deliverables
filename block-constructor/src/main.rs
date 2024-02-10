use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
};

pub const MAX_BLOCK_TX_WEIGHT: u32 = 4_000_000;

fn main() {
    let mut miner = Miner::load_mempool("mempool.csv");
    miner.mine();

    for block in miner.finalized.iter() {
        for tx in block {
            println!("{}", &tx.txid);
        }
        println!("\n\n",);
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Miner {
    mempool: Vec<Transaction>,
    finalized_txids: HashSet<String>,
    finalized: Vec<Vec<Transaction>>,
}

impl Miner {
    pub fn load_mempool(path_to_file: impl AsRef<Path>) -> Self {
        let file = File::open(path_to_file.as_ref()).unwrap();
        let buffer = BufReader::new(file);
        let mut init_miner = Miner::default();

        buffer.lines().for_each(|line| {
            let line = line.unwrap();
            let tx = Transaction::parser(line.trim());

            init_miner.mempool.push(tx);
        });

        init_miner.mempool.sort();

        init_miner
    }

    pub fn mine(&mut self) {
        let mut current_block_weight = 0u32;
        let mut current_block = Vec::<Transaction>::new();
        let mut skipped = Vec::<Transaction>::new();

        while let Some(mempool_tx) = self.mempool.pop() {
            if current_block_weight + mempool_tx.weight > MAX_BLOCK_TX_WEIGHT
                || self.mempool.is_empty()
            {
                for tx in current_block.iter() {
                    self.finalized_txids.insert(tx.txid.clone());
                }
                self.finalized.push(current_block.clone());
                current_block.clear();

                current_block_weight = 0;

                while let Some(skipped_tx) = skipped.pop() {
                    self.mempool.push(skipped_tx);
                    self.mempool.sort();
                }
            }

            let has_no_parents = mempool_tx.parent_txids.is_empty();

            if has_no_parents {
                current_block_weight += mempool_tx.weight;
                current_block.push(mempool_tx);
            } else {
                let mut all_parents_mined = Vec::<bool>::new();

                for current_parent_txid in mempool_tx.parent_txids.iter() {
                    let contains_tx = self.finalized_txids.contains(current_parent_txid);
                    all_parents_mined.push(contains_tx);
                }

                let mut should_be_skipped = false;

                all_parents_mined.iter().for_each(|element| {
                    if !element {
                        should_be_skipped = true;
                    }
                });

                if !should_be_skipped {
                    current_block_weight += mempool_tx.weight;
                    current_block.push(mempool_tx);
                } else {
                    skipped.push(mempool_tx);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct Transaction {
    txid: String,
    fee: u64,
    weight: u32,
    parent_txids: Vec<String>,
}

impl Transaction {
    fn parser(value: &str) -> Self {
        let mut outcome = Self::default();
        let tx_data = value.split(',').collect::<Vec<&str>>();

        let txid = tx_data.first().unwrap().trim();
        let fee = tx_data.get(1).unwrap().trim();
        let weight = tx_data.get(2).unwrap().trim();
        let parents = tx_data.get(3);

        outcome.txid = txid.trim().to_owned();
        outcome.fee = fee.parse::<u64>().unwrap();
        outcome.weight = weight.parse::<u32>().unwrap();

        if let Some(parent_exists) = parents {
            parent_exists.trim().split(';').for_each(|parent| {
                if !parent.is_empty() {
                    outcome.parent_txids.push(parent.trim().to_owned());
                }
            });
        }

        // Reverse the parents order since ancestors of a transaction would need to be in
        // the mempool for a UTXO to be valid
        outcome.parent_txids.reverse();

        outcome
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let self_fee_rate = self.fee / self.weight as u64;
        let other_fee_rate = other.fee / other.weight as u64;

        other_fee_rate.cmp(&self_fee_rate)
    }
}
