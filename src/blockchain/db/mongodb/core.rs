#![allow(unused)]
use futures::TryStreamExt;
use log::{debug, info};
use mongodb::{ 
    bson::{doc, Document},
    Client, Collection,
};
use serde::Serialize;
use serde_json::to_string;

use crate::blockchain::{block::Block, transaction::Transaction};


#[derive(Debug, Clone, Serialize)]
pub struct MongoDB {
    #[serde(skip_serializing)]
    pub client: Client,
}


impl MongoDB {
    pub async fn new() -> MongoDB {
        let client = connect().await.unwrap();
        MongoDB {
            client,
        }
    }

    pub async fn insert_block(&self, block: Block) -> mongodb::error::Result<()> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("BLOCKCHAIN");
        let document = doc! {
            "index": block.index,
            "timestamp": block.timestamp as i64,
            "data": block.data.clone(),
            "prev_hash": block.prev_hash.clone(),
            "hash": block.hash.clone(),
            "nonce": block.nonce as i64,
            "difficulty": block.difficulty as i64,
        };
        let _ = collection.insert_one(document).await;
        debug!("Block inserted into MongoDB");
        Ok(())
    }

    pub async fn update_block_transactions(&self, block: &Block, transaction: &Transaction) -> mongodb::error::Result<()> {
        // add transaction to block
        let collection: Collection<Document> = self.client.database("SERENITY").collection("BLOCKCHAIN");
        let filter = doc! { "index": block.index };
        let update = doc! { "$push": { "transactions": to_string(transaction).unwrap() } };
        let _ = collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn get_balance(&self, address: &str) -> mongodb::error::Result<f64> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("WALLETS");
        let filter = doc! { "address": address };
        let document = collection.find_one(filter).await?;
        let document = document.unwrap_or_default();
        let balance = document.get_f64("balance").unwrap_or_default();
        Ok(balance)
    }

    pub async fn update_balance(&self, address: &str, balance: f64) -> mongodb::error::Result<()> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("WALLETS");
        let filter = doc! { "address": address };
        let update = doc! { "$set": { "balance": balance } };
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        let _ = collection.update_one(filter, update).with_options(options).await?;
        Ok(())
    }

    pub async fn insert_transaction(&self, transaction: &Transaction) -> mongodb::error::Result<()> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("TRANSACTIONS");
        let document = doc! {
            "transaction": to_string(transaction).unwrap(),
        };
        let _ = collection.insert_one(document).await?;
        Ok(())
    }

    pub async fn get_transactions(&self) -> mongodb::error::Result<Vec<Transaction>> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("TRANSACTIONS");
        let mut cursor = collection.find(doc! {}).await?;
        let mut transactions = vec![];

        while let Some(doc) = cursor.try_next().await? {
            let transaction: Transaction = serde_json::from_str(doc.get_str("transaction").unwrap_or_default()).unwrap();
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    pub async fn migrate(&self) -> mongodb::error::Result<()> {
        let db = self.client.database("SERENITY");
        let _ = db.create_collection("BLOCKCHAIN").await?;
        let _ = db.create_collection("TRANSACTIONS").await?;
        let _ = db.create_collection("WALLETS").await?;
        Ok(())
    }

        pub async fn get_blocks(&self) -> mongodb::error::Result<Vec<Block>> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("BLOCKCHAIN");
        let mut cursor = collection.find(doc! {}).await?;
        let mut blocks = vec![];
    
        while let Some(doc) = cursor.try_next().await? {
            let block = Block {
                index: doc.get_i64("index").unwrap_or_default() as u32,
                timestamp: doc.get_i64("timestamp").unwrap_or_default() as u64,
                data: doc.get_str("data").unwrap_or_default().to_string(),
                prev_hash: doc.get_str("prev_hash").unwrap_or_default().to_string(),
                hash: doc.get_str("hash").unwrap_or_default().to_string(),
                nonce: doc.get_i64("nonce").unwrap_or_default() as u64,
                transactions: vec![],
                difficulty: doc.get_i64("difficulty").unwrap_or(1) as u32,
            };
            debug!("Block: {:?}", block);
            blocks.push(block);
        }
        Ok(blocks)
    }
}

pub async fn connect() -> mongodb::error::Result<Client> {
    // Read MongoDB connection string from environment. Do not hardcode secrets.
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string());

    let client = Client::with_uri_str(uri).await?;
    let db = client.database("SERENITY");
    let collection: Collection<Document> = db.collection("BLOCKCHAIN");
    let _ = collection.insert_one(doc! { "test": "test" });

    info!("Connected to MongoDB");
    Ok(client)
}

// pub fn insert_block(client: &Client, block: &Block) -> mongodb::error::Result<()> {
//     let db = client.database("SERENITY");
//     let collection: Collection<Document> = db.collection("BLOCKCHAIN");

//     let document = doc! {
//         "index": block.index,
//         "timestamp": block.timestamp as i64,
//         "data": block.data.clone(),
//         "prev_hash": block.prev_hash.clone(),
//         "hash": block.hash.clone(),
//         "nonce": block.nonce as i64,
//     };

//     let _ = collection.insert_one(document);
//     Ok(())
// }