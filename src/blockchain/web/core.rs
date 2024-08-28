#![allow(unused)]
use log::debug;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::blockchain::core::Blockchain;
use rocket::fs::{FileServer, relative, NamedFile};
use rocket::http::uri::fmt::Kind::Path;
use rocket::response::content::RawHtml;
use crate::blockchain::db::mongodb::core::MongoDB;
use crate::blockchain::transaction::Transaction;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::utils::calculations::calculate_fee;
use crate::blockchain::wallet::Wallet;
use crate::blockchain::db::mongodb;

type SharedBlockchain = Arc<Mutex<Blockchain>>;
type SharedTransactionPool = Arc<Mutex<TransactionPool>>;
type SharedDatabase = Arc<Mutex<MongoDB>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionRequest {
    sender: String,
    receiver: String,
    amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinerRequest {
    address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletRequest {
    address: String,
}

#[post("/transaction", format = "application/json", data = "<transaction>")]
async fn transaction(
    transaction: Json<TransactionRequest>, 
    _blockchain: &rocket::State<SharedBlockchain>, 
    pool: &rocket::State<SharedTransactionPool>
) -> &'static str {
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let tx = Transaction::new(transaction.sender.clone(), transaction.receiver.clone(), transaction.amount, timestamp, calculate_fee(transaction.amount));
    pool.lock().await.add_transaction(tx.clone()).await;
    debug!("Transaction added to pool: {:?}", tx);
    "Transaction received"
}

#[post("/mine", format = "application/json", data = "<miner>")]
async fn mine(miner: Json<MinerRequest>, blockchain: &rocket::State<SharedBlockchain>, pool: &rocket::State<SharedTransactionPool>) -> String {
    let mut blockchain = blockchain.lock().await;
    let mut pool = pool.lock().await;
    let (duration, difficulty) = blockchain.mine_block(&mut pool, &miner.address).await;
    "Block mined in ".to_owned() + duration.as_secs_f64().to_string().as_str() + " seconds" + " with difficulty " + difficulty.to_string().as_str()
}

#[get("/wallet/balance", format = "application/json", data = "<wallet>")]
async fn get_balance(wallet: Json<WalletRequest>, db: &rocket::State<MongoDB>) -> String {
    let wallet = Wallet::new(wallet.address.clone(), db.inner().clone()).await;
    format!("Balance: {}", wallet.get_balance())
}

#[get("/blockchain")]
async fn get_blockchain(blockchain: &rocket::State<SharedBlockchain>) -> Json<Blockchain> {
    let blockchain = blockchain.lock().await;
    debug!("Blockchain: {:?}", blockchain);
    Json(blockchain.clone())
}

#[get("/transactions")]
async fn get_transactions(pool: &rocket::State<SharedTransactionPool>) -> Json<TransactionPool> {
    let pool = pool.lock().await;
    debug!("Transaction pool: {:?}", pool);
    Json(pool.clone())
}

#[get("/")]
async fn index() -> RawHtml<&'static str> {
    RawHtml(
        r#"
        <!DOCTYPE html>
        <html lang='en'>
        <head>
            <meta charset='UTF-8'>
            <meta name='viewport' content='width=device-width, initial-scale=1.0'>
            <title>Serenity Blockchain</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    line-height: 1.6;
                    color: #333;
                    max-width: 800px;
                    margin: 0 auto;
                    padding: 20px;
                    background-color: #f4f4f4;
                }
                .container {
                    background-color: #fff;
                    border-radius: 8px;
                    padding: 30px;
                    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
                }
                h1 {
                    color: #2c3e50;
                    border-bottom: 2px solid #3498db;
                    padding-bottom: 10px;
                }
                p {
                    font-size: 18px;
                }
                .cta-button {
                    display: inline-block;
                    background-color: #3498db;
                    color: #fff;
                    padding: 10px 20px;
                    text-decoration: none;
                    border-radius: 5px;
                    font-weight: bold;
                    margin-top: 20px;
                }
                .cta-button:hover {
                    background-color: #2980b9;
                }
            </style>
        </head>
        <body>
            <div class='container'>
                <h1>Welcome to Serenity Blockchain</h1>
                <p>
                    Serenity is a cutting-edge blockchain platform designed to revolutionize
                    transaction security and efficiency. Our next-generation technology
                    ensures unparalleled protection for your digital assets while providing
                    a seamless user experience.
                </p>
            </div>
        </body>
        </html>
        "#
    )
}

#[allow(dead_code, unused_variables)]
#[launch]
pub async fn rocket() -> _ {
    let db = mongodb::core::MongoDB::new().await;

    let mut blockchain = Blockchain::new(db.clone()).await;
    if blockchain.chain.is_empty() {
        let genesis_block = blockchain.create_genesis_block().await;
        blockchain.chain.push(genesis_block);
    }

    let blockchain_state = Arc::new(Mutex::new(blockchain));
    let transaction_pool = Arc::new(Mutex::new(TransactionPool::new(db.clone())));

    rocket::build()
        .manage(blockchain_state)
        .manage(transaction_pool)
        .attach(rocket::fairing::AdHoc::on_ignite("Database Migrations", |rocket| async {
            let db = mongodb::core::MongoDB::new().await;
            db.migrate().await;
            rocket
        }))
        .manage(db)
        .mount("/", routes![transaction, get_blockchain, mine, get_transactions, get_balance, index])
}