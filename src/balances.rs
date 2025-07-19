
use std::collections::BTreeMap;
use num::traits::{CheckedSub, CheckedAdd, Zero};

// pub struct Pallet {
//     balances: BTreeMap<String, u128>,
//     base_fee: u128,
//     fee_recipient: Option<String>,
// }
pub trait Config: crate::system::Config {
    type Balance: CheckedAdd + CheckedSub + Zero + Copy;  // Balance must support safe math
}

// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

/// Enum and impl to handle  Errors
#[derive(Debug, PartialEq, Clone)]
pub enum BalancesError {
    InsufficientBalance,
    InsufficientFunds,
    OverflowInCalculation,
    OverflowInTransfer,
    InvalidAmount,
}

impl std::fmt::Display for BalancesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalancesError::InsufficientBalance => write!(f, "Insufficient balance"),
            BalancesError::InsufficientFunds => write!(f, "Insufficient funds to pay fees"),
            BalancesError::OverflowInCalculation => {
                write!(f, "Overflow in clculating transfer costs")
            }
            BalancesError::OverflowInTransfer => write!(f, "Overflow in transfer calculation"),
            BalancesError::InvalidAmount => write!(f, "Invalid amount specified"),
        }
    }
}
#[derive(Debug)]
pub struct Pallet<T: Config> {  
    balances: BTreeMap<T::AccountId, T::Balance>,  
    base_fee: T::Balance,                         
    fee_recipient: Option<T::AccountId>,          
}

impl<T: Config> Pallet<T> { 
    // pub fn new() -> Self {
    //     Self {
    //         balances: BTreeMap::new(),
    //         base_fee: 10,
    //         fee_recipient: None,
    //     }
    // }
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
            base_fee: T::Balance::zero(),  // Start with zero fee using generic type
            fee_recipient: None,
        }
    }

    // pub fn new_with_fee_config(base_fee: u128, fee_recipient: Option<String>) -> Self {
    //     Self {
    //         balances: BTreeMap::new(),
    //         base_fee,
    //         fee_recipient,
    //     }
    // }
     pub fn new_with_fee_config(base_fee: T::Balance, fee_recipient: Option<T::AccountId>) -> Self {
        Self {
            balances: BTreeMap::new(),
            base_fee,
            fee_recipient,
        }
    }

    // pub fn set_transaction_fee(&mut self, fee: u128) {
    //     self.base_fee = fee;
    // }
     pub fn set_transaction_fee(&mut self, fee: T::Balance) {
        self.base_fee = fee;
    }

    // pub fn get_transaction_fee(&self) -> u128 {
    //     self.base_fee
    // }
      pub fn get_transaction_fee(&self) -> T::Balance {
        self.base_fee
    }

    // pub fn set_fee_recipient(&mut self, recipient: Option<String>) {
    //     self.fee_recipient = recipient;
    // }
     pub fn set_fee_recipient(&mut self, recipient: Option<T::AccountId>) {
        self.fee_recipient = recipient;
    }

    // fn calculate_fee(&self, _amount: u128) -> u128 {
    //     if _amount > 100 {
    //         _amount / 10
    //     } else {
    //         self.base_fee
    //     }
    // }
    fn calculate_fee(&self, amount: T::Balance) -> T::Balance {
        self.base_fee
    }

    // fn handle_fee_payment(&mut self, who: &String, fee: u128) -> Result<(), BalancesError> {
    //     let payer_balance = self.balance(who);
    //     if payer_balance < fee {
    //         return Result::Err(BalancesError::InsufficientFunds);
    //     }

    //     //Deduct fee from Payer
    //     self.balances.insert(who.clone(), payer_balance - fee);

    //     match &self.fee_recipient {
    //         Some(recipient) => {
    //             let recipient_balance = self.balance(recipient);
    //             self.balances
    //                 .insert(recipient.clone(), recipient_balance + fee);
    //         }
    //         None => {}
    //     }
    //     Ok(())
    // }
      fn handle_fee_payment(&mut self, who: &T::AccountId, fee: T::Balance) -> Result<(), BalancesError> {
        let payer_balance = self.balance(who);
        
        // Check if payer has enough balance for fee
        let new_balance = payer_balance
            .checked_sub(&fee)
            .ok_or(BalancesError::InsufficientFunds)?;

        // Deduct fee from payer
        self.balances.insert(who.clone(), new_balance);

        // Add fee to recipient if one is set
        if let Some(ref recipient) = self.fee_recipient {
            let recipient_balance = self.balance(recipient);
            let new_recipient_balance = recipient_balance
                .checked_add(&fee)
                .ok_or(BalancesError::OverflowInCalculation)?;
            self.balances.insert(recipient.clone(), new_recipient_balance);
        }
        
        Ok(())
    }

    // pub fn set_balance(&mut self, who: &String, amount: u128) {
    //     self.balances.insert(who.clone(), amount);
    // }
     pub fn set_balance(&mut self, who: &T::AccountId, amount: T::Balance) {
        self.balances.insert(who.clone(), amount);
    }

    // pub fn balance(&self, who: &String) -> u128 {
    //     *self.balances.get(who).unwrap_or(&0)
    // }
     pub fn balance(&self, who: &T::AccountId) -> T::Balance {
        *self.balances.get(who).unwrap_or(&T::Balance::zero())
    }

    //Implemented the Balances Error here
    // pub fn get_transfer_cost(&self, amount: u128) -> Result<u128, BalancesError> {
    //     let fee = self.calculate_fee(amount);
    //     amount.checked_add(fee).map_or_else(
    //         || Err(BalancesError::OverflowInCalculation),
    //         |total_cost| Ok(total_cost),
    //     )
    // }
      pub fn get_transfer_cost(&self, amount: T::Balance) -> Result<T::Balance, BalancesError> {
        let fee = self.calculate_fee(amount);
        amount.checked_add(&fee)
            .ok_or(BalancesError::OverflowInCalculation)
    }

    // pub fn transfer(
    //     &mut self,
    //     sender: String,
    //     receiver: String,
    //     amount: u128,
    // ) -> Result<(), BalancesError> {
    //     // Add fee calculation
    //     let fee = self.calculate_fee(amount);
    //     let sender_balance = self.balance(&sender);
    //     let receiver_balance = self.balance(&receiver);

    //     //Check if Sender has enough balance for fee and transfer amount
    //     let total_needed = amount
    //         .checked_add(fee)
    //         .ok_or(BalancesError::OverflowInCalculation)?;
    //     if sender_balance < total_needed {
    //         return Err(BalancesError::InsufficientBalance);
    //     }

    //     let new_sender_balance = sender_balance
    //         .checked_sub(amount)
    //         .ok_or(BalancesError::InsufficientFunds)?;
    //     let new_receiver_balance = receiver_balance
    //         .checked_add(amount)
    //         .ok_or(BalancesError::OverflowInTransfer)?;

    //     self.balances.insert(sender.clone(), new_sender_balance);
    //     self.balances.insert(receiver, new_receiver_balance);

    //     //Handle fee payment and deduct from sender's balance
    //     self.handle_fee_payment(&sender, fee)?;

    //     Ok(())
    // }
    pub fn transfer(
        &mut self,
        sender: T::AccountId,
        receiver: T::AccountId,
        amount: T::Balance,
    ) -> Result<(), BalancesError> {
        let fee = self.calculate_fee(amount);
        let sender_balance = self.balance(&sender);
        let receiver_balance = self.balance(&receiver);

        // Check if sender has enough balance for transfer + fee
        let total_needed = amount
            .checked_add(&fee)
            .ok_or(BalancesError::OverflowInCalculation)?;
        
        if sender_balance < total_needed {
            return Err(BalancesError::InsufficientBalance);
        }

        // Calculate new balances
        let new_sender_balance = sender_balance
            .checked_sub(&amount)
            .ok_or(BalancesError::InsufficientFunds)?;
        let new_receiver_balance = receiver_balance
            .checked_add(&amount)
            .ok_or(BalancesError::OverflowInTransfer)?;

        // Update balances
        self.balances.insert(sender.clone(), new_sender_balance);
        self.balances.insert(receiver, new_receiver_balance);

        // Handle fee payment
        self.handle_fee_payment(&sender, fee)?;

        Ok(())
    }
}

// Enum for calls
pub enum Call<T: Config> {
    Transfer {
        to: T::AccountId,
        amount: T::Balance,
    },
}

// Implement dispatch for the pallet
impl<T: Config> crate::support::Dispatch for Pallet<T> {
    type Call = Call<T>;
    type Caller = T::AccountId;

    fn dispatch(
        &mut self,
        caller: Self::Caller,
        call: Self::Call,
    ) -> crate::support::DispatchResult {
        match call {
            Call::Transfer { to, amount } => {
                self.transfer(caller, to, amount)
                    .map_err(|_| "Transfer failed")?;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

       struct TestConfig;

    impl crate::system::Config for TestConfig {
        type AccountId = String;
        type BlockNumber = u32;
        type Nonce = u32;
    }

    impl Config for TestConfig {
        type Balance = u128;  // Use u128 for balances in tests
    }

    #[test]
    fn init_balances() {
        let mut balances = Pallet::<TestConfig>::new();

        assert_eq!(balances.balance(&"alice".to_string()), 0);
        balances.set_balance(&"alice".to_string(), 100);
        assert_eq!(balances.balance(&"alice".to_string()), 100);
        assert_eq!(balances.balance(&"bob".to_string()), 0);
    }
    // fn init_balances() {
    //     let mut balances = super::Pallet::new();

    //     assert_eq!(balances.balance(&"alice".to_string()), 0);
    //     balances.set_balance(&"alice".to_string(), 100);
    //     assert_eq!(balances.balance(&"alice".to_string()), 100);
    //     assert_eq!(balances.balance(&"bob".to_string()), 0);
    // }

    #[test]
     fn transfer_balance() {
        let mut balances = Pallet::<TestConfig>::new();

        // Try transfer without sufficient balance
        assert_eq!(
            balances.transfer("alice".to_string(), "bob".to_string(), 51),
            Err(BalancesError::InsufficientBalance)
        );

        balances.set_balance(&"alice".to_string(), 100);
        assert_eq!(
            balances.transfer("alice".to_string(), "bob".to_string(), 51),
            Ok(())
        );

        assert_eq!(balances.balance(&"alice".to_string()), 49);
        assert_eq!(balances.balance(&"bob".to_string()), 51);
    }
    // fn transfer_balance() {
    //     let mut balances = super::Pallet::new();

    //     assert_eq!(
    //         balances.transfer("alice".to_string(), "bob".to_string(), 51),
    //         Err(BalancesError::InsufficientFunds)
    //     );

    //     balances.set_balance(&"alice".to_string(), 500);
    //     balances.set_balance(&"bob".to_string(), 20);

    //     assert_eq!(
    //         balances.transfer("alice".to_string(), "bob".to_string(), 40),
    //         Ok(())
    //     );

    //     assert_eq!(balances.balance(&"alice".to_string()), 450);
    //     assert_eq!(balances.balance(&"bob".to_string()), 60);

    //     // assert_eq!(
    //     //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
    //     //     Err("Not  enough funds")
    //     // );

    //     // balances.set_balance(&"alice".to_string(), 100);
    //     // assert_eq!(
    //     //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
    //     //     Ok(())
    //     // );

    //     // assert_eq!(balances.balance(&"alice".to_string()), 49);
    //     // assert_eq!(balances.balance(&"bob".to_string()), 51);

    //     // assert_eq!(
    //     //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
    //     //     Err("No enough balances")
    //     // );
    // }

    #[test]
     fn transfer_with_fee_recipient() {
        let mut balances = Pallet::<TestConfig>::new_with_fee_config(5, Some("treasury".to_string()));

        balances.set_balance(&"alice".to_string(), 100);
        balances.set_balance(&"treasury".to_string(), 10);

        assert_eq!(
            balances.transfer("alice".to_string(), "bob".to_string(), 30),
            Ok(())
        );

        // Alice: 100 - 30 - 5 = 65
        assert_eq!(balances.balance(&"alice".to_string()), 65);
        // Bob: 0 + 30 = 30
        assert_eq!(balances.balance(&"bob".to_string()), 30);
        // Treasury: 10 + 5 = 15
        assert_eq!(balances.balance(&"treasury".to_string()), 15);
    }
    // fn transfer_with_fee_recipient() {
    //     let mut balances = super::Pallet::new_with_fee_config(5, Some("treasury".to_string()));

    //     balances.set_balance(&"alice".to_string(), 100);
    //     balances.set_balance(&"treasury".to_string(), 10);

    //     assert_eq!(
    //         balances.transfer("alice".to_string(), "bob".to_string(), 30),
    //         Ok(())
    //     );

    //     // Alice: 100 - 30 - 5 = 65
    //     assert_eq!(balances.balance(&"alice".to_string()), 65);
    //     // Bob: 0 + 30 = 30
    //     assert_eq!(balances.balance(&"bob".to_string()), 30);
    //     // Treasury: 10 + 5 = 15
    //     assert_eq!(balances.balance(&"treasury".to_string()), 15);
    // }
}
