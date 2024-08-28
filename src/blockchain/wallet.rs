use crate::blockchain::transaction::Transaction;
use crate::utils::calculations::calculate_fee;
use crate::blockchain::db::mongodb::core::MongoDB;

pub struct Wallet {
    pub address: String,
    pub balance: f64,
    pub db: MongoDB,
}

impl Wallet {
    pub async fn new(address: String, db: MongoDB) -> Wallet {
        let balance = db.get_balance(&address).await.unwrap_or(0.0);

        Wallet {
            address,
            balance,
            db,
        }
    }

    pub async fn send_money(&mut self, receiver: String, amount: f64) -> Transaction {
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
        let _ = self.db.update_balance(&self.address, self.balance).await;
        let _ = self.db.insert_transaction(&transaction).await;

        let mut receiver_balance = self.db.get_balance(&receiver).await.unwrap_or(0.0);
        receiver_balance += amount;
        let _ = self.db.update_balance(&receiver, receiver_balance).await;

        transaction
    }

    pub async fn receive_money(&mut self, amount: f64) {
        self.balance += amount;

        let _ = self.db.update_balance(&self.address, self.balance).await;
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }
}
