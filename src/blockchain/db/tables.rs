// use super::core::Database;
// use ecdsa::Error;

// pub fn create_wallet_table(instance: &Database) -> Result<(), Error> {
//     let conn = instance.pool.get().expect("Failed to get connection.");
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS wallets (
//             id INTEGER PRIMARY KEY,
//             address TEXT NOT NULL,
//             balance REAL NOT NULL
//         )",
//         [],
//     ).expect("Failed to create wallets table.");
//     Ok(())
// }

// pub fn create_blocks_table(instance: &Database) -> Result<(), Error> {
//     let conn = instance.pool.get().expect("Failed to get connection.");
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS blocks (
//             id INTEGER PRIMARY KEY,
//             \"index\" INTEGER NOT NULL,
//             timestamp INTEGER NOT NULL,
//             data TEXT NOT NULL,
//             prev_hash TEXT NOT NULL,
//             hash TEXT NOT NULL,
//             nonce INTEGER NOT NULL,
//             transactions TEXT
//         )",
//         [],
//     ).expect("Failed to create blocks table.");
//     Ok(())
// }

// pub fn create_transaction_table(instance: &Database) -> Result<(), Error> {
//     let conn = instance.pool.get().expect("Failed to get connection.");
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS transactions (
//             id INTEGER PRIMARY KEY,
//             sender TEXT NOT NULL,
//             receiver TEXT NOT NULL,
//             amount REAL NOT NULL,
//             timestamp INTEGER NOT NULL
//         )",
//         [],
//     ).expect("Failed to create transactions table.");
//     Ok(())
// }