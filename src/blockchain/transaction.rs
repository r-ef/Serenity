use std::ops::Deref;
use std::fmt::{self, Display};

use ed25519_dalek::Signer;

use ed25519_dalek::{Signature, SigningKey};
use serde_with::skip_serializing_none;
use faster_hex::hex_encode;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub timestamp: u64,
    pub fee: f64,
}

impl Transaction {
    pub fn new(
        sender: String,
        receiver: String,
        amount: f64,
        timestamp: u64,
        fee: f64,
    ) -> Transaction {
        Transaction {
            sender,
            receiver,
            amount,
            timestamp,
            fee,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} transferred {} to {}",
            self.sender, self.amount, self.receiver
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().bytes().collect()
    }

    pub fn sign_transaction(&mut self) -> (Vec<u8>, SigningKey, Vec<u8>) {
        let message = self.to_bytes();
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let signature: Signature = signing_key.try_sign(message.as_slice()).unwrap();
        (signature.to_bytes().to_vec(), signing_key, message)
    }

    pub fn verify_transaction(
        &self,
        signing_key: SigningKey,
        signature: &Signature,
        message: Vec<u8>,
    ) {
        assert!(signing_key.verify(message.as_slice(), signature).is_ok())
    }
}

/// SHA3-256 hash
pub const TRANSACTION_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TransactionID([u8; TRANSACTION_ID_LENGTH]);

impl TransactionID {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_hex(&self) -> String {
        format!("{}", self)
    }
}

impl AsRef<[u8]> for TransactionID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for TransactionID {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl Display for TransactionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; TRANSACTION_ID_LENGTH * 2];
        let _ = hex_encode(self, &mut buf);
        write!(f, "{}", String::from_utf8_lossy(&buf))
    }
}