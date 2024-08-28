use std::{fmt, time::{SystemTime, UNIX_EPOCH}};
use ecdsa::Error;
use rand::Rng;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::convert::AsRef;
use ibig::UBig;
use std::ops::{Deref, DerefMut};
use std::fmt::{Debug, Display};
use sha3::{Digest, Sha3_256};
use faster_hex::hex_encode;

use super::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
    pub difficulty: u32,
}

/// SHA3-256 hash
pub const BLOCK_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct BlockID([u8; BLOCK_ID_LENGTH]);

#[allow(dead_code)]
impl Block {
    pub fn new(index: u32, data: String, prev_hash: String) -> Block {
        let mut rng = rand::thread_rng();
        let start_nonce = rng.gen_range(0..u64::MAX);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
            nonce: start_nonce,
            transactions: vec![],
            difficulty: 0,
        }
    }

    pub fn id(&self) -> Result<BlockID, Error> {
        let mut hasher = Sha3_256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.data.as_bytes());
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        Ok(hasher.finalize().into())
    }
}

impl BlockID {
    pub fn new() -> BlockID {
        Default::default()
    }

    /// Returns BlockID as a hex string
    pub fn as_hex(&self) -> String {
        format!("{}", self)
    }

    /// Converts from BlockID to BigInt.
    pub fn as_big_int(&self) -> UBig {
        UBig::from_be_bytes(self)
    }
}

impl AsRef<[u8]> for BlockID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for BlockID {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BlockID {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlockID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for BlockID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for BlockID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; BLOCK_ID_LENGTH * 2];
        let _ = hex_encode(self, &mut buf);
        write!(f, "{}", String::from_utf8_lossy(&buf))
    }
}

impl From<UBig> for BlockID {
    /// Converts from BlockID to BigInt.
    fn from(value: UBig) -> Self {
        let mut block_id: BlockID = BlockID::new();
        let int_bytes: [u8; BLOCK_ID_LENGTH] = value.to_be_bytes();

        if int_bytes.len() > 32 {
            panic!("Too much work")
        }

        block_id[32 - int_bytes.len()..].copy_from_slice(&int_bytes);
        block_id
    }
}

impl From<Vec<u8>> for BlockID {
    fn from(value: Vec<u8>) -> Self {
        BlockID(value.try_into().expect("incorrect bytes for block id"))
    }
}

impl From<&[u8]> for BlockID {
    fn from(value: &[u8]) -> Self {
        BlockID(value.try_into().expect("incorrect bytes for block id"))
    }
}

impl FromIterator<u8> for BlockID {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<u8>>().into()
    }
}

impl Serialize for BlockID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        faster_hex::nopfx_lowercase::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for BlockID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        faster_hex::nopfx_lowercase::deserialize(deserializer)
    }
}