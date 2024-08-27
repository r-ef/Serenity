use std::sync::{Arc, Mutex};
use crate::blockchain::transaction::Transaction;
use crate::utils::calculations::calculate_fee;
use crate::blockchain::db::core::Database;

pub struct Wallet {
    pub address: String,
    pub balance: f64,
    pub db: Arc<Mutex<Database>>,
}

impl Wallet {
    pub fn new(address: String, db: Arc<Mutex<Database>>) -> Wallet {
        // Retrieve the balance from the database if it exists
        let balance = db.lock().unwrap().get_balance(&address).unwrap_or(0.0);

        Wallet {
            address,
            balance,
            db,
        }
    }

    pub fn send_money(&mut self, receiver: String, amount: f64) -> Transaction {
        let fee = calculate_fee(amount);
        let total_amount = amount + fee;

        if self.balance < total_amount {
            panic!("Insufficient balance");
        }

        let timestamp = chrono::Utc::now().timestamp() as u64;
        let transaction = Transaction {
            sender: self.address.clone(),
            receiver: receiver.clone(),
            amount,
            fee,
            timestamp,
        };

        self.balance -= total_amount;

        let mut db = self.db.lock().unwrap();
        db.update_balance(&self.address, self.balance).expect("Failed to update balance");
        db.insert_transaction(&transaction).expect("Failed to insert transaction");

        let mut receiver_balance = db.get_balance(&receiver).unwrap_or(0.0);
        receiver_balance += amount;
        db.update_balance(&receiver, receiver_balance).expect("Failed to update receiver's balance");

        transaction
    }

    pub fn receive_money(&mut self, amount: f64) {
        self.balance += amount;

        self.db.lock().unwrap().update_balance(&self.address, self.balance).expect("Failed to update balance");
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }
}
