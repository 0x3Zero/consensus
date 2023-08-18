mod block;
mod utils;

use std::{fs, path::PathBuf};

use types::*;

use crate::{utils::{bash_cmd, is_base64}, block::serialize};

const DEFAULT_TIMEOUT_SEC: u64 = 1u64;
const DEFAULT_IPFS_MULTIADDR: &str = "/ip4/127.0.0.1/tcp/5001";

fn make_cmd_args(args: Vec<String>, api_multiaddr: String, timeout_sec: u64) -> Vec<String> {
  args.into_iter()
      .chain(vec![
          String::from("--timeout"),
          get_timeout_string(timeout_sec),
          String::from("--api"),
          api_multiaddr,
      ])
      .collect()
}

#[inline]
fn get_timeout_string(timeout: u64) -> String {
    format!("{}s", timeout)
}

pub fn put_ipld(
  content: String,
  previous_cid: String,
  transaction: String,
  api_multiaddr: String,
  timeout_sec: u64,
) -> IpfsDagPutResult {
  let address: String;
  let t;

  if api_multiaddr.is_empty() {
      address = DEFAULT_IPFS_MULTIADDR.to_string();
  } else {
      address = api_multiaddr;
  }

  if timeout_sec == 0 {
      t = DEFAULT_TIMEOUT_SEC;
  } else {
      t = timeout_sec;
  }

  let block = serialize(content.clone(), previous_cid.clone(), transaction.clone());

  let input;

  if previous_cid.is_empty() {
      input = format!(
          r#"echo '{{"timestamp": {}, "content": {}, "previous": "{{}}", "transaction": {} }}' | ipfs dag put"#,
          block.timestamp, block.content, block.transaction
      );
  } else {
      input = format!(
          r#"echo '{{"timestamp": {}, "content": {}, "previous": {{"/": "{}" }}, "transaction": {} }}' | ipfs dag put"#,
          block.timestamp, block.content, previous_cid, block.transaction
      );
  }

  let args = make_cmd_args(vec![input], address, t);

  let cmd = vec![String::from("-c"), args.join(" ")];

  println!("ipfs put args : {:?}", cmd);

  bash_cmd("/bin/bash", cmd).into()
}

pub fn get_ipld(hash: String, api_multiaddr: String, timeout_sec: u64) -> IpfsDagGetResult {
  let address: String;
  let t;

  if api_multiaddr.is_empty() {
      address = DEFAULT_IPFS_MULTIADDR.to_string();
  } else {
      address = api_multiaddr;
  }

  if timeout_sec == 0 {
      t = DEFAULT_TIMEOUT_SEC;
  } else {
      t = timeout_sec;
  }

  println!("get called with hash {}", hash);

  let args = vec![String::from("dag"), String::from("get"), hash];

  let cmd = make_cmd_args(args, address, t);

  println!("ipfs dag get args {:?}", cmd);

  bash_cmd("ipfs", cmd).into()
}

pub fn put_contract(content: String, api_multiaddr: String, timeout_sec: u64) -> IpfsPutResult {
  let address;

  let t;

  if api_multiaddr.is_empty() {
      address = DEFAULT_IPFS_MULTIADDR.to_string();
  } else {
      address = api_multiaddr;
  }

  if timeout_sec == 0 {
      t = DEFAULT_TIMEOUT_SEC;
  } else {
      t = timeout_sec;
  }

  let file = "/tmp/vault/raw";

  let result: Result<_, _>;

  if is_base64(&content) {
      let decode_content = base64::decode(content.clone()).unwrap();
      result = fs::write(PathBuf::from(&file), decode_content);
  } else {
      result = fs::write(PathBuf::from(&file), content.clone());
  }

  if let Err(e) = result {
      println!("error: {:?}", e);
      return IpfsPutResult {
          success: false,
          error: format!("file can't be written: {}", e),
          cid: "".to_string(),
      };
  };

  let input = format!("ipfs add {}", "/tmp/vault/raw");

  let args = make_cmd_args(vec![input], address, t);

  let cmd = vec![String::from("-c"), args.join(" ")];

  println!("ipfs put args : {:?}", cmd);

  bash_cmd("/bin/bash", cmd).into()
}

pub fn get_contract(cid: String, api_multiaddr: String, timeout_sec: u64) -> IpfsGetResult {
  let address;

  let t;

  if api_multiaddr.is_empty() {
      address = DEFAULT_IPFS_MULTIADDR.to_string();
  } else {
      address = api_multiaddr;
  }

  if timeout_sec == 0 {
      t = DEFAULT_TIMEOUT_SEC;
  } else {
      t = timeout_sec;
  }

  let file = format!("/tmp/vault/{}", cid);
  let input = vec![
   "get".to_string(), 
   cid.to_string(),
   "-o".to_string(),
   file.clone(),
  ];
    // ("get {} -o {}", cid.to_string(), file);

  let args = make_cmd_args(input, address, t);

  // let cmd = vec![args.join(" ")];

  // println!("ipfs get args : {:?}", args);

  let result_mounted_binary = bash_cmd("ipfs", args);

  match result_mounted_binary {
    Ok(data) => {
      // println!("data: {:?}", data);
      let raw: Vec<u8> = fs::read(file).expect("No such file or directory");
      IpfsGetResult {
        success: true,
        error: "".to_string(),
        block: raw,
      }
    },
    Err(e) => {
      IpfsGetResult {
        success: false,
        error: e.to_string(),
        block: Vec::new(),
      }
    },
  }
}