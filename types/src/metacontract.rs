
use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::FinalMetadata;

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct MetaContract {
  pub program_id: String,
  pub public_key: String,
  pub cid: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TxContract {
    // pub program_id: String,
    pub cid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaContractResult {
  pub result: bool,
  pub metadatas: Vec<FinalMetadata>,
  pub error_string: String,
}