use std::process::Command;
use crate::appconfig::CONFIG;
use sha2::{Digest, Sha256};

pub fn bash_cmd(key: &str, args: Vec<String>) -> String {
  // println!("args: {:?} {:?}", args, key);
  let output = Command::new(key)
              .args(args)
              .output()
              .expect("Failed to execute the curl command");

  // println!("output: {:?}", output);
  let mut result;
  if output.status.success() {
      // Convert the output bytes to a string
      let response_body = String::from_utf8_lossy(&output.stdout);

      // Print the response body
      // println!("Response body:\n{}", response_body);
      result = response_body;
  } else {
      // If the command failed, print the error message
      let error_message = String::from_utf8_lossy(&output.stderr);
      // println!("Error executing the curl command:\n{}", error_message);
      result = error_message;
  }

  trimmer(result.to_string())
}

pub fn trimmer(text: String) -> String {
  text.replace("\n", "")  // Remove newline characters
  .trim_start()       // Trim leading whitespace
  .trim_end()
  .to_string()
}

pub fn hasher(content: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}", content).as_bytes());

    bs58::encode(hasher.finalize()).into_string()
}