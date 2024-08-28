// BIB TODO
#![allow(unused)]
use futures::TryStreamExt;
use log::info;
use mongodb::{ 
	bson::{doc, Document},
	Client, Collection,
};
use serde::Serialize;
use serde_json::to_string;

use crate::blockchain::{block::Block, transaction::Transaction};

#[derive(Debug, Clone)]
pub struct MongoDB {
    pub client: Client,
}

impl MongoDB {
    pub async fn new() -> MongoDB {
        let client = connect().await.unwrap();
        MongoDB {
            client,
        }
    }

    pub fn insert_block(&self, block: Block) -> mongodb::error::Result<()> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("BLOCKCHAIN");
        let document = doc! {
            "index": block.index,
            "timestamp": block.timestamp as i64,
            "data": block.data.clone(),
            "prev_hash": block.prev_hash.clone(),
            "hash": block.hash.clone(),
            "nonce": block.nonce as i64,
        };
        let _ = collection.insert_one(document);
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

    pub async fn get_blocks(&self) -> mongodb::error::Result<Vec<Block>> {
        let collection: Collection<Document> = self.client.database("SERENITY").collection("BLOCKCHAIN");
        let mut cursor = collection.find(doc! {}).await?;
        let mut blocks = vec![];
    
        while let Some(doc) = cursor.try_next().await? {
            let block = Block {
                index: doc.get_i64("index").unwrap() as u32,
                timestamp: doc.get_i64("timestamp").unwrap() as u64,
                data: doc.get_str("data").unwrap().to_string(),
                prev_hash: doc.get_str("prev_hash").unwrap().to_string(),
                hash: doc.get_str("hash").unwrap().to_string(),
                nonce: doc.get_i64("nonce").unwrap() as u64,
                transactions: vec![],
            };
            blocks.push(block);
        }
    
        Ok(blocks)
    }
}

pub async fn connect() -> mongodb::error::Result<Client> {
    let uri = "mongodb+srv://wiseleet:9MMeTZLzvP1nfT1W@serenitydb.uminc.mongodb.net/?retryWrites=true&w=majority&appName=SerenityDB";

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