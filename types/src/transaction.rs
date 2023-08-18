use serde::{Deserialize, Serialize};
use rlp::{encode, Decodable, Rlp};
use rlp_derive::{RlpEncodable, RlpDecodable};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct Transaction {
    pub hash: String,
    pub method: String,
    pub program_id: String,
    pub data_key: String,
    pub data: String,
    pub public_key: String,
    pub alias: String,
    pub timestamp: u64,
    pub chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub version: String,
    pub mcdata: String,
    pub status: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionSubset {
    pub hash: String,
    pub timestamp: u64,
    pub program_id: String,
    pub method: String,
    pub value: String,
}