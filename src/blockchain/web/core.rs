#![allow(unused)]
use log::debug;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::blockchain::core::Blockchain;
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
async fn mine(miner: Json<MinerRequest>, blockchain: &rocket::State<SharedBlockchain>, pool: &rocket::State<SharedTransactionPool>) -> &'static str {
    let mut blockchain = blockchain.lock().await;
    let mut pool = pool.lock().await;
    blockchain.mine_block(&mut pool, &miner.address).await;
    "Block mined"
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
        .mount("/", routes![transaction, get_blockchain, mine, get_transactions, get_balance])
}