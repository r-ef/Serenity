#![allow(unused)]
use log::debug;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::sync::{Mutex, Arc};

use crate::blockchain::core::Blockchain;
use crate::blockchain::db::core::Database;
use crate::blockchain::transaction::Transaction;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::utils::calculations::calculate_fee;
use crate::blockchain::wallet::Wallet;

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
fn transaction(
    transaction: Json<TransactionRequest>, 
    _blockchain: &rocket::State<Arc<Mutex<Blockchain>>>, 
    pool: &rocket::State<Arc<Mutex<TransactionPool>>>
) -> &'static str {
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let tx = Transaction::new(transaction.sender.clone(), transaction.receiver.clone(), transaction.amount, timestamp, calculate_fee(transaction.amount));
    pool.lock().unwrap().add_transaction(tx);
    "Transaction received"
}

#[post("/mine", format = "application/json", data = "<miner>")]
fn mine(miner: Json<MinerRequest>, _blockchain: &rocket::State<Arc<Mutex<Blockchain>>>, pool: &rocket::State<Arc<Mutex<TransactionPool>>>) -> &'static str {
    let mut blockchain = _blockchain.lock().expect("Failed to lock blockchain");
    let mut pool = pool.lock().unwrap();
    blockchain.mine_block(&mut pool, &miner.address);
    "Block mined"
}

#[get("/wallet/balance", format = "application/json", data = "<wallet>")]
fn get_balance(wallet: Json<WalletRequest>) -> String {
    let wallet = Wallet::new(wallet.address.clone(), Arc::new(Mutex::new(Database::new("database.db"))));
    format!("Balance: {}", wallet.get_balance())
}

#[get("/blockchain")]
fn get_blockchain(blockchain: &rocket::State<Arc<Mutex<Blockchain>>>) -> Json<Blockchain> {
    let blockchain = blockchain.lock().unwrap();
    debug!("Blockchain: {:?}", blockchain);
    Json(blockchain.clone())
}

#[get("/transactions")]
fn get_transactions(pool: &rocket::State<Arc<Mutex<TransactionPool>>>) -> Json<TransactionPool> {
    let pool = pool.lock().unwrap();
    debug!("Transaction pool: {:?}", pool);
    Json(pool.clone())
}

#[allow(dead_code, unused_variables)]
#[launch]
pub fn rocket() -> _ {
    let db = Arc::new(Mutex::new(Database::new("database.db")));
    let db_clone = db.clone();
    db_clone.lock().unwrap().create_tables().expect("Failed to create tables");

    let mut blockchain = Blockchain::new();
    if blockchain.chain.is_empty() {
        let genesis_block = blockchain.create_genesis_block();
        blockchain.chain.push(genesis_block);
    }

    let blockchain_state = Arc::new(Mutex::new(blockchain));
    let transaction_pool = Arc::new(Mutex::new(TransactionPool::new(db_clone.clone())));

    rocket::build()
        .manage(blockchain_state)
        .manage(transaction_pool)
        .mount("/", routes![transaction, get_blockchain, mine, get_transactions, get_balance])
}