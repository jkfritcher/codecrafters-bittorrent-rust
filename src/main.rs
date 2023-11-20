#[allow(unused_imports)]
use anyhow::{anyhow, Result};
use sha1::Digest;
use std::{env, fs, io::{Read, Write}, net::{SocketAddrV4, TcpStream}};

mod decoder;
mod types;

use crate::decoder::decode_bencoded_value;
use crate::types::{Files, Info, Torrent, TrackerResponse};

fn calculate_info_hash(info: &Info) -> Result<[u8; 20]> {
    let encoded_info = serde_bencode::to_bytes(info)?;
    let mut hasher = sha1::Sha1::new();
    hasher.update(&encoded_info);
    Ok(hasher.finalize().into())
}

fn urlencode_info_hash(info_hash: &[u8; 20]) -> String {
    let mut escaped_info_hash = String::with_capacity(info_hash.len() * 3);
    for byte in info_hash {
        escaped_info_hash.push_str(format!("%{:02X}", byte).as_str());
    }
    escaped_info_hash
}

fn get_peers_from_tracker(announce_url: String, info_hash: &[u8; 20], left: usize) -> Result<Vec<SocketAddrV4>> {
    // Build url with query parameters
    let mut url = announce_url;
    if url.contains('?') {
        url.push('&');
    } else {
        url.push('?');
    }
    url.push_str(format!("info_hash={}", urlencode_info_hash(info_hash)).as_str());
    url.push_str("&peer_id=00112233445566778899");
    url.push_str("&port=6881");
    url.push_str("&uploaded=0");
    url.push_str("&downloaded=0");
    url.push_str(format!("&left={}", left).as_str());
    url.push_str("&compact=1");

    let response = reqwest::blocking::get(&url)?.bytes()?;
    let response: TrackerResponse = serde_bencode::from_bytes(&response)?;

    Ok(response.peers.0)
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
        "peers" => {
            let torrent_name = &args[2];
            let encoded_value = fs::read(torrent_name)?;
            let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
            let info_hash = calculate_info_hash(&torrent.info)?;
            let left = match torrent.info.files {
                Files::Single { length } => length,
                Files::Multiple { files } => files.iter().map(|file| file.length).sum(),
            };
            let peers = get_peers_from_tracker(torrent.announce, &info_hash, left)?;
            for peer in peers {
                println!("{}:{}", peer.ip(), peer.port());
            }
        }
        "handshake" => {
            let torrent_name = &args[2];
            let peer_addr = &args[3];

            let encoded_value = fs::read(torrent_name)?;
            let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
            let info_hash = calculate_info_hash(&torrent.info)?;

            let mut stream = TcpStream::connect(peer_addr)?;
            stream.write_all(&[19])?;
            stream.write_all(b"BitTorrent protocol")?;
            stream.write_all(&[0; 8])?;
            stream.write_all(&info_hash)?;
            stream.write_all(b"01234567890123456789")?;
            let mut handshake = [0; 68];
            stream.read_exact(&mut handshake)?;
            if info_hash != handshake[28..48] {
                return Err(anyhow!("Peer sent wrong info hash"));
            }
            println!("Peer ID: {}", hex::encode(&handshake[48..68]));
        }
        "download_piece" => {
            unimplemented!();
        }
        "download" => {
            unimplemented!();
        }
        _ => { println!("unknown command: {}", command) }
    }
    Ok(())
}
