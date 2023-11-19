#[allow(unused_imports)]
use anyhow::{anyhow, Result};
use serde_bencode;
use sha1::Digest;
use std::{env, fs};

mod decoder;
mod types;

use crate::decoder::decode_bencoded_value;
use crate::types::{Files, Info, Torrent};

fn calculate_info_hash(info: &Info) -> Result<[u8; 20]> {
    let encoded_info = serde_bencode::to_bytes(info)?;
    let mut hasher = sha1::Sha1::new();
    hasher.update(&encoded_info);
    return Ok(hasher.finalize().into());
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        // Usage: your_bittorrent.sh decode "<encoded_value>"
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value;
            (decoded_value, _) = decode_bencoded_value(encoded_value)?;
            println!("{}", decoded_value);
        }
        // Usage: your_bittorrent.sh info <torrent_name>
        "info" => {
            let torrent_name = &args[2];
            let encoded_value = fs::read(torrent_name)?;
            let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
            println!("Tracker URL: {}", torrent.announce);
            if let Files::Single { length } = torrent.info.files {
                println!("Length: {}", length);
            }
            let info_hash = calculate_info_hash(&torrent.info)?;
            println!("Info Hash: {}", hex::encode(info_hash));
            println!("Piece Length: {}", torrent.info.piece_length);
            println!("Piece Hashes:");
            for hash in torrent.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
        _ => { println!("unknown command: {}", command) }
    }
    return Ok(());
}
