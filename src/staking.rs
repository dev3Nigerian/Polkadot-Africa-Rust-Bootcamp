use std::collections::BTreeMap;
use num::traits::{CheckedAdd, CheckedSub, Zero, One};

// Staking Config trait - extends the system Config with staking-specific types
pub trait Config: crate::system::Config {
    type Balance: CheckedAdd + CheckedSub + Zero + Copy + PartialOrd;  // Balance type with comparison
}

// Custom Result enum for staking operations
#[derive(Debug, PartialEq)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Result::Ok(_))
    }
    
    pub fn is_err(&self) -> bool {
        matches!(self, Result::Err(_))
    }
    
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        match self {
            Result::Ok(val) => val,
            Result::Err(err) => panic!("Called `Result::unwrap()` on an `Err` value: {:?}", err),
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => default,
        }
    }
}

// Staking-specific error types
#[derive(Debug, PartialEq, Clone)]
pub enum StakingError {
    InsufficientBalance,
    NotStaked,
    AlreadyStaked,
    MinimumStakeNotMet,
    InvalidValidator,
    TooManyValidators,
    NotValidator,
    AlreadyValidator,
    RewardCalculationError,
    UnstakingPeriodNotMet,
}

impl std::fmt::Display for StakingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StakingError::InsufficientBalance => write!(f, "Insufficient balance to stake"),
            StakingError::NotStaked => write!(f, "Account is not staking"),
            StakingError::AlreadyStaked => write!(f, "Account is already staking"),
            StakingError::MinimumStakeNotMet => write!(f, "Minimum stake amount not met"),
            StakingError::InvalidValidator => write!(f, "Invalid validator"),
            StakingError::TooManyValidators => write!(f, "Too many validators"),
            StakingError::NotValidator => write!(f, "Account is not a validator"),
            StakingError::AlreadyValidator => write!(f, "Account is already a validator"),
            StakingError::RewardCalculationError => write!(f, "Error calculating rewards"),
            StakingError::UnstakingPeriodNotMet => write!(f, "Unstaking period not met"),
        }
    }
}

// Staking Info for each account - now generic over Config types
#[derive(Debug, Clone)]
pub struct StakeInfo<T: Config> {
    pub staked_amount: T::Balance,
    pub validator: T::AccountId,
    pub stake_block: T::BlockNumber,
    pub last_reward_block: T::BlockNumber,
    pub total_rewards: T::Balance,
}

// Validator Info - generic over Config types
#[derive(Debug, Clone)]
pub struct ValidatorInfo<T: Config> {
    pub total_stake: T::Balance,
    pub commission_rate: u8, // percentage (0-100)
    pub is_active: bool,
    pub nominators_count: u32,
    pub blocks_produced: u32,
}

// Staking events - generic over Config types
#[derive(Debug, Clone)]
pub enum StakingEvent<T: Config> {
    Staked {
        who: T::AccountId,
        amount: T::Balance,
        validator: T::AccountId,
    },
    Unstaked {
        who: T::AccountId,
        amount: T::Balance,
    },
    ValidatorAdded {
        validator: T::AccountId,
    },
    ValidatorRemoved {
        validator: T::AccountId,
    },
    RewardsPaid {
        who: T::AccountId,
        amount: T::Balance,
    },
    SlashApplied {
        who: T::AccountId,
        amount: T::Balance,
    },
}

// Generic Staking Pallet
#[derive(Debug)]
pub struct Pallet<T: Config> {
    pub stakes: BTreeMap<T::AccountId, StakeInfo<T>>,
    pub validators: BTreeMap<T::AccountId, ValidatorInfo<T>>,

    // Configuration - using generic types
    pub minimum_stake: T::Balance,
    pub reward_rate: T::Balance, // rewards per block per 1000 tokens
    pub unstaking_period: T::BlockNumber,
    pub max_validators: u32,

    // Staking tracking
    pub total_staked: T::Balance,
    pub current_block: T::BlockNumber,
    pub events: Vec<StakingEvent<T>>,
}

impl<T: Config> Pallet<T> {
    pub fn new() -> Self {
        Self {
            stakes: BTreeMap::new(),
            validators: BTreeMap::new(),
            minimum_stake: T::Balance::zero(), // Will need to be set properly
            reward_rate: T::Balance::zero(),   // Will need to be set properly
            unstaking_period: T::BlockNumber::zero(),
            max_validators: 10,
            total_staked: T::Balance::zero(),
            current_block: T::BlockNumber::zero(),
            events: Vec::new(),
        }
    }

    pub fn new_with_config(
        minimum_stake: T::Balance,
        reward_rate: T::Balance,
        unstaking_period: T::BlockNumber,
        max_validators: u32,
    ) -> Self {
        Self {
            stakes: BTreeMap::new(),
            validators: BTreeMap::new(),
            minimum_stake,
            reward_rate,
            unstaking_period,
            max_validators,
            total_staked: T::Balance::zero(),
            current_block: T::BlockNumber::zero(),
            events: Vec::new(),
        }
    }

    // Updates current block - should be called by system pallet
    pub fn on_block(&mut self, block_number: T::BlockNumber) {
        self.current_block = block_number;
        self.distribute_rewards();
    }

    pub fn add_validator(
        &mut self,
        validator: T::AccountId,
        commission_rate: u8,
    ) -> Result<(), StakingError> {
        if self.validators.contains_key(&validator) {
            return Result::Err(StakingError::AlreadyValidator);
        }
        if self.validators.len() >= self.max_validators as usize {
            return Result::Err(StakingError::TooManyValidators);
        }
        if commission_rate > 100 {
            return Result::Err(StakingError::InvalidValidator);
        }

        let validator_info = ValidatorInfo {
            total_stake: T::Balance::zero(),
            commission_rate,
            is_active: true,
            nominators_count: 0,
            blocks_produced: 0,
        };

        self.validators.insert(validator.clone(), validator_info);
        
        let event = StakingEvent::ValidatorAdded {
            validator: validator,
        };
        self.events.push(event);
        
        Result::Ok(())
    }

    pub fn remove_validator(&mut self, validator: &T::AccountId) -> Result<(), StakingError> {
        if !self.validators.contains_key(validator) {
            return Result::Err(StakingError::NotValidator);
        }
        
        self.validators.remove(validator);
        
        let event = StakingEvent::ValidatorRemoved {
            validator: validator.clone(),
        };
        self.events.push(event);
        
        Result::Ok(())
    }

    // Stake tokens with validator - using a closure for balance checking
    pub fn stake(
        &mut self,
        who: T::AccountId,
        amount: T::Balance,
        validator: T::AccountId,
        balance_check: impl Fn(&T::AccountId) -> T::Balance,
    ) -> std::result::Result<(), StakingError> {
        // Check if already staked
        if self.stakes.contains_key(&who) {
            return Err(StakingError::AlreadyStaked);
        }

        if amount < self.minimum_stake {
            return Err(StakingError::MinimumStakeNotMet);
        }

        let validator_info = self
            .validators
            .get(&validator)
            .ok_or(StakingError::InvalidValidator)?;

        if !validator_info.is_active {
            return Err(StakingError::InvalidValidator);
        }

        // Check if user has enough balance
        if balance_check(&who) < amount {
            return Err(StakingError::InsufficientBalance);
        }

        // Create stake info
        let stake_info = StakeInfo {
            staked_amount: amount,
            validator: validator.clone(),
            stake_block: self.current_block,
            last_reward_block: self.current_block,
            total_rewards: T::Balance::zero(),
        };

        // Update validator info
        if let Some(validator_info) = self.validators.get_mut(&validator) {
            validator_info.total_stake = validator_info.total_stake
                .checked_add(&amount)
                .ok_or(StakingError::RewardCalculationError)?;
            validator_info.nominators_count += 1;
        }

        // Store stake info
        self.stakes.insert(who.clone(), stake_info);
        self.total_staked = self.total_staked
            .checked_add(&amount)
            .ok_or(StakingError::RewardCalculationError)?;

        let event = StakingEvent::Staked {
            who,
            amount,
            validator,
        };
        self.events.push(event);

        Ok(())
    }

    pub fn unstake(&mut self, who: T::AccountId) -> std::result::Result<T::Balance, StakingError> {
        let stake_info = self.stakes.get(&who).ok_or(StakingError::NotStaked)?;

        // Check unstaking period (simplified comparison)
        let stake_block_plus_period = stake_info.stake_block; // Simplified for now
        if self.current_block < stake_block_plus_period {
            return Err(StakingError::UnstakingPeriodNotMet);
        }

        let staked_amount = stake_info.staked_amount;
        let validator = stake_info.validator.clone();

        // Update validator info
        if let Some(validator_info) = self.validators.get_mut(&validator) {
            validator_info.total_stake = validator_info.total_stake
                .checked_sub(&staked_amount)
                .ok_or(StakingError::RewardCalculationError)?;
            validator_info.nominators_count -= 1;
        }

        // Remove stake
        self.stakes.remove(&who);
        self.total_staked = self.total_staked
            .checked_sub(&staked_amount)
            .ok_or(StakingError::RewardCalculationError)?;

        let event = StakingEvent::Unstaked {
            who,
            amount: staked_amount,
        };
        self.events.push(event);
        
        Ok(staked_amount)
    }

    /// Calculate rewards for a staker
    pub fn calculate_rewards(&self, who: &T::AccountId) -> std::result::Result<T::Balance, StakingError> {
        let stake_info = self.stakes.get(who).ok_or(StakingError::NotStaked)?;

        // Simplified reward calculation
        let base_reward = self.reward_rate; // Simplified for now

        // Apply validator commission
        if let Some(validator_info) = self.validators.get(&stake_info.validator) {
            // Simplified commission calculation
            let net_reward = base_reward; // Simplified for now
            Ok(net_reward)
        } else {
            Err(StakingError::InvalidValidator)
        }
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, who: T::AccountId) -> std::result::Result<T::Balance, StakingError> {
        let reward_amount = self.calculate_rewards(&who)?;

        if let Some(stake_info) = self.stakes.get_mut(&who) {
            stake_info.last_reward_block = self.current_block;
            stake_info.total_rewards = stake_info.total_rewards
                .checked_add(&reward_amount)
                .ok_or(StakingError::RewardCalculationError)?;
        }

        let event = StakingEvent::RewardsPaid {
            who: who.clone(),
            amount: reward_amount,
        };
        self.events.push(event);
        
        Ok(reward_amount)
    }

    /// Internal function to distribute rewards automatically
    fn distribute_rewards(&mut self) {
        let stakers: Vec<T::AccountId> = self.stakes.keys().cloned().collect();

        for staker in stakers {
            if let Ok(_reward) = self.claim_rewards(staker) {
                // Rewards distributed
            }
        }
    }

    /// Get staking info for an account
    pub fn get_stake_info(&self, who: &T::AccountId) -> Option<&StakeInfo<T>> {
        self.stakes.get(who)
    }

    /// Get validator info
    pub fn get_validator_info(&self, validator: &T::AccountId) -> Option<&ValidatorInfo<T>> {
        self.validators.get(validator)
    }

    pub fn get_active_validators(&self) -> Vec<(&T::AccountId, &ValidatorInfo<T>)> {
        self.validators
            .iter()
            .filter(|(_, info)| info.is_active)
            .collect()
    }

    // Get total stake for all validators
    pub fn get_total_staked(&self) -> T::Balance {
        self.total_staked
    }

    // Check if account is staking
    pub fn is_staking(&self, who: &T::AccountId) -> bool {
        self.stakes.contains_key(who)
    }

    // Check if account is a validator
    pub fn is_validator(&self, who: &T::AccountId) -> bool {
        self.validators.contains_key(who)
    }

    // Get staking events
    pub fn get_events(&self) -> &Vec<StakingEvent<T>> {
        &self.events
    }

    // Clear events (should be called after each block)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn get_staking_stats(&self) -> StakingStats<T> {
        let total_validators = self.validators.len() as u32;
        let active_validators = self.get_active_validators().len() as u32;
        let total_stakers = self.stakes.len() as u32;
        let average_stake = if total_stakers > 0 {
            // Simplified average calculation
            self.total_staked
        } else {
            T::Balance::zero()
        };

        StakingStats {
            total_staked: self.total_staked,
            total_validators,
            active_validators,
            total_stakers,
            average_stake,
        }
    }
}

// Staking statistics structure - now generic
#[derive(Debug)]
pub struct StakingStats<T: Config> {
    pub total_staked: T::Balance,
    pub total_validators: u32,
    pub active_validators: u32,
    pub total_stakers: u32,
    pub average_stake: T::Balance,
}

// Staking calls enum
pub enum Call<T: Config> {
    AddValidator {
        validator: T::AccountId,
        commission: u8,
    },
    Stake {
        validator: T::AccountId,
        amount: T::Balance,
    },
    Unstake,
    ClaimRewards,
}

// Implement dispatch for the staking pallet
impl<T: Config> crate::support::Dispatch for Pallet<T> {
    type Call = Call<T>;
    type Caller = T::AccountId;

    fn dispatch(
        &mut self,
        caller: Self::Caller,
        call: Self::Call,
    ) -> crate::support::DispatchResult {
        match call {
            Call::AddValidator { validator, commission } => {
                self.add_validator(validator, commission)
                    .map_err(|_| "Failed to add validator")?;
            }
            Call::Stake { validator, amount } => {
                // This would need access to balance pallet for balance checking
                // For now, we'll return an error
                return Err("Staking through dispatch not implemented yet");
            }
            Call::Unstake => {
                self.unstake(caller)
                    .map_err(|_| "Failed to unstake")?;
            }
            Call::ClaimRewards => {
                self.claim_rewards(caller)
                    .map_err(|_| "Failed to claim rewards")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test configuration
    struct TestConfig;

    impl crate::system::Config for TestConfig {
        type AccountId = String;
        type BlockNumber = u32;
        type Nonce = u32;
    }

    impl Config for TestConfig {
        type Balance = u128;
    }

    fn mock_balance_check(balance: u128) -> impl Fn(&String) -> u128 {
        move |_| balance
    }

    #[test]
    fn test_validator_management() {
        let mut staking = Pallet::<TestConfig>::new();

        // Add validator
        assert_eq!(
            staking.add_validator("alice".to_string(), 10),
            Result::Ok(())
        );
        assert!(staking.is_validator(&"alice".to_string()));

        // Try adding same validator
        assert_eq!(
            staking.add_validator("alice".to_string(), 10),
            Result::Err(StakingError::AlreadyValidator)
        );

        // Invalid commission rate
        assert_eq!(
            staking.add_validator("bob".to_string(), 150),
            Result::Err(StakingError::InvalidValidator)
        );

        // Remove validator
        assert_eq!(
            staking.remove_validator(&"alice".to_string()),
            Result::Ok(())
        );
        assert!(!staking.is_validator(&"alice".to_string()));
    }

    #[test]
    fn test_staking() {
        let mut staking = Pallet::<TestConfig>::new_with_config(100, 5, 10, 10);

        // Add validator
        staking.add_validator("validator1".to_string(), 5).unwrap();

        // Stake tokens
        let balance_check = mock_balance_check(1000);
        assert_eq!(
            staking.stake(
                "user1".to_string(),
                200,
                "validator1".to_string(),
                balance_check
            ),
            Ok(())
        );

        assert!(staking.is_staking(&"user1".to_string()));
        assert_eq!(staking.get_total_staked(), 200);

        // Try staking again should fail
        let balance_check = mock_balance_check(1000);
        assert_eq!(
            staking.stake(
                "user1".to_string(),
                100,
                "validator1".to_string(),
                balance_check
            ),
            Err(StakingError::AlreadyStaked)
        );
    }
}