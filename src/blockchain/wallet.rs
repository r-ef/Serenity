use crate::blockchain::transaction::Transaction;

pub struct Wallet {
    pub address: String,
    pub balance: f64,
}

impl Wallet {
    pub fn new(address: String) -> Wallet {
        Wallet {
            address,
            balance: 0.0,
        }
    }

    pub fn send_money(&mut self, receiver: String, amount: f64) -> Transaction {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let transaction = Transaction::new(self.address.clone(), receiver, amount, timestamp);
        self.balance -= amount;
        transaction
    }

    pub fn receive_money(&mut self, transaction: Transaction) {
        self.balance += transaction.amount;
    }
}
