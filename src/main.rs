use serde_json;
use std::env;
use std::collections::BTreeMap;
use serde_json::Map;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<(serde_json::Value, usize)> {
    match encoded_value.chars().next() {
        Some('i') => {
            let end = encoded_value.find('e').ok_or_else(|| anyhow::anyhow!("Invalid integer encoding"))?;
            let num = encoded_value[1..end].parse::<i64>()?;
            Ok((serde_json::Value::Number(num.into()), end + 1))
        }
        Some('l') => {
            let mut index = 1;
            let mut elements = Vec::new();
            while encoded_value.chars().nth(index) != Some('e') {
                let (element, len) = decode_bencoded_value(&encoded_value[index..])?;
                elements.push(element);
                index += len;
            }
            Ok((serde_json::Value::Array(elements), index + 1))
        }
        Some('d') => {
            let mut index = 1;
            let mut map = BTreeMap::new();
            while encoded_value.chars().nth(index) != Some('e') {
                let (key, key_len) = decode_bencoded_value(&encoded_value[index..])?;
                index += key_len;
                let (value, value_len) = decode_bencoded_value(&encoded_value[index..])?;
                index += value_len;
                if let serde_json::Value::String(key_str) = key {
                    map.insert(key_str, value);
                } else {
                    anyhow::bail!("Dictionary keys must be strings");
                }
            }
            let json_map: Map<String, serde_json::Value> = map.into_iter().collect();
            Ok((serde_json::Value::Object(json_map), index + 1))
        }
        Some(c) if c.is_ascii_digit() => {
            let colon_pos = encoded_value.find(':').ok_or_else(|| anyhow::anyhow!("Invalid string encoding"))?;
            let len: usize = encoded_value[..colon_pos].parse()?;
            let start = colon_pos + 1;
            let end = start + len;
            Ok((serde_json::Value::String(encoded_value[start..end].to_string()), end))
        }
        _ => anyhow::bail!("Invalid Bencode value"),
    }
}

fn convert(value: serde_bencode::value::Value) -> anyhow::Result<serde_json::Value> {
    match value {
        serde_bencode::value::Value::Bytes(b) => {
            let string = String::from_utf8(b)?;
            Ok(serde_json::Value::String(string))
        }
        serde_bencode::value::Value::Int(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i)))
        }
        serde_bencode::value::Value::List(l) => {
            let array = l
                .into_iter()
                .map(|item| convert(item))
                .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;
            Ok(serde_json::Value::Array(array))
        }
        _ => {
            panic!("Unhandled encoded value: {:?}", value)
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let (decoded_value, _) = decode_bencoded_value(encoded_value)?;
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
