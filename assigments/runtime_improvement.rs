use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Pallet {
    pub block_number: u32,            // 2^32 = 4.5 million
    pub nonce: BTreeMap<String, u32>, // <username, nonce_value> e.g. ("femi", 10)

    /**
     * ASSIGNMENT TO IMPROVE
     */
    pub block_hashes: BTreeMap<u32, [u8; 32]>, //Track block hashes
}

impl Pallet {
    /// Create an instance of the pallet
    pub fn new() -> Self {
        Self {
            block_number: 0,
            nonce: BTreeMap::new(),
            block_hashes: BTreeMap::new(),
        }
    }

    /// Get the current block number
    pub fn block_number(&self) -> u32 {
        self.block_number
    }

    /// Increase the block number by one
    pub fn inc_block_number(&mut self) {
        self.block_number += 1;
    }

    /// Increase the nonce value of the caller `who`
    pub fn inc_nonce(&mut self, who: &String) {
        // Check for the nonce of `who`, and store. If it does not exist, set nonce to `0`
        // create new nonce => nonce + 1
        // store new nonce, with caller
        let nonce = self.nonce.get(who).unwrap_or(&0);
        let new_nonce = nonce + 1;
        self.nonce.insert(who.clone(), new_nonce);
    }

    /// Generate block hash based on block number and nonce data
    fn generate_block_hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];

        let block_bytes = self.block_number.to_be_bytes();
        hash[0..4].copy_from_slice(&block_bytes);

        // Include nonce data in hash
        let nonce_sum: u32 = self.nonce.values().sum();
        let nonce_bytes = nonce_sum.to_be_bytes();
        hash[4..8].copy_from_slice(&nonce_bytes);

        if let Some(parent_hash) = self.get_block_hash(self.block_number.saturating_sub(1)) {
            hash[8..16].copy_from_slice(&parent_hash[0..8]);
        }

        for i in 16..32 {
            hash[i] = ((i + self.block_number as usize) % 256) as u8
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
    pub fn get_block_hash(&self, block_number: u32) -> Option<[u8; 32]> {
        self.block_hashes.get(&block_number).copied()
    }

    /// Get the hash of the current block (if finalized)
    pub fn current_block_hash(&self) -> Option<[u8; 32]> {
        self.get_block_hash(self.block_number)
    }

    /// Get the hash of the parent block
    pub fn parent_block_hash(&self) -> Option<[u8; 32]> {
        if self.block_number > 0 {
            self.get_block_hash(self.block_number - 1)
        } else {
            None
        }
    }

    /// Get all block hashes
    pub fn all_block_hashes(&self) -> &BTreeMap<u32, [u8; 32]> {
        &self.block_hashes
    }

    /// Get the genesis block hash (block 0)
    pub fn genesis_hash(&self) -> Option<[u8; 32]> {
        self.get_block_hash(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_pallet_work() {
        // Arrange
        // create system pallet
        let mut system = Pallet::new();

        // Act
        // increase current block number
        system.inc_block_number();
        // increase the nonce of a user - `Temi`
        system.inc_nonce(&"Temi".to_string());

        // Assert
        // Check the block number (i.e. 1)
        assert_eq!(system.block_number(), 1);
        // Check the nonce of Temi (i.e. 1)
        assert_eq!(system.nonce.get("Temi"), Some(&1));
        // Check the nonce of Faithful (i.e. 0)
        assert_eq!(system.nonce.get("Faithful"), None);
    }

    #[test]
    fn test_block_hash_generation() {
        let mut system = Pallet::new();

        // Genesis block
        let genesis_hash = system.finalize_block();
        assert_eq!(system.block_number(), 0);
        assert_eq!(system.get_block_hash(0), Some(genesis_hash));
        assert_eq!(system.current_block_hash(), Some(genesis_hash));
        assert_eq!(system.genesis_hash(), Some(genesis_hash));

        // Move to block 1
        system.inc_block_number();
        assert_eq!(system.block_number(), 1);

        //Add nonce data
        system.inc_nonce(&"Alice".to_string());
        system.inc_nonce(&"Bob".to_string());

        // FInalize block 1
        let block_1_hash = system.finalize_block();
        assert_eq!(system.get_block_hash(1), Some(block_1_hash));
        assert_eq!(system.current_block_hash(), Some(block_1_hash));
        assert_eq!(system.parent_block_hash(), Some(genesis_hash));

        // Hashes should be different
        assert_ne!(genesis_hash, block_1_hash);
    }

    #[test]
    fn test_block_hash_consistency() {
        let mut system = Pallet::new();

        let hash_1 = system.finalize_block();
        let hash_2 = system.finalize_block();

        //same blocks should produce same hash
        assert_eq!(hash_1, hash_2);

        system.inc_block_number();
        system.inc_nonce(&"Bob".to_string());

        let hash_3 = system.finalize_block();
        assert_ne!(hash_2, hash_3)
    }

    #[test]
    fn test_parent_bloch_hash() {
        let mut system = Pallet::new();

        //Genesis block has no parent
        assert_eq!(system.parent_block_hash(), None);

        let genesis_hash = system.finalize_block();
        // This test will fail
        // assert_eq!(system.parent_block_hash(), Some(genesis_hash))

        system.inc_block_number();
        assert_eq!(system.parent_block_hash(), Some(genesis_hash));

        // Block is finalized without being increased; this will still be the genesis block
        let block_1_hash = system.finalize_block();
        assert_eq!(system.parent_block_hash(), Some(block_1_hash))
    }

    #[test]
    fn test_all_block_hashes() {
        let mut system = Pallet::new();

        // Initially empty
        assert_eq!(system.all_block_hashes().len(), 0);

        let hash_0 = system.finalize_block();
        system.inc_block_number();
        let hash_1 = system.finalize_block();
        system.inc_block_number();
        let hash_2 = system.finalize_block();
        system.inc_block_number();

        let all_hashes = system.all_block_hashes();
        assert_eq!(all_hashes.len(), 3);
        assert_eq!(all_hashes.get(&0), Some(&hash_0));
        assert_eq!(all_hashes.get(&1), Some(&hash_1));
        assert_eq!(all_hashes.get(&2), Some(&hash_2));
    }
}
