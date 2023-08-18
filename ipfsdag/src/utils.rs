use std::{process::Command};
use eyre::Result;

pub fn bash_cmd(key: &str, args: Vec<String>) -> Result<String> {
  let output = Command::new(key)
              .args(args)
              .output();

  // println!("output: {:?}", output);

  let result: String = match output {
    Ok(data) => {
        if data.status.success() {
            trimmer(String::from_utf8(data.stdout).unwrap())
        } else {
            trimmer(String::from_utf8(data.stderr).unwrap())
        }
    },
    Err(e) => return Err(e.into()), // Convert the error to eyre::Error
  };

  Ok(result)
}

pub fn trimmer(text: String) -> String {
  text.replace("\n", "")  // Remove newline characters
  .trim_start()       // Trim leading whitespace
  .trim_end()
  .to_string()
}

pub fn is_base64(input: &str) -> bool {
  // Attempt to decode the input string
  match base64::decode(input) {
      Ok(_) => true,   // The string is Base64 encoded
      Err(_) => false, // The string is not Base64 encoded
  }
}