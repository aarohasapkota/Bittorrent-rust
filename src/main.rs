use anyhow::anyhow;
use serde_json;

use std::{collections::HashMap, env, path::PathBuf};
// Available if you need it!
// use serde_bencode
#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value)?;
    convert(value)
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
        serde_bencode::value::Value::Dict(d) => {
            let object = d
                .into_iter()
                .map(|(k, v)| {
                    let key = String::from_utf8(k)?;
                    let value = convert(v)?;
                    Ok((key, value))
                })
                .collect::<anyhow::Result<serde_json::Map<String, serde_json::Value>>>()?;
            Ok(serde_json::Value::Object(object))
        }
    }
}
// announce:
// URL to a "tracker", which is a central server that keeps track of peers participating in the sharing of a torrent.
// info:
// A dictionary with keys:
//     length: size of the file in bytes, for single-file torrents
//     name: suggested name to save the file / directory as
//     piece length: number of bytes in each piece
//     pieces: concatenated SHA-1 hashes of each piece
struct TorrentInfo {
    length: i64,
    name: String,
    piece_length: i64,
    pieces: Vec<u8>,
}
struct Torrent {
    announce: reqwest::Url,
    info: TorrentInfo,
}
fn extract_string(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<String> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => String::from_utf8(b.clone()).ok(),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}
fn extract_bytes(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<Vec<u8>> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => Some(b.clone()),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}
fn extract_dict(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<HashMap<Vec<u8>, serde_bencode::value::Value>> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Dict(d) => Some(d.clone()),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}
fn extract_int(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<i64> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Int(i) => Some(*i),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}
fn parse_torrent_file<T>(file_path: T) -> anyhow::Result<Torrent>
where
    T: Into<PathBuf>,
{
    let content = std::fs::read(file_path.into())?;
    let value: serde_bencode::value::Value = serde_bencode::from_bytes(content.as_slice())?;
    match value {
        serde_bencode::value::Value::Dict(d) => {
            let announce = extract_string("announce", &d)?;
            let info = extract_dict("info", &d)?;
            Ok(Torrent {
                announce: reqwest::Url::parse(announce.as_str())?,
                info: TorrentInfo {
                    length: extract_int("length", &info)?,
                    name: extract_string("name", &info)?,
                    piece_length: extract_int("piece length", &info)?,
                    pieces: extract_bytes("pieces", &info)?,
                },
            })
        }
        _ => Err(anyhow!("Incorrect format, required dict")),
    }
}
// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value)?;
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let file_name = &args[2];
        let torrent = parse_torrent_file(file_name)?;
        println!("Tracker URL: {}", torrent.announce);
        println!("Length: {}", torrent.info.length);
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}