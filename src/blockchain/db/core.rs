use ecdsa::Error;
use log::debug;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};
use serde::Serialize;
use serde_json::{from_str, to_string};
use crate::blockchain::{block::Block, transaction::Transaction};

#[derive(Debug)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let conn = self.pool.get().expect("Failed to get connection.");
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM blocks").expect("Failed to prepare query.");
        let count: i64 = stmt.query_row([], |row| row.get(0)).expect("Failed to query blocks count.");
        count.serialize(serializer)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}

impl Database {
    pub fn new(db_path: &str) -> Database {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder().build(manager).expect("Failed to create pool.");
        Database { pool }
    }

    pub fn create_blocks_table(&self) -> Result<(), Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY,
                \"index\" INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                data TEXT NOT NULL,
                prev_hash TEXT NOT NULL,
                hash TEXT NOT NULL,
                nonce INTEGER NOT NULL,
                transactions BLOB
            )",
            [],
        ).expect("Failed to create blocks table.");
        Ok(())
    }

    pub fn create_transaction_table(&self) -> Result<(), Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                sender TEXT NOT NULL,
                receiver TEXT NOT NULL,
                amount REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        ).expect("Failed to create transactions table.");
        Ok(())
    }

    pub fn create_tables(&self) -> Result<(), Error> {
        self.create_blocks_table()?;
        self.create_transaction_table()?;
        Ok(())
    }

    pub fn insert_block(&self, block: Block) -> Result<(), Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        conn.execute(
            "INSERT INTO blocks (\"index\", timestamp, data, prev_hash, hash, nonce) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![block.index, block.timestamp, block.data, block.prev_hash, block.hash, block.nonce],
        ).expect("Failed to insert block.");
        
        for transaction in block.transactions {
            self.insert_transaction(&transaction).expect("Failed to insert transaction.");
        }
        
        Ok(())
    }

    pub fn insert_transaction(&self, transaction: &Transaction) -> Result<(), Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        conn.execute(
            "INSERT INTO transactions (sender, receiver, amount, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![
                transaction.sender,
                transaction.receiver,
                transaction.amount,
                transaction.timestamp,
            ],
        ).expect("Failed to insert transaction.");
        Ok(())
    }

    pub fn get_transactions(&self) -> Result<Vec<Transaction>, Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        let mut stmt = conn.prepare("SELECT sender, receiver, amount, timestamp FROM transactions").expect("Failed to prepare query.");
        let tx_iter = stmt.query_map([], |row| {
            Ok(Transaction {
                sender: row.get(0)?,
                receiver: row.get(1)?,
                amount: row.get(2)?,
                timestamp: row.get(3)?,
            })
        }).expect("Failed to query transactions.");
    
        let mut transactions = Vec::new();
        for tx in tx_iter {
            transactions.push(tx.expect("Failed to get transaction."));
        }
        Ok(transactions)
    }

    pub fn get_blocks(&self) -> Result<Vec<Block>, Error> {
        let conn = self.pool.get().expect("Failed to get connection.");
        let mut stmt = conn.prepare("SELECT \"index\", timestamp, data, prev_hash, hash, nonce, transactions FROM blocks").expect("Failed to prepare query.");
        let block_iter = stmt.query_map([], |row| {
            let transactions_json: String = row.get(6)?;
            let transactions: Vec<Transaction> = from_str(&transactions_json).expect("Failed to deserialize transactions");
    
            Ok(Block {
                index: row.get(0)?,
                timestamp: row.get(1)?,
                data: row.get(2)?,
                prev_hash: row.get(3)?,
                hash: row.get(4)?,
                nonce: row.get(5)?,
                transactions,
            })
        }).expect("Failed to query blocks.");
    
        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block.expect("Failed to get block."));
        }
        Ok(blocks)
    }
}
