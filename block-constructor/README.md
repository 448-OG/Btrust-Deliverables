### Block Constructor Challenge

##### Algorithm
1. Read the mempool line by line parsing each line into a transaction
2. Parents are parsed by splitting at `;` symbol and read into a vector
3. Reverve the vector of parents to ensure that ancestors come first since a transaction can only reference a transaction that has already been mined
4. Implement sorting in order to sort transactions by most profitable fee rate `(fee / weight)`
5. Mine the transactions ensuring that transactions with parent transaction IDs that have not been mined are skipped in each iteration
6. Push transactions into a block and mine them
7. After mining them, print all blocks on the console separating all blocks using a newline


##### Code docs
1. Import std types and define the maximum block weight as a const
    ```rust
    use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
    };

    pub const MAX_BLOCK_TX_WEIGHT: u32 = 4_000_000;

    ```
2. Create a `Transaction` struct used to parse transactions. Implement `PartialOrd` and `Ord` struct in order to sort a transactions based on fee rate as mentioned in the algorithm section.
    ```rust
    /// Define a transaction 
    #[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
    pub struct Transaction {
        txid: String,
        fee: u64,
        weight: u32,
        parent_txids: Vec<String>,
    }

    impl Transaction {
        /// Implement a parser that splits transactions
        /// based on their individual components at `,`
        /// and parents at `;'
        fn parser(value: &str) -> Self {
            let mut outcome = Self::default();
            // Split transaction into components
            let tx_data = value.split(',').collect::<Vec<&str>>();

            /// Get transaction ID
            let txid = tx_data.first().unwrap().trim();
            /// Get transaction fee
            let fee = tx_data.get(1).unwrap().trim();
            /// Get transaction weight
            let weight = tx_data.get(2).unwrap().trim();
            /// Get parents
            let parents = tx_data.get(3);

            outcome.txid = txid.trim().to_owned();
            outcome.fee = fee.parse::<u64>().unwrap();
            outcome.weight = weight.parse::<u32>().unwrap();

            /// Split parents based on ';' character
            if let Some(parent_exists) = parents {
                parent_exists.trim().split(';').for_each(|parent| {
                    if !parent.is_empty() {
                        outcome.parent_txids.push(parent.trim().to_owned());
                    }
                });
            }

            // Reverse the parents order since ancestors
            // of a transaction would need to be in
            // the mempool for a UTXO to be valid
            outcome.parent_txids.reverse();

            outcome
        }
    }

    /// Implement Ordering for transactions
    /// based on `fee / weight` which is the fee rate
    impl Ord for Transaction {
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            let self_fee_rate = self.fee / self.weight as u64;
            let other_fee_rate = other.fee / other.weight as u64;

            other_fee_rate.cmp(&self_fee_rate)
        }
    }

    /// Impl `PartialOrd` by invoking implementation
    /// of `Ord`
    impl PartialOrd for Transaction {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    ```
3. Create a `Miner` struct that performs fetching transactions from the mempool and grouping them into blocks maximizing fee rate
    ```rust
    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct Miner {
        // The mempool that holds all parsed transactions
        mempool: Vec<Transaction>,
        // The currently mined transaction IDS
        // for easier lookup of already processed
        // transactions
        finalized_txids: HashSet<String>,
        // the mined transactions
        finalized: Vec<Vec<Transaction>>,
    }
    ```
4.  Loader for the mempool to load parsed transactions into miner's memory
    ```rust
    impl Miner {
        /// Load the mempool and initialize other fields with defaults.
        /// This field takes a path to the file
        pub fn load_mempool(path_to_file: impl AsRef<Path>) -> Self {

            let file = File::open(path_to_file.as_ref()).unwrap();
            let buffer = BufReader::new(file);
            let mut init_miner = Miner::default();

            // Read each line
            buffer.lines().for_each(|line| {
                let line = line.unwrap();
                // Call transaction parser on each line
                let tx = Transaction::parser(line.trim());

                // Then push each transaction into the mempool
                init_miner.mempool.push(tx);
            });

            // Sort the mempool based on most profitable
            // transactions since we implemented
            // sorting of transactions based on
            // fee rate `(fee / weight)`
            init_miner.mempool.sort();

            init_miner
        }
    }
    ```
5.  Mine all the blocks
    ```rust
   
    impl Miner {
        // ... Already implemented code here

        /// Implement code to mine,
        /// organize and reorganize transactions
        /// based on most profitable ones
        pub fn mine(&mut self) {
            // Initialize an empty block weight
            let mut current_block_weight = 0u32;
            // Initialize an empty block
            let mut current_block = Vec::<Transaction>::new();
            // Initialize a temporary buffer
            // to hold transactions from mempool
            // that have parents who have not
            // been mined
            let mut skipped = Vec::<Transaction>::new();

            // Iterate over the mempool popping
            // the first transaction. This ensures
            // we don't have to iterate over the mempool
            // again checking if a transaction has already
            // been mined
            while let Some(mut mempool_tx) = self.mempool.pop() {
                // Check if adding this transaction
                // will exceed block weight or if the mempool is empty since all transactions 
                // have already been looped over
                if current_block_weight + mempool_tx.weight > MAX_BLOCK_TX_WEIGHT
                    || self.mempool.is_empty()
                {
                    // Iterate over the current block,
                    // get transaction IDs 
                    for tx in current_block.iter() {
                        // Add transaction IDs for easier reference
                        self.finalized_txids.insert(tx.txid.clone());
                    }
                    //Push the current block into the
                    // list of mined blocks
                    self.finalized.push(current_block.clone());

                    // Clear the current block
                    current_block.clear();

                    // Reset the block weight
                    current_block_weight = 0;

                    // Re-insert the skipped transactions
                    // which have a parent that has not yet
                    // been mined.
                    // `pop()` in order to avoid cloning
                    while let Some(skipped_tx) = skipped.pop() {
                        self.mempool.push(skipped_tx);
                        self.mempool.sort();
                    }
                }

                // Check if current transaction has parents
                let has_no_parents = mempool_tx.parent_txids.is_empty();

                // If no parents then add transaction
                // to current block and increase the 
                // current block weight
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
    ```
    
6. Initialize and mine blocks
    ```rust
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

    ```


###### Licensed Under CC0-1.0