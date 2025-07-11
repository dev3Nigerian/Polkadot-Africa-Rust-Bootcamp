use std::{collections::BTreeMap, fmt::write};

#[derive(Debug)]
pub struct Pallet {
    balances: BTreeMap<String, u128>,
    base_fee: u128,
    fee_recipient: Option<String>,
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

impl Pallet {
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
            base_fee: 10,
            fee_recipient: None,
        }
    }

    pub fn new_with_fee_config(base_fee: u128, fee_recipient: Option<String>) -> Self {
        Self {
            balances: BTreeMap::new(),
            base_fee,
            fee_recipient,
        }
    }

    pub fn set_transaction_fee(&mut self, fee: u128) {
        self.base_fee = fee;
    }

    pub fn get_transaction_fee(&self) -> u128 {
        self.base_fee
    }

    pub fn set_fee_recipient(&mut self, recipient: Option<String>) {
        self.fee_recipient = recipient;
    }

    fn calculate_fee(&self, _amount: u128) -> u128 {
        if _amount > 100 {
            _amount / 10
        } else {
            self.base_fee
        }
    }

    fn handle_fee_payment(&mut self, who: &String, fee: u128) -> Result<(), BalancesError> {
        let payer_balance = self.balance(who);
        if payer_balance < fee {
            return Result::Err(BalancesError::InsufficientFunds);
        }

        //Deduct fee from Payer
        self.balances.insert(who.clone(), payer_balance - fee);

        match &self.fee_recipient {
            Some(recipient) => {
                let recipient_balance = self.balance(recipient);
                self.balances
                    .insert(recipient.clone(), recipient_balance + fee);
            }
            None => {}
        }
        Ok(())
    }

    pub fn set_balance(&mut self, who: &String, amount: u128) {
        self.balances.insert(who.clone(), amount);
    }

    pub fn balance(&self, who: &String) -> u128 {
        *self.balances.get(who).unwrap_or(&0)
    }

    //Implemented the Balances Error here
    pub fn get_transfer_cost(&self, amount: u128) -> Result<u128, BalancesError> {
        let fee = self.calculate_fee(amount);
        amount.checked_add(fee).map_or_else(
            || Err(BalancesError::OverflowInCalculation),
            |total_cost| Ok(total_cost),
        )
    }

    pub fn transfer(
        &mut self,
        sender: String,
        receiver: String,
        amount: u128,
    ) -> Result<(), BalancesError> {
        // Add fee calculation
        let fee = self.calculate_fee(amount);
        let sender_balance = self.balance(&sender);
        let receiver_balance = self.balance(&receiver);

        //Check if Sender has enough balance for fee and transfer amount
        let total_needed = amount
            .checked_add(fee)
            .ok_or(BalancesError::OverflowInCalculation)?;
        if sender_balance < total_needed {
            return Err(BalancesError::InsufficientBalance);
        }

        let new_sender_balance = sender_balance
            .checked_sub(amount)
            .ok_or(BalancesError::InsufficientFunds)?;
        let new_receiver_balance = receiver_balance
            .checked_add(amount)
            .ok_or(BalancesError::OverflowInTransfer)?;

        self.balances.insert(sender.clone(), new_sender_balance);
        self.balances.insert(receiver, new_receiver_balance);

        //Handle fee payment and deduct from sender's balance
        self.handle_fee_payment(&sender, fee)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_balances() {
        let mut balances = super::Pallet::new();

        assert_eq!(balances.balance(&"alice".to_string()), 0);
        balances.set_balance(&"alice".to_string(), 100);
        assert_eq!(balances.balance(&"alice".to_string()), 100);
        assert_eq!(balances.balance(&"bob".to_string()), 0);
    }

    #[test]
    fn transfer_balance() {
        let mut balances = super::Pallet::new();

        assert_eq!(
            balances.transfer("alice".to_string(), "bob".to_string(), 51),
            Err(BalancesError::InsufficientFunds)
        );

        balances.set_balance(&"alice".to_string(), 500);
        balances.set_balance(&"bob".to_string(), 20);

        assert_eq!(
            balances.transfer("alice".to_string(), "bob".to_string(), 40),
            Ok(())
        );

        assert_eq!(balances.balance(&"alice".to_string()), 450);
        assert_eq!(balances.balance(&"bob".to_string()), 60);

        // assert_eq!(
        //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
        //     Err("Not  enough funds")
        // );

        // balances.set_balance(&"alice".to_string(), 100);
        // assert_eq!(
        //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
        //     Ok(())
        // );

        // assert_eq!(balances.balance(&"alice".to_string()), 49);
        // assert_eq!(balances.balance(&"bob".to_string()), 51);

        // assert_eq!(
        //     balances.transfer("alice".to_string(), "bob".to_string(), 51),
        //     Err("No enough balances")
        // );
    }

    #[test]
    fn transfer_with_fee_recipient() {
        let mut balances = super::Pallet::new_with_fee_config(5, Some("treasury".to_string()));

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
}
