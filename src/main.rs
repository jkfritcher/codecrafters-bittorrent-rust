#[allow(unused_imports)]
use anyhow::{anyhow, Result};
use serde_bencode;
use std::{env, fs};

mod decoder;
mod types;

use crate::decoder::decode_bencoded_value;
use crate::types::{Files, Torrent};


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
        }
        _ => { println!("unknown command: {}", command) }
    }
    return Ok(());
}
