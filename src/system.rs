use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Pallet {
    pub block_number: u32,            // 2^32 = 4.5 million
    pub nonce: BTreeMap<String, u32>, // <username, nonce_value> e.g. ("femi", 10)
}

impl Pallet {
    /// Create an instance of the pallet
    pub fn new() -> Self {
        Self {
            block_number: 0,
            nonce: BTreeMap::new(),
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
}
