use ed25519_dalek::Signer;

use ed25519_dalek::Signature;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub timestamp: u64,
    pub fee: f64,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: f64, timestamp: u64, fee: f64) -> Transaction {
        Transaction {
            sender,
            receiver,
            amount,
            timestamp,
            fee
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
