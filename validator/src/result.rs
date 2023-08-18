use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrieResult {
  pub success: bool,
  pub result: Option<String>,
}

impl From<String> for TrieResult {
  fn from(result: String) -> Self {

    let raw_data: String = serde_json::from_str(&result).unwrap();

    serde_json::from_str(&raw_data).unwrap_or(TrieResult { success: false, result: None })
  }
}