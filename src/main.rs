use anyhow::{anyhow, Result};
use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value> {
    match encoded_value.chars().next().unwrap() {
        // If encoded_value starts with a digit, it's a number
        '0'..='9' => {
            // Example: "5:hello" -> "hello"
            let colon_index = encoded_value.find(':').unwrap();
            let number_string = &encoded_value[..colon_index];
            let number = number_string.parse::<i64>()?;
            let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
            return Ok(serde_json::Value::String(string.to_string()));
        }
        _ => {
            return Err(anyhow!("Unhandled encoded value: {}", encoded_value));
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()>{
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value)?;
            println!("{}", decoded_value.to_string());
        }
        _ => { println!("unknown command: {}", command) }
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bencoded_value() {
        assert_eq!(decode_bencoded_value("5:hello").unwrap(), serde_json::Value::String("hello".to_string()));
    }
}
