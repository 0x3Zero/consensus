use std::{env, path::PathBuf, fs};

use anyhow::{Result, Error};
use appconfig::CONFIG;
use ipfsdag::{put_ipld, get_ipld, put_contract, get_contract};
use result::TrieResult;
use serde_json::json;
use types::*;
use utils::{bash_cmd, hasher};

use crate::wasirunner::wasi_runner;

mod appconfig;
mod utils;
mod result;
mod wasirunner;

fn main() -> Result<()> {
  let args: Vec<String> = env::args().collect();
  let method = &args[1];

  let result = match method.as_str() {
    "validate_contract" => validate_contract(),
    "validate_metadata" => validate_metadata(),
    _ => TrieResult { success: false, result: None },
  };
  // println!("result: {:?}", result.result.unwrap());
  println!("{:?}", serde_json::to_string(&result).unwrap_or("".to_string()));

  Ok(())
}

fn validate_contract() -> TrieResult {
  let args: Vec<String> = env::args().collect();
  let arg_tx = &args[2];

  let mut success = false;
  let mut result = None;

  let dec_tx: Result<Transaction, serde_json::Error> = serde_json::from_str(&arg_tx);

  match dec_tx {
    Ok(tx) => {
      // let contract_result = put_contract(
      //   tx.data.clone(), 
      //   CONFIG.get::<String>("IPFS_ADDR").unwrap(),
      //   CONFIG.get::<u64>("IPFS_TIMEOUT_SEC").unwrap(),
      // );

      let mut status;

      let tx_contract: Result<TxContract, serde_json::Error> = serde_json::from_str(&tx.data.clone());

      match tx_contract {
        Ok(data) => {
          let program_id = tx.program_id.clone();
          let cid = data.cid.clone();


          let mc = MetaContract {
            program_id: program_id,
            public_key: tx.public_key.clone(),
            cid: cid,
          };

          let args = vec![
            "insert_trie".to_string(),
            CONFIG.get::<String>("METACONTRACT_KEY").unwrap().to_string(),
            serde_json::to_string(&mc).unwrap(),
          ];

          let rst = bash_cmd(&CONFIG.get::<String>("WORLD_STATE").unwrap(), args);
          println!("tx_contract: {:?}", rst);

          success = true;
          status = CONFIG.get::<String>("TX_STATUS_SUCCESS").unwrap();

          result = Some(arg_tx.to_string());
        },
        Err(e) => {
          success = false;
          result = Some(e.to_string());
          status = CONFIG.get::<String>("TX_STATUS_FAILED").unwrap();
        }
      }

      let mut tx_args = vec![
        "update_tx_status".to_string(),
        tx.hash.clone(),
        status,
      ];

      if !success {
        tx_args.push(result.clone().unwrap());
      }

      bash_cmd(&CONFIG.get::<String>("WORLD_STATE").unwrap(), tx_args);
    },
    Err(e) => {
      success = false;
      result = Some(e.to_string());
    }
  }

  TrieResult { 
    success, 
    result, 
  }
}

fn validate_metadata() -> TrieResult {
  let args: Vec<String> = env::args().collect();
  let arg_tx = args[2].as_str();

  let mut success = false;
  let mut result = None;

  let dec_tx: Result<Transaction, serde_json::Error> = serde_json::from_str(&arg_tx);

  match dec_tx {
    Ok(tx) => {
      
      let filter_result = filter_trie(CONFIG.get::<String>("METACONTRACT_KEY").unwrap(), tx.program_id.clone(), None);

      if filter_result.success {
        let metadata: Result<Vec<MetaContract>, _> = serde_json::from_str(&filter_result.result.unwrap());

        match metadata {
          Ok(mc) => {
            let d = mc.get(0).unwrap().clone();
            println!("mc: {:?}", d);

            let filter_key = format!("{}{}", tx.data_key.clone(), hasher(tx.version.clone()));
            let metadata_key = Metadata::generate_hash(
                tx.data_key.clone(), 
                hasher(tx.version.clone()), 
                hasher(tx.alias.clone()), 
                tx.public_key.clone(),
              );

            let metadata_result = filter_trie(
              CONFIG.get::<String>("METADATA_KEY").unwrap(),
              filter_key,
              None,
            );

            let mut new_metadata: Vec<MetadataRequest> = Vec::new();
            if metadata_result.success {
              let serde_metadata: Vec<Metadata> = serde_json::from_str(&metadata_result.result.unwrap()).unwrap();
              new_metadata = serde_metadata
                              .into_iter()
                              .filter(|d| d.public_key.clone() == tx.public_key.clone())
                              .map(|d| d.into())
                              .collect();
            }
            
            let wasi_result = wasi_runner(
              "on_execute".to_string(), 
              d.cid.clone(), 
              new_metadata, 
              tx.clone(),
            );

            let wasi_serde: Result<MetaContractResult, serde_json::Error> = serde_json::from_str(&wasi_result);

            match wasi_serde {
              Ok(mcr) => {
                if mcr.result {
                  println!("metadatas: {:?}", mcr.metadatas);

                  for metadata in mcr.metadatas {

                    let tx_subset = TransactionSubset {
                      hash: tx.hash.clone(),
                      timestamp: tx.timestamp.clone(),
                      program_id: tx.program_id.clone(),
                      method: tx.method.clone(),
                      value: "".to_string(),
                    };
                    let tx_ss_serde = serde_json::to_string(&tx_subset).unwrap();

                    let result_ipfs = put_ipld(
                      metadata.content, 
                      "".to_string(), //previous cid 
                      tx_ss_serde, 
                      CONFIG.get::<String>("IPFS_ADDR").unwrap(), 
                      0
                    );

                    if result_ipfs.success {
                      let m = Metadata::new(
                        tx.data_key.clone(), 
                        tx.program_id.clone(), 
                        metadata.alias.clone(), 
                        result_ipfs.cid, 
                        metadata.public_key.clone(), 
                        tx.chain_id.clone(), 
                        tx.token_address.clone(), 
                        tx.token_id.clone(), 
                        tx.version.clone(), 
                        0,
                      );

                      let args = vec![
                        "insert_trie".to_string(),
                        CONFIG.get::<String>("METADATA_KEY").unwrap().to_string(),
                        serde_json::to_string(&m).unwrap(),
                      ];

                      let rst = bash_cmd(&CONFIG.get::<String>("WORLD_STATE").unwrap(), args);
                      println!("metadata: {:?}", rst);

                    } else {
                      result = Some(result_ipfs.error);
                    }
                    
                  }
                } else {
                  result = Some(mcr.error_string);
                }
              },
              Err(e) => result = Some(e.to_string()),
            }
            
            println!("metadata_key: {:?}", metadata_key);
          },
          Err(e) => result = Some(e.to_string()),
        }
      } else {
        result = Some("Invalid meta contract".to_string());
      }
    },
    Err(e) => {
      success = false;
      result = Some(e.to_string());
    }
  }

  TrieResult { 
    success, 
    result, 
  }
}

fn filter_trie(trie_key: String, filter_key: String, filters: Option<String>) -> TrieResult {
  let mut args = vec![
      "filter_trie".to_string(),
      trie_key,
      filter_key,
  ];

  if filters.is_some() {
    args.push(filters.unwrap());
  }
  
  let result = bash_cmd(&CONFIG.get::<String>("WORLD_STATE").unwrap(), args);

  result.into()
}

// fn update_tx_status(key: &str, status: &str, error_text: Option<String>) -> TrieResult {

// }

#[test]
fn test_put_ipld() {
  let new_data = json!({
    "test": "test".to_string(),
  });
  let result = put_ipld(
    "test".to_string(), 
    "".to_string(), 
    new_data.to_string(), 
    CONFIG.get::<String>("IPFS_ADDR").unwrap(), 
    CONFIG.get::<u64>("IPFS_TIMEOUT_SEC").unwrap());

  println!("result: {:?}", result);
}

#[test]
fn test_get_ipld() {
  let result = get_ipld(
    "bafyreifv4szmzbubunjwcnstzi2syjuwt4fupw74idbcwgz52vu625vomi".to_string(),
    CONFIG.get::<String>("IPFS_ADDR").unwrap(),
    CONFIG.get::<u64>("IPFS_TIMEOUT_SEC").unwrap(),
  );

  println!("result: {:?}", result);
}

#[test]
fn test_put_contract() {
  let content = json!({
    "test": "test".to_string(),
  }).to_string();

  let result = put_contract(
    content, 
    CONFIG.get::<String>("IPFS_ADDR").unwrap(),
    CONFIG.get::<u64>("IPFS_TIMEOUT_SEC").unwrap(),
  );

  println!("result: {:?}", result);
}

#[test]
fn test_get_contract() {
  let result = get_contract(
    "QmTxRuEnQDSgxELZk1QmHD6oT386JKkci4XM2cALmzL98G".to_string(),
    CONFIG.get::<String>("IPFS_ADDR").unwrap(),
    CONFIG.get::<u64>("IPFS_TIMEOUT_SEC").unwrap(),
  );

  println!("result: {:?}", result);

  fs::write(PathBuf::from(&"/tmp/vault/ori"), result.block).unwrap();
}

#[test]
fn test_tx_contract() {
  let t = "{\"program_id\":\"b2f2544b587c8b2f923bffea0407694098a4a1f4c3af46799e28b88cdbe783a4\",\"cid\":\"QmTxRuEnQDSgxELZk1QmHD6oT386JKkci4XM2cALmzL98G\"}";

  let result: TxContract = serde_json::from_str(t).unwrap();

  println!("result: {:?}", result);
}