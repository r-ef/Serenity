// // basic blockchain infrastructure and demonstrates how to initialize, mine blocks, and manage a transaction pool

// mod blockchain {
//     pub mod block;
//     pub mod core;
//     pub mod hashing;
//     pub mod transaction;
//     pub mod transaction_pool;
//     pub mod wallet;
// }

// mod utils {
//     pub mod logging;
// }

// use blockchain::core::Blockchain;
// use blockchain::transaction_pool::TransactionPool;
// use utils::logging;
// use log::info;

// fn main() {
//     logging::setup_logger();


//     let mut blockchain = Blockchain::new();
//     let genesis_block = Blockchain::create_genesis_block();
//     blockchain.chain.push(genesis_block);
    
//     let mut transaction_pool = TransactionPool::new();
    
//     for i in 1..=5 {
//         let transaction = blockchain::transaction::Transaction::new(
//             format!("Alice_{}", i),
//             format!("Bob_{}", i),
//             10.0 * i as f64,
//         );
//         transaction_pool.add_transaction(transaction);
//     }
    
//     for i in 1..=20 {
//         info!("Mining block {}", i);
//         blockchain.mine_block(&mut transaction_pool);
//         info!("Current blockchain difficulty: {}", blockchain.difficulty);
//         info!("Current blockchain length: {}", blockchain.chain.len());
//     }
// }

#[macro_use] extern crate rocket;
mod blockchain {
    pub mod block;
    pub mod core;
    pub mod hashing;
    pub mod transaction;
    pub mod transaction_pool;
    pub mod wallet;
    pub mod web {
        pub mod core;
    }
    pub mod db {
        pub mod core;
    }
}

mod utils {
    pub mod logging;
}
#[tokio::main]
async fn main() {
    utils::logging::setup_logger();
    let _ = blockchain::web::core::rocket().launch().await;
}