use serde::Serialize;
use crate::blockchain::transaction::Transaction;
use super::db::mongodb::core::MongoDB;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
    pub pool: Vec<Transaction>,
    #[serde(skip)]
    pub db: MongoDB
}

impl TransactionPool {
    pub fn new(db: MongoDB) -> TransactionPool {
        TransactionPool {
            pool: vec![],
            db,
        }
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) {
        let _ = self.db.insert_transaction(&transaction).await;
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
