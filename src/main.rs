use support::Dispatch;

mod balances;
mod staking;
mod support;
mod system;

// Type module - this is where we define all the concrete types for our runtime
mod types {
    pub type AccountId = String;        // Accounts are represented as Strings
    pub type Balance = u128;           // Balances are 128-bit unsigned integers
    pub type BlockNumber = u32;        // Block numbers are 32-bit unsigned integers
    pub type Nonce = u32;             // Nonces are 32-bit unsigned integers
    
    // Complex types built from the basic types
    pub type Extrinsic = crate::support::Extrinsic<AccountId, crate::RuntimeCall>;
    pub type Header = crate::support::Header<BlockNumber>;
    pub type Block = crate::support::Block<Header, Extrinsic>;
}

// This enum contains all the calls available to our runtime
// Each pallet contributes its calls here
pub enum RuntimeCall {
    Balances(balances::Call<Runtime>),  // Balances pallet calls
    Staking(staking::Call<Runtime>),    // Staking pallet calls
}

// Our main Runtime struct - this implements the Config traits for all pallets
#[derive(Debug)]
pub struct Runtime {
    pub system: system::Pallet<Self>,    // Self refers to Runtime
    pub balances: balances::Pallet<Self>,
    pub staking: staking::Pallet<Self>,  // Add staking pallet
}

// Implement system::Config for Runtime
// This tells the system pallet what types to use
impl system::Config for Runtime {
    type AccountId = types::AccountId;     // Use String for accounts
    type BlockNumber = types::BlockNumber; // Use u32 for block numbers
    type Nonce = types::Nonce;            // Use u32 for nonces
}

// Implement balances::Config for Runtime
// This tells the balances pallet what types to use
impl balances::Config for Runtime {
    type Balance = types::Balance;  // Use u128 for balances
}

// Implement staking::Config for Runtime
// This tells the staking pallet what types to use
impl staking::Config for Runtime {
    type Balance = types::Balance;  // Use u128 for staking balances too
}

impl Runtime {
    // Create a new instance of the runtime
    fn new() -> Self {
        Runtime {
            system: system::Pallet::new(),   // Create system pallet with Runtime's config
            balances: balances::Pallet::new(), // Create balances pallet with Runtime's config
            staking: staking::Pallet::new_with_config(100, 5, 10, 10), // Create staking pallet with config
        }
    }

    fn create_block(&mut self, transactions: Vec<Transaction>) -> BlockResult {
        self.system.inc_block_number();
        let current_block = self.system.block_number();

        // Notify staking pallet about new block
        self.staking.on_block(current_block);

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
        
        // Print staking events for this block
        self.print_staking_events();
        
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

                // Attempt the transfer using the generic balances pallet
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
            Transaction::AddValidator { validator, commission } => {
                match self.staking.add_validator(validator.clone(), commission) {
                    staking::Result::Ok(_) => {
                        println!("✅ Validator added: {} (commission: {}%)", validator, commission);
                        Ok(())
                    }
                    staking::Result::Err(e) => {
                        println!("❌ Failed to add validator: {} - Error: {:?}", validator, e);
                        Err(format!("{:?}", e))
                    }
                }
            }
            Transaction::Stake { who, amount, validator } => {
                self.system.inc_nonce(&who);

                // Create a closure that checks balance
                let balances = &self.balances;
                let balance_check = |account: &String| -> u128 { balances.balance(account) };

                match self.staking.stake(who.clone(), amount, validator.clone(), balance_check) {
                    Ok(_) => {
                        // Deduct the staked amount from balance
                        let current_balance = self.balances.balance(&who);
                        self.balances.set_balance(&who, current_balance - amount);
                        println!("🔒 Staked: {} staked {} with validator {}", who, amount, validator);
                        Ok(())
                    }
                    Err(e) => {
                        println!("❌ Staking failed for {}: {:?}", who, e);
                        Err(format!("{:?}", e))
                    }
                }
            }
            Transaction::Unstake { who } => {
                self.system.inc_nonce(&who);

                match self.staking.unstake(who.clone()) {
                    Ok(amount) => {
                        // Return the unstaked amount to balance
                        let current_balance = self.balances.balance(&who);
                        self.balances.set_balance(&who, current_balance + amount);
                        println!("🔓 Unstaked: {} unstaked {} tokens", who, amount);
                        Ok(())
                    }
                    Err(e) => {
                        println!("❌ Unstaking failed for {}: {:?}", who, e);
                        Err(format!("{:?}", e))
                    }
                }
            }
            Transaction::ClaimRewards { who } => {
                self.system.inc_nonce(&who);

                match self.staking.claim_rewards(who.clone()) {
                    Ok(rewards) => {
                        // Add rewards to balance
                        let current_balance = self.balances.balance(&who);
                        self.balances.set_balance(&who, current_balance + rewards);
                        println!("🎁 Rewards claimed: {} received {} tokens", who, rewards);
                        Ok(())
                    }
                    Err(e) => {
                        println!("❌ Failed to claim rewards for {}: {:?}", who, e);
                        Err(format!("{:?}", e))
                    }
                }
            }
        }
    }

    // Execute a block using the support framework
    fn execute_block(&mut self, block: types::Block) -> support::DispatchResult {
        self.system.inc_block_number();

        if self.system.block_number() != block.header.block_number {
            return Err("block number does not match what is expected");
        }

        // Process each extrinsic in the block
        for (i, support::Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
            self.system.inc_nonce(&caller);
            let _res = self.dispatch(caller, call).map_err(|e| {
                eprintln!(
                    "Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
                    block.header.block_number, i, e
                )
            });
        }

        Ok(())
    }

    // Print comprehensive blockchain state - updated to include staking info
    fn print_blockchain_state(&self) {
        println!("\n🔍 === BLOCKCHAIN STATE ===");
        println!("Current Block: #{}", self.system.block_number());

        // Show block hashes
        let all_hashes = self.system.all_block_hashes();
        println!("\n📚 Block Hashes:");
        for (block_num, hash) in all_hashes.iter().rev().take(5) {
            println!("  Block #{}: {}", block_num, hex_encode(&hash[..8]));
        }

        // Show account balances
        println!("\n💳 Account Balances:");
        let accounts = ["Femi", "temi", "cheryl", "nathaniel", "faith"];
        for account in accounts {
            let balance = self.balances.balance(&account.to_string());
            if balance > 0 {
                let nonce = self.system.nonce.get(&account.to_string()).unwrap_or(&0);
                println!("  {}: {} (nonce: {})", account, balance, nonce);
            }
        }

        // Show Staking Information
        println!("\n🔒 Staking Information:");
        let stats = self.staking.get_staking_stats();
        println!("  Total Staked: {}", stats.total_staked);
        println!("  Active Validators: {}/{}", stats.active_validators, stats.total_validators);
        println!("  Total Stakers: {}", stats.total_stakers);

        // Show validators
        if stats.total_validators > 0 {
            println!("\n  Validators:");
            for (validator, info) in self.staking.get_active_validators() {
                println!(
                    "    • {}: {} staked ({}% commission, {} nominators)",
                    validator, info.total_stake, info.commission_rate, info.nominators_count
                );
            }
        }

        // Show stakers
        for account in accounts {
            if let Some(stake_info) = self.staking.get_stake_info(&account.to_string()) {
                println!(
                    "    • {} staking {} with {} (rewards: {})",
                    account, stake_info.staked_amount, stake_info.validator, stake_info.total_rewards
                );
            }
        }

        if let Some(genesis_hash) = self.system.genesis_hash() {
            println!("\n🌱 Genesis Hash: {}", hex_encode(&genesis_hash[..8]));
        }
        println!("=========================\n");
    }

    // Verify Blockchain Integrity
    fn verify_chain_integrity(&self) -> bool {
        let all_hashes = self.system.all_block_hashes();

        for block_num in 1..=self.system.block_number() {
            if let Some(_current_hash) = all_hashes.get(&block_num) {
                println!("✅ Block #{} hash verified", block_num);
            } else {
                println!("❌ Block #{} hash missing!", block_num);
                return false;
            }
        }
        println!("🔐 Blockchain integrity verified!");
        true
    }

    /// Print staking events
    fn print_staking_events(&self) {
        let events = self.staking.get_events();
        if !events.is_empty() {
            println!("\n📋 Staking Events:");
            for event in events {
                match event {
                    staking::StakingEvent::ValidatorAdded { validator } => {
                        println!("  • Validator added: {}", validator);
                    }
                    staking::StakingEvent::Staked { who, amount, validator } => {
                        println!("  • {} staked {} tokens with {}", who, amount, validator);
                    }
                    staking::StakingEvent::Unstaked { who, amount } => {
                        println!("  • {} unstaked {} tokens", who, amount);
                    }
                    staking::StakingEvent::RewardsPaid { who, amount } => {
                        println!("  • {} received {} tokens in rewards", who, amount);
                    }
                    _ => {}
                }
            }
        }
    }
}

// Implement the Dispatch trait for Runtime
// This allows the runtime to route calls to the appropriate pallet
impl support::Dispatch for Runtime {
    type Caller = <Runtime as system::Config>::AccountId;  // Use the AccountId from our config
    type Call = RuntimeCall;

    fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> support::DispatchResult {
        match call {
            RuntimeCall::Balances(call) => {
                self.balances.dispatch(caller, call)?;  // Route to balances pallet
            }
            RuntimeCall::Staking(call) => {
                self.staking.dispatch(caller, call)?;   // Route to staking pallet
            }
        }
        Ok(())
    }
}

// Transaction types for our simplified API
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
    AddValidator {
        validator: String,
        commission: u8,
    },
    Stake {
        who: String,
        amount: u128,
        validator: String,
    },
    Unstake {
        who: String,
    },
    ClaimRewards {
        who: String,
    },
}

// Block execution result
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
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn main() {
    let mut runtime = Runtime::new();

    println!("🚀 Starting Blockchain Simulation with Generics");
    println!("===============================================");

    // Users - these are of type String (our AccountId type)
    let cheryl = String::from("cheryl");
    let femi = String::from("Femi");
    let temi = String::from("temi");
    let nathaniel = String::from("nathaniel");
    let faith = String::from("faith");

    // Genesis Block - Initial setup
    println!("\n🌱 === GENESIS BLOCK ===");
    let genesis_transactions = vec![
        Transaction::SetBalance {
            who: cheryl.clone(),
            amount: 10000,  // This is of type u128 (our Balance type)
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

    // Block 1 - Transfers
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
    println!("Block 1 completed with {} transactions", block_1_result.transaction_count);

    // Block 2 - More transfers
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
    println!("Block 2 completed with {} transactions", block_2_result.transaction_count);

    // Block 3 - Include some failures
    let block_3_transactions = vec![
        Transaction::Transfer {
            from: cheryl.clone(),
            to: nathaniel.clone(),
            amount: 9200, // Should fail
        },
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
    println!("Block 3 completed with {} transactions", block_3_result.transaction_count);

    // Block 4 - Set up validators and staking
    println!("\n⚡ === STAKING SETUP ===");
    let block_4_transactions = vec![
        Transaction::AddValidator {
            validator: "cheryl".to_string(),
            commission: 5, // 5% commission
        },
        Transaction::AddValidator {
            validator: "nathaniel".to_string(),
            commission: 10, // 10% commission
        },
    ];
    let block_4_result = runtime.create_block(block_4_transactions);
    println!("Block 4 completed: Validators initialized");

    // Block 5 - Staking transactions
    let block_5_transactions = vec![
        Transaction::Stake {
            who: "femi".to_string(),
            amount: 200,
            validator: "cheryl".to_string(),
        },
        Transaction::Stake {
            who: "temi".to_string(),
            amount: 150,
            validator: "nathaniel".to_string(),
        },
    ];
    let block_5_result = runtime.create_block(block_5_transactions);
    println!("Block 5 completed: Staking initiated");

    // Advance several blocks to accumulate rewards
    for i in 6..=10 {
        runtime.create_block(vec![]);
        println!("Block {} created (empty block for rewards)", i);
    }

    // Block 11 - Claim rewards and unstake
    let block_11_transactions = vec![
        Transaction::ClaimRewards {
            who: "femi".to_string(),
        },
        Transaction::ClaimRewards {
            who: "temi".to_string(),
        },
        Transaction::Unstake {
            who: "femi".to_string(),
        },
    ];
    let block_11_result = runtime.create_block(block_11_transactions);
    println!("Block 11 completed: Rewards claimed and unstaking attempted");

    // Example using the support framework (like the main branch)
    println!("\n🔧 === USING SUPPORT FRAMEWORK ===");
    
    // Create a block using the support framework types
    let support_block = types::Block {
        header: support::Header { 
            block_number: runtime.system.block_number() + 1 
        },
        extrinsics: vec![
            support::Extrinsic {
                caller: cheryl.clone(),
                call: RuntimeCall::Balances(balances::Call::Transfer {
                    to: faith.clone(),
                    amount: 25,
                }),
            },
            support::Extrinsic {
                caller: "nathaniel".to_string(),
                call: RuntimeCall::Staking(staking::Call::ClaimRewards),
            },
        ],
    };

    // Execute the block
    runtime.execute_block(support_block).expect("Block execution failed");

    // Print final state
    runtime.print_blockchain_state();

    // Verify blockchain integrity
    runtime.verify_chain_integrity();

    // Demonstrate hash relationships
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

    println!("\n✨ === GENERIC BENEFITS DEMONSTRATED ===");
    println!("• Type safety: AccountId, Balance, BlockNumber are enforced at compile time");
    println!("• Flexibility: Easy to change u128 to u64 or String to u32 by updating types module");
    println!("• Reusability: Same pallet code works with different type configurations");
    println!("• Maintainability: Types are centralized in one place");
    println!("• Staking integration: Generic staking pallet works seamlessly with balances");
    
    println!("\n🎯 === STAKING FEATURES IMPLEMENTED ===");
    println!("• Generic validator management");
    println!("• Type-safe staking operations");
    println!("• Reward calculation and distribution");
    println!("• Unstaking with period requirements");
    println!("• Integration with balance transfers");
}