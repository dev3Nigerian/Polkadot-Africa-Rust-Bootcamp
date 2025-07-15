use std::collections::BTreeMap;

//Custom Result enum
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

//Staking Info for each account
#[derive(Debug, Clone)]
pub struct StakeInfo {
    pub staked_amount: u128,
    pub validator: String,
    pub stake_block: u32,
    pub last_reward_block: u32,
    pub total_rewards: u128,
}

// Validator Info
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub total_stake: u128,
    pub commission_rate: u8, // percentage (0-100)
    pub is_active: bool,
    pub nominators_count: u32,
    pub blocks_produced: u32,
}

// Staking events
#[derive(Debug, Clone)]
pub enum StakingEvent {
    Staked {
        who: String,
        amount: u128,
        validator: String,
    },
    Unstaked {
        who: String,
        amount: u128,
    },
    ValidatorAdded {
        validator: String,
    },
    ValidatorRemoved {
        validator: String,
    },
    RewardsPaid {
        who: String,
        amount: u128,
    },
    SlashApplied {
        who: String,
        amount: u128,
    },
}

#[derive(Debug)]
pub struct Pallet {
    pub stakes: BTreeMap<String, StakeInfo>,
    pub validators: BTreeMap<String, ValidatorInfo>,

    // config
    pub minimum_stake: u128,
    pub reward_rate: u128, //rewards per block per 1000 tokens
    pub unstaking_period: u32,
    pub max_validators: u32,

    // staking tracking
    pub total_staked: u128,
    pub current_block: u32,
    pub events: Vec<StakingEvent>,
}
impl Pallet {
    pub fn new() -> Self {
        Self {
            stakes: BTreeMap::new(),
            validators: BTreeMap::new(),
            minimum_stake: 100,
            reward_rate: 5, //0.5% per block
            unstaking_period: 10,
            max_validators: 10,
            total_staked: 0,
            current_block: 0,
            events: Vec::new(),
        }
    }

    pub fn new_with_config(
        minimum_stake: u128,
        reward_rate: u128,
        unstaking_period: u32,
        max_validators: u32,
    ) -> Self {
        Self {
            stakes: BTreeMap::new(),
            validators: BTreeMap::new(),
            minimum_stake,
            reward_rate,
            unstaking_period,
            max_validators,
            total_staked: 0,
            current_block: 0,
            events: Vec::new(),
        }
    }

    //Updates current block should be called by system Pallet
    pub fn on_block(&mut self, block_number: u32) {
        self.current_block = block_number;
        self.distribute_rewards()
    }

    pub fn add_validator(
        &mut self,
        validator: String,
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
            total_stake: 0,
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

    pub fn remove_validator(&mut self, validator: &String) -> Result<(), StakingError> {
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

    //stake tokens with validator
    pub fn stake(
        &mut self,
        who: String,
        amount: u128,
        validator: String,
        balance_check: impl Fn(&String) -> u128,
    ) -> std::result::Result<(), StakingError> {
        //check if already staked
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

        //check if user has enough balance
        if balance_check(&who) < amount {
            return Err(StakingError::InsufficientBalance);
        }

        //Create stake info
        let stake_info = StakeInfo {
            staked_amount: amount,
            validator: validator.clone(),
            stake_block: self.current_block,
            last_reward_block: self.current_block,
            total_rewards: 0,
        };

        //update validator info
        if let Some(validator_info) = self.validators.get_mut(&validator) {
            validator_info.total_stake += amount;
            validator_info.nominators_count += 1;
        }

        //Store stake info
        self.stakes.insert(who.clone(), stake_info);
        self.total_staked += amount;

        let event = StakingEvent::Staked {
            who,
            amount,
            validator,
        };
        self.events.push(event);

        Ok(())
    }

    pub fn unstake(&mut self, who: String) -> std::result::Result<u128, StakingError> {
        let stake_info = self.stakes.get(&who).ok_or(StakingError::NotStaked)?;

        if self.current_block < stake_info.stake_block + self.unstaking_period {
            return Err(StakingError::UnstakingPeriodNotMet);
        }

        let staked_amount = stake_info.staked_amount;
        let validator = stake_info.validator.clone();

        //Update validator info
        if let Some(validator_info) = self.validators.get_mut(&validator) {
            validator_info.total_stake -= staked_amount;
            validator_info.nominators_count -= 1
        }

        //Remove rust
        self.stakes.remove(&who);
        self.total_staked -= staked_amount;

        let event = StakingEvent::Unstaked {
            who,
            amount: staked_amount,
        };
        self.events.push(event);
        Ok(staked_amount)
    }

    /// Calculate rewards for a staker
    pub fn calculate_rewards(&self, who: &String) -> std::result::Result<u128, StakingError> {
        let stake_info = self.stakes.get(who).ok_or(StakingError::NotStaked)?;

        let blocks_since_last_reward = self.current_block - stake_info.last_reward_block;
        let base_reward =
            (stake_info.staked_amount * self.reward_rate * blocks_since_last_reward as u128) / 1000;

        //Apply Validator commission
        if let Some(validator_info) = self.validators.get(&stake_info.validator) {
            let commission = (base_reward * validator_info.commission_rate as u128) / 100;
            let net_reward = base_reward - commission;
            Ok(net_reward)
        } else {
            Err(StakingError::InvalidValidator)
        }
    }

    /// Claim Rewards
    pub fn claim_rewards(&mut self, who: String) -> std::result::Result<u128, StakingError> {
        let reward_amount = self.calculate_rewards(&who)?;

        if let Some(stake_info) = self.stakes.get_mut(&who) {
            stake_info.last_reward_block = self.current_block;
            stake_info.total_rewards += reward_amount;
        }

        let event = StakingEvent::RewardsPaid {
            who: who.to_string(),
            amount: reward_amount,
        };
        self.events.push(event);
        Ok(reward_amount)
    }

    /// Internal function to distribute rewards automatically
    fn distribute_rewards(&mut self) {
        let stakers: Vec<String> = self.stakes.keys().cloned().collect();

        for staker in stakers {
            if let Ok(reward) = self.claim_rewards(staker) {
                todo!()
            }
        }
    }

    /// Get staking info for an account
    pub fn get_stake_info(&self, who: &String) -> Option<&StakeInfo> {
        self.stakes.get(who)
    }

    /// Get Validator INfo
    pub fn get_validator_info(&self, validator: &String) -> Option<&ValidatorInfo> {
        self.validators.get(validator)
    }

    pub fn get_active_validators(&self) -> Vec<(&String, &ValidatorInfo)> {
        self.validators
            .iter()
            .filter(|(_, info)| info.is_active)
            .collect()
    }

    // Get total stake for all validators
    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }

    // Check if account is staking
    pub fn is_staking(&self, who: &String) -> bool {
        self.stakes.contains_key(who)
    }

    // Check if account is a validator
    pub fn is_validator(&self, who: &String) -> bool {
        self.validators.contains_key(who)
    }

    // Get staking events
    pub fn get_events(&self) -> &Vec<StakingEvent> {
        &self.events
    }

    // Clear events (should be called after each block)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn get_staking_stats(&self) -> StakingStats {
        let total_validators = self.validators.len() as u32;
        let active_validators = self.get_active_validators().len() as u32;
        let total_stakers = self.stakes.len() as u32;
        let average_stake = if total_stakers > 0 {
            self.total_staked / total_stakers as u128
        } else {
            0
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

// Staking statistics structure
#[derive(Debug)]
pub struct StakingStats {
    pub total_staked: u128,
    pub total_validators: u32,
    pub active_validators: u32,
    pub total_stakers: u32,
    pub average_stake: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_balance_check(balance: u128) -> impl Fn(&String) -> u128 {
        move |_| balance
    }

    #[test]
    fn test_validator_management() {
        let mut staking = Pallet::new();

        // Add Validator
        assert_eq!(
            staking.add_validator("alice".to_string(), 10),
            Result::Ok(())
        );
        assert!(staking.is_validator(&"alice".to_string()));

        // Try Adding same validator
        assert_eq!(
            staking.add_validator("alice".to_string(), 10),
            Result::Err(StakingError::AlreadyValidator)
        );

        assert_eq!(
            staking.add_validator(String::from("Bob"), 150),
            Result::Err(StakingError::InvalidValidator)
        );

        // Remove Validator
        assert_eq!(
            staking.remove_validator(&"alice".to_string()),
            Result::Ok(())
        );
        assert!(!staking.is_validator(&"alice".to_string()))
    }

    #[test]
    fn test_staking() {
        let mut staking = Pallet::new();

        // Add validator
        staking.add_validator("validator1".to_string(), 5).unwrap();

        //Stake tokens
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

        //Try staking again should fail
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

    #[test]
    fn test_staking_minimum() {
        let mut staking = Pallet::new();
        staking.add_validator("validator1".to_string(), 10).unwrap();

        // Try to stake less than minimum
        let balance_check = mock_balance_check(1000);
        assert_eq!(
            staking.stake(
                "user1".to_string(),
                80,
                "validator1".to_string(),
                balance_check
            ),
            Err(StakingError::MinimumStakeNotMet)
        );
    }

    #[test]
    fn test_staking_insufficient_balance() {
        let mut staking = Pallet::new();

        staking.add_validator("validator1".to_string(), 10).unwrap();
        let balance_check = mock_balance_check(100);
        assert_eq!(
            staking.stake(
                "user1".to_string(),
                300,
                "validator1".to_string(),
                balance_check
            ),
            Err(StakingError::InsufficientBalance)
        );
    }

    #[test]
    fn test_unstaking() {
        let mut staking = Pallet::new();

        staking.add_validator("validator1".to_string(), 10).unwrap();
        let balance_check = mock_balance_check(1000);
        staking
            .stake(
                "user1".to_string(),
                300,
                "validator1".to_string(),
                balance_check,
            )
            .unwrap();

        // Try to unstake immediately (should fail due to unstaking period)
        assert_eq!(
            staking.unstake("user1".to_string()),
            Err(StakingError::UnstakingPeriodNotMet)
        );
        //Advance blocks
        staking.on_block(15);

        //Unstaking should work
        assert_eq!(staking.unstake("user1".to_string()), Ok(200));
        assert!(!staking.is_staking(&"user1".to_string()));
        assert_eq!(staking.get_total_staked(), 0)
    }

    #[test]
    fn test_staking_stats() {
        let mut staking = Pallet::new();

        staking.add_validator("validator1".to_string(), 10).unwrap();
        staking.add_validator("validator2".to_string(), 5).unwrap();

        let balance_check = mock_balance_check(1000);
        staking
            .stake(
                "user1".to_string(),
                200,
                "validator1".to_string(),
                balance_check,
            )
            .unwrap();
        let balance_check = mock_balance_check(1000);
        staking
            .stake(
                "user2".to_string(),
                850,
                "validator1".to_string(),
                balance_check,
            )
            .unwrap();

        let stats = staking.get_staking_stats();
        assert_eq!(stats.total_staked, 1050);
        assert_eq!(stats.total_validators, 2);
        assert_eq!(stats.active_validators, 2);
        assert_eq!(stats.total_stakers, 2);
        assert_eq!(stats.average_stake, 525);
    }
}
