use anyhow::{anyhow, Result};
use std::env;

mod commands;
mod decoder;
mod protocol;
mod types;

use crate::commands::{
    cmd_decode, cmd_download, cmd_download_piece, cmd_handshake, cmd_info, cmd_peers
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow!("Usage: your_bittorrent.sh <command>"));
    }
    let command = &args[1];

    match command.as_str() {
        // Usage: your_bittorrent.sh decode "<encoded_value>"
        "decode" => {
            if args.len() != 3 {
                return Err(anyhow!("Usage: your_bittorrent.sh decode \"<encoded_value>\""));
            }
            cmd_decode(&args[2])
        }
        // Usage: your_bittorrent.sh info <torrent_name>
        "info" => {
            if args.len() != 3 {
                return Err(anyhow!("Usage: your_bittorrent.sh info <torrent_name>"));
            }
            cmd_info(&args[2])
        }
        // Usage: your_bittorrent.sh peers <torrent_name>
        "peers" => {
            if args.len() != 3 {
                return Err(anyhow!("Usage: your_bittorrent.sh peers <torrent_name>"));
            }
            cmd_peers(&args[2])
        }
        // Usage: your_bittorrent.sh handshake <torrent_name> <peer_ip:peer_port>
        "handshake" => {
            if args.len() != 4 {
                return Err(anyhow!("Usage: your_bittorrent.sh handshake <torrent_name> <peer_ip:peer_port>"));
            }
            cmd_handshake(&args[2], &args[3])
        }
        // Usage: your_bittorrent.sh download_piece -o <output_file_name> <torrent_name> <piece_num>
        "download_piece" => {
            if args.len() != 6 && args[2] != "-o" {
                return Err(anyhow!("Usage: your_bittorrent.sh download_piece -o <output_file_name> <torrent_name> <piece_num>"));
            }
            cmd_download_piece(&args[3], &args[4], &args[5])
        }
        // Usage: your_bittorrent.sh download -o <output_file_name> <torrent_name>
        "download" => {
            if args.len() != 5 && args[2] != "-o" {
                return Err(anyhow!("Usage: your_bittorrent.sh download -o <output_file_name> <torrent_name>"));
            }
            cmd_download(&args[3], &args[4])
        }
        _ => Err(anyhow!("Unknown command: {}", command))
    }
}
