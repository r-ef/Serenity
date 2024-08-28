#[macro_use]
extern crate rocket;
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
        pub mod tables;
        pub mod mongodb {
            pub mod core;
        }
    }
}

mod utils {
    pub mod logging;
    pub mod calculations;
}
#[tokio::main]
#[allow(dead_code)]
async fn main() {
    utils::logging::setup_logger();
    let _ = blockchain::web::core::rocket().await.launch().await;
}