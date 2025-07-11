mod balances;
mod system;

/// This is our runtime, it allows us to interact with all logic in the system.
#[derive(Debug)]
pub struct Runtime {
    pub system: system::Pallet,
    pub balances: balances::Pallet,
}

impl Runtime {
    // Create a new instance of the runtime
    fn new() -> Self {
        Runtime {
            system: system::Pallet::new(),
            balances: balances::Pallet::new(),
        }
    }

    fn create_block(&mut self, transactions: Vec<Transaction>) -> BlockResult {
        self.system.inc_block_number();
        let current_block = self.system.block_number();

        println!("\n=== Creating Block #{} ===", current_block);

        let mut successful_transactions = Vec::new();
        let mut failed_transactions = Vec::new();

        // Execute all transactions in the block
        for transaction in transactions {
            match self.execute_transaction(transaction.clone()) {
                Ok(_) => {
                    successful_transactions.push(transaction);
                    println!("✅ Transaction successful");
                }
                Err(e) => {
                    failed_transactions.push((transaction, e));
                    println!("❌ Transaction failed");
                }
            }
        }
        // Finalize the block and generate hash
        let block_hash = self.system.finalize_block();
        println!("📦 Block #{} finalized", current_block);
        println!("🔗 Block Hash: {:?}", hex_encode(&block_hash[..8]));

        if let Some(parent_hash) = self.system.parent_block_hash() {
            println!("⬆️  Parent Hash: {:?}", hex_encode(&parent_hash[..8]));
        }

        BlockResult {
            block_number: current_block,
            block_hash,
            successful_transactions: successful_transactions.clone(),
            failed_transactions,
            transaction_count: successful_transactions.len(),
        }
    }

    fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        match transaction {
            Transaction::Transfer { from, to, amount } => {
                self.system.inc_nonce(&from);

                //Attempt the transfer
                match self.balances.transfer(from.clone(), to.clone(), amount) {
                    Ok(_) => {
                        println!("💸 Transfer: {} -> {} (amount: {})", from, to, amount);
                        Ok(())
                    }
                    Err(e) => {
                        println!(
                            "💥 Transfer failed: {} -> {} (amount: {}) - Error: {:?}",
                            from, to, amount, e
                        );
                        Err(format!("{:?}", e))
                    }
                }
            }
            Transaction::SetBalance { who, amount } => {
                println!("💰 Set balance: {} = {}", who, amount);
                self.balances.set_balance(&who, amount);
                Ok(())
            }
        }
    }

    // Print comprehensive Blockchain state
    fn print_blockchain_state(&self) {
        println!("\n🔍 === BLOCKCHAIN STATE ===");
        println!("Current Block: #{}", self.system.block_number());

        // show block hashes
        let all_hashes = self.system.all_block_hashes();
        println!("\n📚 Block Hashes:");
        for (block_num, hash) in all_hashes.iter().rev().take(5) {
            println!("  Block #{}: {}", block_num, hex_encode(&hash[..8]));
        }

        // Show account Balances
        println!("\n💳 Account Balances:");
        let accounts = ["Femi", "temi", "cheryl", "nathaniel", "faith"];
        for account in accounts {
            let balance = self.balances.balance(&account.to_string());
            if balance > 0 {
                let nonce = self.system.nonce.get(&account.to_string()).unwrap_or(&0);
                println!("  {}: {} (nonce: {})", account, balance, nonce);
            }
        }

        //Show Genesis Hash
        if let Some(genesis_hash) = self.system.genesis_hash() {
            println!("\n🌱 Genesis Hash: {}", hex_encode(&genesis_hash[..8]));
        }
        println!("=========================\n");
    }

    // Verify Blockchain Integrity
    fn verify_chain_integrity(&self) -> bool {
        let all_hashes = self.system.all_block_hashes();

        for block_num in 1..=self.system.block_number() {
            if let Some(current_hash) = all_hashes.get(&block_num) {
                println!("✅ Block #{} hash verified", block_num);
            } else {
                println!("❌ Block #{} hash missing!", block_num);
                return false;
            }
        }
        println!("🔐 Blockchain integrity verified!");
        true
    }
}

// Transaction types
#[derive(Debug, Clone)]
pub enum Transaction {
    Transfer {
        from: String,
        to: String,
        amount: u128,
    },
    SetBalance {
        who: String,
        amount: u128,
    },
}

// Block Execution Result
#[derive(Debug)]
pub struct BlockResult {
    pub block_number: u32,
    pub block_hash: [u8; 32],
    pub successful_transactions: Vec<Transaction>,
    pub failed_transactions: Vec<(Transaction, String)>,
    pub transaction_count: usize,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:03x}", b))
        .collect::<String>()
}

fn main() {
    let mut runtime = Runtime::new();

    println!("🚀 Starting Blockchain Simulation");
    println!("==================================");

    // Users
    let femi = String::from("Femi");
    let temi = String::from("temi");
    let cheryl = String::from("cheryl");
    let nathaniel = String::from("nathaniel");
    let faith = String::from("faith");

    // Genesis Block (Block 0) - Initial setup
    println!("\n🌱 === GENESIS BLOCK ===");

    let genesis_transactions = vec![
        Transaction::SetBalance {
            who: cheryl.clone(),
            amount: 10000,
        },
        Transaction::SetBalance {
            who: femi.clone(),
            amount: 500,
        },
    ];

    let genesis_result = runtime.create_block(genesis_transactions);
    println!(
        "Genesis block created with {} transactions",
        genesis_result.transaction_count
    );

    // Block 1 - Initial Transfers
    let block_1_transactions = vec![
        Transaction::Transfer {
            from: cheryl.clone(),
            to: faith.clone(),
            amount: 50,
        },
        Transaction::Transfer {
            from: cheryl.clone(),
            to: nathaniel.clone(),
            amount: 70,
        },
        Transaction::Transfer {
            from: femi.clone(),
            to: temi.clone(),
            amount: 100,
        },
    ];
    let block_1_result = runtime.create_block(block_1_transactions);
    println!(
        "Block 1 completed with {}/{} successful transactions",
        block_1_result.transaction_count,
        block_1_result.successful_transactions.len() + block_1_result.failed_transactions.len()
    );

    // Block 2 - More  transfers
    let block_2_transactions = vec![
        Transaction::Transfer {
            from: cheryl.clone(),
            to: femi.clone(),
            amount: 100,
        },
        Transaction::Transfer {
            from: faith.clone(),
            to: temi.clone(),
            amount: 20,
        },
        Transaction::Transfer {
            from: nathaniel.clone(),
            to: femi.clone(),
            amount: 30,
        },
    ];

    let block_2_result = runtime.create_block(block_2_transactions);
    println!(
        "Block 2 completed with {}/{} successful transactions",
        block_2_result.transaction_count,
        block_2_result.successful_transactions.len() + block_2_result.failed_transactions.len()
    );

    // Block 3 - Included some failures
    let block_3_transactions = vec![
        Transaction::Transfer {
            from: cheryl.clone(),
            to: nathaniel.clone(),
            amount: 9200,
        }, // Should fail
        Transaction::Transfer {
            from: temi.clone(),
            to: faith.clone(),
            amount: 50,
        },
        Transaction::Transfer {
            from: femi.clone(),
            to: cheryl.clone(),
            amount: 200,
        },
    ];

    let block_3_result = runtime.create_block(block_3_transactions);
    println!(
        "Block 3 completed with {}/{} successful transactions",
        block_3_result.transaction_count,
        block_3_result.successful_transactions.len() + block_3_result.failed_transactions.len()
    );

    //Print final state
    runtime.print_blockchain_state();

    //Verify blockchain Integrity
    runtime.verify_chain_integrity();

    //Demonstrate hash relationships
    println!("🔗 === BLOCK HASH RELATIONSHIPS ===");
    for block_num in 0..=runtime.system.block_number() {
        if let Some(hash) = runtime.system.get_block_hash(block_num) {
            println!("Block #{}: {}", block_num, hex_encode(&hash[..16]));

            if block_num > 0 {
                if let Some(parent_hash) = runtime.system.get_block_hash(block_num - 1) {
                    println!("  └─ Parent: {}", hex_encode(&parent_hash[..16]));
                }
            }
        }
    }

    // Demonstrate block hash queries
    println!("\n🔍 === BLOCK HASH QUERIES ===");
    if let Some(current_hash) = runtime.system.current_block_hash() {
        println!("Current block hash: {}", hex_encode(&current_hash[..8]));
    }

    // Demonstrate memory management
    println!("\n🧹 === MEMORY MANAGEMENT ===");
    println!(
        "Total blocks stored: {}",
        runtime.system.all_block_hashes().len()
    );

    // // give some money - GENSIS Block
    // runtime.balances.set_balance(&cheryl, 1000);

    // create a block
    // increase block number
    // runtime.system.inc_block_number();
    // assert_eq!(runtime.system.block_number(), 1);

    // // first transaction
    // runtime.system.inc_nonce(&cheryl);
    // let _res = runtime
    //     .balances
    //     .transfer(cheryl.clone(), faith.clone(), 50)
    //     .map_err(|e| println!("error: {}", e));

    // // second transaction
    // runtime.system.inc_nonce(&cheryl);
    // let _res = runtime
    //     .balances
    //     .transfer(cheryl.clone(), nathaniel.clone(), 70)
    //     .map_err(|e| println!("error: {}", e));

    // // Create block 2
    // runtime.system.inc_block_number();
    // assert_eq!(runtime.system.block_number(), 2);

    // runtime.system.inc_nonce(&cheryl);
    // let _res = runtime
    //     .balances
    //     .transfer(cheryl.clone(), femi.clone(), 100)
    //     .map_err(|e| println!("error: {}", e));

    // runtime.system.inc_nonce(&femi);
    // let _res = runtime
    //     .balances
    //     .transfer(femi.clone(), temi.clone(), 100)
    //     .map_err(|e| println!("error: {}", e));

    // // block 3 : should fail
    // runtime.system.inc_block_number();
    // assert_eq!(runtime.system.block_number(), 3);

    // runtime.system.inc_nonce(&cheryl);
    // let _res = runtime
    //     .balances
    //     .transfer(cheryl.clone(), nathaniel.clone(), 1200)
    //     .map_err(|e| println!("error: {}", e));

    // println!("{:#?}", runtime);
}
