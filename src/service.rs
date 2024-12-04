
use std::collections::HashMap;

use crate::entities::{ClientAccount, Transaction};

pub struct PaymentEngine {
    accounts: HashMap<u16, ClientAccount>, 
    transactions: HashMap<u32, (u16, f64, bool)>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine {
            accounts : HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction : Transaction) {
        match transaction.r#type.as_str() {
            "deposit" => self.deposit(transaction.tx, transaction.client, transaction.amount.unwrap_or(0.0)),
            "withdrawal" => self.withdraw( transaction.tx, transaction.client, transaction.amount.unwrap_or(0.0)),
            "dispute" => self.dispute(transaction.tx, transaction.client),
            "resolve" => self.resolve(transaction.tx, transaction.client),
            "chargeback" => self.chargeback(transaction.tx, transaction.client),
            _ => eprintln!("Unknown transaction type: {}", transaction.r#type),
        }
    }

    fn deposit(&mut self, transaction_id : u32, client_id : u16, amount: f64) {

        let current_account = self.accounts.entry(client_id).or_insert(ClientAccount {
            client: client_id,
            available_amount: 0.0,
            held_amount : 0.0,
            total_amount: 0.0,
            locked : false,
        });

        if current_account.locked {
            return;
        }

        current_account.available_amount += amount;
        current_account.total_amount += amount;

        self.transactions.insert(transaction_id, (client_id, amount, false));

    }

    fn withdraw(&mut self, transaction_id : u32, client_id : u16, amount: f64) {

        if let Some(current_account) = self.accounts.get_mut(&client_id) {
            if current_account.locked || current_account.available_amount < amount {
                return;
            }

            current_account.available_amount -= amount;
            current_account.total_amount -= amount;

            self.transactions.insert(transaction_id, (client_id, amount, false));
        }
    } 

    
    fn dispute(&mut self, transaction_id : u32, client_id : u16) {

        if let Some((transaction_client_id, amount, is_disputed)) = self.transactions.get_mut(&transaction_id) {
        
            if *transaction_client_id == client_id && !*is_disputed {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.available_amount -= *amount;
                    account.held_amount += *amount;

                    //we need to mark the transaction as disputed, so we don't disputed again in the future
                    *is_disputed = true;
                }
            } else if *is_disputed {
                eprintln!("Transaction {} is already disputed.", transaction_id);
            }
        }
    }

    fn resolve(&mut self, transaction_id : u32, client_id : u16) {

        if let Some((transaction_client_id, amount, is_disputed)) = self.transactions.get_mut(&transaction_id) {

            if *transaction_client_id == client_id && *is_disputed {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.available_amount += *amount;
                    account.held_amount -= *amount;

                    //we resolved the transaction, so we can remove it from the map, since another future transaction cannot operate on a solved transaction
                    self.transactions.remove(&transaction_id);
                }
            } else if !*is_disputed {
                eprintln!("Transaction {} is not disputed, so we cannot solve it", transaction_id);
            }
        }
    }

    fn chargeback(&mut self, transaction_id : u32, client_id : u16) {

        if let Some((transaction_client_id, amount, is_disputed)) = self.transactions.get_mut(&transaction_id) {

            if *transaction_client_id == client_id && *is_disputed {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.total_amount -= *amount;
                    account.held_amount -= *amount;
                    account.locked = true;

                    //after chargeback, we remove the transaction, since no future transaction can operate on this transaction anymore
                    self.transactions.remove(&transaction_id);
                }
            } else if !*is_disputed {
                eprintln!("Transaction {} is not disputed, so we cannot chargeback it", transaction_id);
            }
        }
    }
    

    pub fn get_client_accounts(&mut self) -> Vec<ClientAccount> {
        self.accounts.values().cloned().collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Transaction;

    fn create_test_transaction(tx_id: u32, client_id: u16, amount: f64, r#type: &str) -> Transaction {
        Transaction {
            tx: tx_id,
            client: client_id,
            amount: Some(amount),
            r#type: r#type.to_string(),
        }
    }

    #[test]
    fn test_deposit() {
        let tx = create_test_transaction(1, 1, 100.0, "deposit");
        
        let mut engine = PaymentEngine::new();
        engine.process_transaction(tx);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();
        assert_eq!(current_account.available_amount, 100.0);
        assert_eq!(current_account.total_amount, 100.0);
    }

    #[test]
    fn test_withdrawal() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 100.0, "deposit");
        engine.process_transaction(tx_deposit);
        
        let tx_withdrawal = create_test_transaction(2, 1, 50.0, "withdrawal");
        engine.process_transaction(tx_withdrawal);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();
        assert_eq!(current_account.available_amount, 50.0);
        assert_eq!(current_account.total_amount, 50.0);
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 50.0, "deposit");
        engine.process_transaction(tx_deposit);
        
        let tx_withdrawal = create_test_transaction(2, 1, 100.0, "withdrawal");
        engine.process_transaction(tx_withdrawal);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();

        // Withdrawal should be ignored, account should still have the original amount
        assert_eq!(current_account.available_amount, 50.0);
        assert_eq!(current_account.total_amount, 50.0);
    }

    #[test]
    fn test_dispute() {
        let mut engine = PaymentEngine::new();
        let tx_deposit_one = create_test_transaction(1, 1, 100.0, "deposit");
        let tx_deposit_two = create_test_transaction(2, 1, 25.0, "deposit");
        engine.process_transaction(tx_deposit_one);
        engine.process_transaction(tx_deposit_two);

        let tx_dispute = create_test_transaction(2, 1, 0.0, "dispute");
        engine.process_transaction(tx_dispute);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();

        assert_eq!(current_account.available_amount, 100.0);
        assert_eq!(current_account.held_amount, 25.0);

        // Attempting a second dispute should be ignored
        let tx_dispute = create_test_transaction(2, 1, 0.0, "dispute");
        engine.process_transaction(tx_dispute);
    }

    #[test]
    fn test_resolve() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 100.0, "deposit");
        engine.process_transaction(tx_deposit);

        let tx_dispute = create_test_transaction(2, 1, 100.0, "dispute");
        engine.process_transaction(tx_dispute);

        let tx_resolve = create_test_transaction(3, 1, 100.0, "resolve");
        engine.process_transaction(tx_resolve);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();
        assert_eq!(current_account.available_amount, 100.0);
        assert_eq!(current_account.held_amount, 0.0);
    }

    #[test]
    fn test_chargeback() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 100.0, "deposit");
        engine.process_transaction(tx_deposit);

        let tx_dispute = create_test_transaction(1, 1, 0.0, "dispute");
        engine.process_transaction(tx_dispute);

        let tx_chargeback = create_test_transaction(1, 1, 0.0, "chargeback");
        engine.process_transaction(tx_chargeback);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();
        assert_eq!(current_account.held_amount, 0.0);
        assert_eq!(current_account.total_amount, 0.0);
        assert_eq!(current_account.locked, true);
    }

    #[test]
    fn test_chargeback_on_non_disputed() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 100.0, "deposit");
        engine.process_transaction(tx_deposit);

        let tx_chargeback = create_test_transaction(1, 1, 100.0, "chargeback");
        engine.process_transaction(tx_chargeback);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();

        assert_eq!(current_account.held_amount, 0.0);
        assert_eq!(current_account.total_amount, 100.0);
        assert_eq!(current_account.locked, false);
    }

    #[test]
    fn test_multiple_operations() {
        let mut engine = PaymentEngine::new();
        let tx_deposit = create_test_transaction(1, 1, 200.0, "deposit");
        engine.process_transaction(tx_deposit);

        let tx_withdrawal = create_test_transaction(1, 1, 100.0, "withdrawal");
        engine.process_transaction(tx_withdrawal);

        let tx_dispute = create_test_transaction(1, 1, 100.0, "dispute");
        engine.process_transaction(tx_dispute);

        let tx_resolve = create_test_transaction(1, 1, 100.0, "resolve");
        engine.process_transaction(tx_resolve);

        let tx_chargeback = create_test_transaction(1, 1, 100.0, "chargeback");
        engine.process_transaction(tx_chargeback);

        let accounts = engine.get_client_accounts();
        let current_account = accounts.get(0).unwrap();

        assert_eq!(current_account.available_amount, 100.0);
        assert_eq!(current_account.held_amount, 0.0);
        assert_eq!(current_account.locked, false); 
    }
}
