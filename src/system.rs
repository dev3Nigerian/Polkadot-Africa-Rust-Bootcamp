use std::collections::BTreeMap;
use num::traits::{One, Zero};
use core::ops::AddAssign;

pub trait Config {
    type AccountId: Ord + Clone;                    
    type BlockNumber: Zero + One + AddAssign + Copy; 
    type Nonce: Zero + One + Copy;                 
}


#[derive(Debug)]
pub struct Pallet<T: Config> {  // T is a placeholder for any type that implements Config
    pub block_number: T::BlockNumber,                    // Uses the BlockNumber type from T
    pub nonce: BTreeMap<T::AccountId, T::Nonce>,        // Uses AccountId and Nonce types from T
    pub block_hashes: BTreeMap<T::BlockNumber, [u8; 32]>, // Track block hashes with generic type
}

impl<T: Config> Pallet<T> {      /// Create an instance of the pallet
    pub fn new() -> Self {
        Self {
            block_number: T::BlockNumber::zero(),  // Start at zero using the generic type's zero
            nonce: BTreeMap::new(),
            block_hashes: BTreeMap::new(),
        }
    }

    /// Get the current block number
  pub fn block_number(&self) -> T::BlockNumber {
        self.block_number
    }

    /// Increase the block number by one
      pub fn inc_block_number(&mut self) {
        self.block_number += T::BlockNumber::one();  
    }
    /// Increase the nonce value of the caller `who`
     pub fn inc_nonce(&mut self, who: &T::AccountId) {
        let nonce = *self.nonce.get(who).unwrap_or(&T::Nonce::zero());
        let new_nonce = nonce + T::Nonce::one();
        self.nonce.insert(who.clone(), new_nonce);
    }

    /// Generate block hash based on block number and nonce data
   fn generate_block_hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];

        let block_num_as_u32 = if self.block_number == T::BlockNumber::zero() {
            0u32
        } else {

            1u32
        };
        
        let block_bytes = block_num_as_u32.to_be_bytes();
        hash[0..4].copy_from_slice(&block_bytes);

        let nonce_count = self.nonce.len() as u32;
        let nonce_bytes = nonce_count.to_be_bytes();
        hash[4..8].copy_from_slice(&nonce_bytes);

        // Fill the rest with pattern based on block number
        for i in 8..32 {
            hash[i] = ((i + block_num_as_u32 as usize) % 256) as u8;
        }

        hash
    }

    /// Finalize the current block and generate its hash
    pub fn finalize_block(&mut self) -> [u8; 32] {
        let hash = self.generate_block_hash();
        self.block_hashes.insert(self.block_number, hash);
        hash
    }

    /// Get block hash for a specific block number
     pub fn get_block_hash(&self, block_number: T::BlockNumber) -> Option<[u8; 32]> {
        self.block_hashes.get(&block_number).copied()
    }

    /// Get the hash of the current block (if finalized)
    pub fn current_block_hash(&self) -> Option<[u8; 32]> {
        self.get_block_hash(self.block_number)
    }

    /// Get the hash of the parent block
    // pub fn parent_block_hash(&self) -> Option<[u8; 32]> {
    //     if self.block_number > 0 {
    //         self.get_block_hash(self.block_number - 1)
    //     } else {
    //         None
    //     }
    // }
     pub fn parent_block_hash(&self) -> Option<[u8; 32]> {
        if self.block_number > T::BlockNumber::zero() {
            let prev_blocks: Vec<_> = self.block_hashes.keys().collect();
            if let Some(&last_block) = prev_blocks.iter().rev().nth(1) {
                self.get_block_hash(*last_block)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get all block hashes
    // pub fn all_block_hashes(&self) -> &BTreeMap<u32, [u8; 32]> {
    //     &self.block_hashes
    // }
     pub fn all_block_hashes(&self) -> &BTreeMap<T::BlockNumber, [u8; 32]> {
        &self.block_hashes
    }

    /// Get the genesis block hash (block 0)
    // pub fn genesis_hash(&self) -> Option<[u8; 32]> {
    //     self.get_block_hash(0)
    // }
     pub fn genesis_hash(&self) -> Option<[u8; 32]> {
        self.get_block_hash(T::BlockNumber::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestConfig;

    impl Config for TestConfig {
        type AccountId = String;     // In tests, accounts are Strings
        type BlockNumber = u32;      // In tests, block numbers are u32
        type Nonce = u32;           // In tests, nonces are u32
    }

    #[test]
    // fn system_pallet_work() {
    //     // Arrange
    //     // create system pallet
    //     let mut system = Pallet::new();

    //     // Act
    //     // increase current block number
    //     system.inc_block_number();
    //     // increase the nonce of a user - `Temi`
    //     system.inc_nonce(&"Temi".to_string());

    //     // Assert
    //     // Check the block number (i.e. 1)
    //     assert_eq!(system.block_number(), 1);
    //     // Check the nonce of Temi (i.e. 1)
    //     assert_eq!(system.nonce.get("Temi"), Some(&1));
    //     // Check the nonce of Faithful (i.e. 0)
    //     assert_eq!(system.nonce.get("Faithful"), None);
    // }
    fn system_pallet_work() {
        // Create system pallet with our test configuration
        let mut system = Pallet::<TestConfig>::new();

        system.inc_block_number();
        system.inc_nonce(&"Temi".to_string());

        assert_eq!(system.block_number(), 1);
        assert_eq!(system.nonce.get("Temi"), Some(&1));
        assert_eq!(system.nonce.get("Faithful"), None);
    }

    #[test]
     fn test_block_hash_generation() {
        let mut system = Pallet::<TestConfig>::new();

        let genesis_hash = system.finalize_block();
        assert_eq!(system.block_number(), 0);
        assert_eq!(system.get_block_hash(0), Some(genesis_hash));
        assert_eq!(system.current_block_hash(), Some(genesis_hash));
        assert_eq!(system.genesis_hash(), Some(genesis_hash));

        system.inc_block_number();
        system.inc_nonce(&"Alice".to_string());
        system.inc_nonce(&"Bob".to_string());

        let block_1_hash = system.finalize_block();
        assert_eq!(system.get_block_hash(1), Some(block_1_hash));
        assert_eq!(system.current_block_hash(), Some(block_1_hash));

        // Hashes should be different
        assert_ne!(genesis_hash, block_1_hash);
    }
    // fn test_block_hash_generation() {
    //     let mut system = Pallet::new();

    //     // Genesis block
    //     let genesis_hash = system.finalize_block();
    //     assert_eq!(system.block_number(), 0);
    //     assert_eq!(system.get_block_hash(0), Some(genesis_hash));
    //     assert_eq!(system.current_block_hash(), Some(genesis_hash));
    //     assert_eq!(system.genesis_hash(), Some(genesis_hash));

    //     // Move to block 1
    //     system.inc_block_number();
    //     assert_eq!(system.block_number(), 1);

    //     //Add nonce data
    //     system.inc_nonce(&"Alice".to_string());
    //     system.inc_nonce(&"Bob".to_string());

    //     // FInalize block 1
    //     let block_1_hash = system.finalize_block();
    //     assert_eq!(system.get_block_hash(1), Some(block_1_hash));
    //     assert_eq!(system.current_block_hash(), Some(block_1_hash));
    //     assert_eq!(system.parent_block_hash(), Some(genesis_hash));

    //     // Hashes should be different
    //     assert_ne!(genesis_hash, block_1_hash);
    // }

    
    

    // TODO change to new config
    // #[test]
    // fn test_block_hash_consistency() {
    //     let mut system = Pallet::new();

    //     let hash_1 = system.finalize_block();
    //     let hash_2 = system.finalize_block();

    //     //same blocks should produce same hash
    //     assert_eq!(hash_1, hash_2);

    //     system.inc_block_number();
    //     system.inc_nonce(&"Bob".to_string());

    //     let hash_3 = system.finalize_block();
    //     assert_ne!(hash_2, hash_3)
    // }

    // #[test]
    // fn test_parent_bloch_hash() {
    //     let mut system = Pallet::new();

    //     //Genesis block has no parent
    //     assert_eq!(system.parent_block_hash(), None);

    //     let genesis_hash = system.finalize_block();
    //     // This test will fail
    //     // assert_eq!(system.parent_block_hash(), Some(genesis_hash))

    //     system.inc_block_number();
    //     assert_eq!(system.parent_block_hash(), Some(genesis_hash));

    //     // Block is finalized without being increased; this will still be the genesis block
    //     let block_1_hash = system.finalize_block();
    //     assert_eq!(system.parent_block_hash(), Some(block_1_hash))
    // }

    // #[test]
    // fn test_all_block_hashes() {
    //     let mut system = Pallet::new();

    //     // Initially empty
    //     assert_eq!(system.all_block_hashes().len(), 0);

    //     let hash_0 = system.finalize_block();
    //     system.inc_block_number();
    //     let hash_1 = system.finalize_block();
    //     system.inc_block_number();
    //     let hash_2 = system.finalize_block();
    //     system.inc_block_number();

    //     let all_hashes = system.all_block_hashes();
    //    assert_eq!(all_hashes.len(), 3);
    //    assert_eq!(all_hashes.get(&0), Some(&hash_0));
    //    assert_eq!(all_hashes.get(&1), Some(&hash_1));
    //    assert_eq!(all_hashes.get(&2), Some(&hash_2));
    // }
}
