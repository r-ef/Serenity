use serde::Serialize;
use crate::blockchain::transaction::Transaction;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
    pub pool: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> TransactionPool {
        TransactionPool {
            pool: vec![],
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pool.push(transaction);
    }

    pub fn remove_transaction(&mut self, transaction: &Transaction) {
        self.pool.retain(|tx| tx != transaction);
    }

    pub fn clear_pool(&mut self) {
        self.pool.clear();
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.pool.clone()
    }
}
