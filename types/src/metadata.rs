use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct Metadata {
    pub hash: String,
    pub data_key: String,
    pub program_id: String,
    pub alias: String,
    pub chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub version: String,
    pub cid: String,
    pub public_key: String,
    pub loose: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinalMetadata {
  pub public_key: String,
  pub alias: String,
  pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataRequest {
  pub data_key: String,
  pub alias: String,
  pub cid: String,
  pub public_key: String,
}

impl From<Metadata> for MetadataRequest {
  fn from(data: Metadata) -> Self {
    
    Self { 
      data_key: data.data_key, 
      alias: data.alias, 
      cid: data.cid, 
      public_key: data.public_key
    }
  }
}

impl Metadata {
  pub fn new(
      data_key: String,
      program_id: String,
      alias: String,
      cid: String,
      public_key: String,
      chain_id: String,
      token_address: String,
      token_id: String,
      version: String,
      loose: u64,
  ) -> Self {
      let hash = Self::generate_hash(
          // chain_id.clone(),
          // token_address.clone(),
          // token_id.clone(),
          data_key.clone(),
          hasher(version.clone()),
          hasher(alias.clone()),
          public_key.clone(),
      );

      let data_key = Self::generate_data_key(
        chain_id.clone(), 
        token_address.clone(), 
        token_id.clone(),
      );

      Self {
          hash,
          data_key,
          program_id,
          alias,
          cid,
          chain_id,
          token_address,
          token_id,
          version,
          public_key,
          loose,
      }
  }
  pub fn generate_hash(
      // chain_id: String,
      // token_address: String,
      // token_id: String,
      data_key: String,
      version: String,
      alias: String,
      public_key: String,
  ) -> String {
      // let data_key = Self::generate_data_key(chain_id, token_address, token_id);

      format!("{}{}{}{}",
        data_key,
        version,
        alias,
        public_key,
      )
  }

  pub fn generate_data_key(
    chain_id: String,
    token_address: String,
    token_id: String,
  ) -> String {
    let mut hasher = Sha256::new();
    hasher.update(
        format!(
            "{}{}{}",
            chain_id, token_address, token_id,
        )
        .as_bytes(),
    );
    bs58::encode(hasher.finalize()).into_string()
  }
}

pub fn hasher(content: String) -> String {
  let mut hasher = Sha256::new();
  hasher.update(format!("{}", content).as_bytes());

  bs58::encode(hasher.finalize()).into_string()
}