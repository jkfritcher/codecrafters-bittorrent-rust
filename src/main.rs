use anyhow::{anyhow, Result};
use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> Result<(serde_json::Value, &str)> {
    match encoded_value.chars().next() {
        // If encoded_value starts with a digit, it's a string
        Some('0'..='9') => {
            // Example: "5:hello" -> "hello"
            let colon_index = encoded_value.find(':').ok_or_else(|| anyhow!("No colon found"))?;
            let len = (&encoded_value[..colon_index]).parse::<usize>()?;
            let string = &encoded_value[colon_index + 1..colon_index + 1 + len];
            return Ok((serde_json::Value::String(string.to_string()), &encoded_value[colon_index + 1 + len..]));
        }
        // If encoded_value starts with an 'i', it's a number
        Some('i') => {
            // Example: "i42e" -> 42
            let end_index = encoded_value.find('e').ok_or_else(|| anyhow!("No end found"))?;
            let number = (&encoded_value[1..end_index]).parse::<i64>()?;
            return Ok((serde_json::Value::Number(number.into()), &encoded_value[end_index + 1..]));
        }
        // If encoded_value starts with an 'l', it's a list
        Some('l') => {
            // Example: "l5:helloi42ee" -> ["hello", 42]
            let mut encoded_value = &encoded_value[1..];
            let mut list = Vec::new();
            let mut decoded_value;
            while encoded_value.chars().next() != Some('e') {
                (decoded_value, encoded_value) = decode_bencoded_value(encoded_value)?;
                list.push(decoded_value);
            }
            return Ok((serde_json::Value::Array(list), &encoded_value[1..]));
        }
        // If encoded_value starts with a 'd', it's a dict
        Some('d') => {
            // Example: "d3:cow3:moo4:spam4:eggse" -> {"cow": "moo", "spam": "eggs"}
            let mut encoded_value = &encoded_value[1..];
            let mut dict = serde_json::Map::new();
            let mut key;
            let mut decoded_value;
            while encoded_value.chars().next() != Some('e') {
                (key, encoded_value) = decode_bencoded_value(encoded_value)?;
                (decoded_value, encoded_value) = decode_bencoded_value(encoded_value)?;
                dict.insert(key.as_str().unwrap().to_string(), decoded_value);
            }
            return Ok((serde_json::Value::Object(dict), &encoded_value[1..]));
        }
        Some(_) => {
            return Err(anyhow!("Unhandled encoded value: {}", encoded_value));
        }
        None => {
            return Err(anyhow!("Empty encoded value"));
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
            let decoded_value = decode_bencoded_value(encoded_value)?.0;
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
        assert_eq!(decode_bencoded_value("5:hello").unwrap().0, serde_json::Value::String("hello".to_string()));
    }

    #[test]
    fn test_decode_bencoded_value_with_number() {
        assert_eq!(decode_bencoded_value("i42e").unwrap().0, serde_json::Value::Number(serde_json::Number::from(42)));
    }

    #[test]
    fn test_decode_bencoded_value_with_negative_number() {
        assert_eq!(decode_bencoded_value("i-42e").unwrap().0, serde_json::Value::Number(serde_json::Number::from(-42)));
    }

    #[test]
    fn test_decode_bencoded_value_with_empty_list() {
        assert_eq!(decode_bencoded_value("le").unwrap().0, serde_json::Value::Array(vec![]));
    }

    #[test]
    fn test_decode_bencoded_value_with_list() {
        assert_eq!(decode_bencoded_value("l5:helloi42ee").unwrap().0, serde_json::Value::Array(vec![serde_json::Value::String("hello".to_string()), serde_json::Value::Number(serde_json::Number::from(42))]));
    }

    #[test]
    fn test_decode_bencoded_value_with_nested_list() {
        assert_eq!(decode_bencoded_value("l5:helloi42el5:worldi-42eee").unwrap().0, serde_json::Value::Array(vec![serde_json::Value::String("hello".to_string()), serde_json::Value::Number(serde_json::Number::from(42)), serde_json::Value::Array(vec![serde_json::Value::String("world".to_string()), serde_json::Value::Number(serde_json::Number::from(-42))])]));
    }

    #[test]
    fn test_decode_bencoded_value_with_empty_dict() {
        let dict = serde_json::Map::new();
        assert_eq!(decode_bencoded_value("de").unwrap().0, serde_json::Value::Object(dict));
    }

    #[test]
    fn test_decode_bencoded_value_with_dict() {
        let mut dict = serde_json::Map::new();
        dict.insert("key1".to_string(), serde_json::Value::String("val1".to_string()));
        dict.insert("key2".to_string(), serde_json::Value::String("val2".to_string()));
        assert_eq!(decode_bencoded_value("d4:key14:val14:key24:val2e").unwrap().0, serde_json::Value::Object(dict));
    }
}
