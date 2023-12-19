use anyhow::{anyhow, Result};
use std::{cmp, fs, io::Write, net::TcpStream};

use crate::{
    decoder::decode_bencoded_value,
    protocol::{download_piece, get_peers_from_tracker, perform_handshake_with_peer, wait_for_bitfield, send_am_interested, wait_for_unchoke},
    types::{Files, Torrent},
};

pub fn cmd_decode(encoded_value: &str) -> Result<()> {
    let decoded_value;
    (decoded_value, _) = decode_bencoded_value(encoded_value)?;
    println!("{}", decoded_value);
    Ok(())
}

pub fn cmd_info(torrent_name: &str) -> Result<()> {
    let encoded_value = fs::read(torrent_name)?;
    let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.files.length());
    let info_hash = torrent.info.calculate_info_hash()?;
    println!("Info Hash: {}", hex::encode(info_hash));
    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    for hash in torrent.info.pieces.0 {
        println!("{}", hex::encode(hash));
    }
    Ok(())
}

pub fn cmd_peers(torrent_name: &str) -> Result<()> {
    let encoded_value = fs::read(torrent_name)?;
    let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
    let info_hash = torrent.info.calculate_info_hash()?;
    let left = match torrent.info.files {
        Files::Single { length } => length,
        Files::Multiple { files } => files.iter().map(|file| file.length).sum(),
    };
    let peers = get_peers_from_tracker(torrent.announce, &info_hash, left)?;
    for peer in peers {
        println!("{}:{}", peer.ip(), peer.port());
    }
    Ok(())
}

pub fn cmd_handshake(torrent_name: &str, peer_addr: &str) -> Result<()> {
    let encoded_value = fs::read(torrent_name)?;
    let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
    let info_hash = torrent.info.calculate_info_hash()?;

    let mut stream = TcpStream::connect(peer_addr)?;
    let peer_id = perform_handshake_with_peer(&mut stream, &info_hash)?;
    println!("Peer ID: {}", hex::encode(&peer_id));
    Ok(())
}

pub fn cmd_download_piece(output_name: &str, torrent_name: &str, piece_num: &str) -> Result<()> {
    let encoded_value = fs::read(torrent_name)?;
    let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
    let info_hash = torrent.info.calculate_info_hash()?;
    let left = torrent.info.files.length();

    let peers = get_peers_from_tracker(torrent.announce, &info_hash, left)?;
    let peer = peers[0];
    let mut stream = TcpStream::connect(peer)?;
    let _ = perform_handshake_with_peer(&mut stream, &info_hash)?;

    let bitfield = wait_for_bitfield(&mut stream)?;
    let bitfield_bytes = (torrent.info.pieces.0.len() + 7) / 8;
    if bitfield_bytes != bitfield.len() {
        return Err(anyhow!(
            "Expected bitfield of length {}, got {}", bitfield_bytes, bitfield.len()
        ));
    }
    eprintln!("Bitfield: {}", hex::encode(&bitfield));

    send_am_interested(&mut stream)?;

    wait_for_unchoke(&mut stream)?;

    let piece_index = piece_num.parse::<u32>()?;
    let piece_length = (torrent.info.piece_length) as u32;
    let piece_length = cmp::min(piece_length, (torrent.info.files.length() as u32) - piece_index * piece_length);
    let expected_piece_hash = &torrent.info.pieces.0[piece_index as usize];
    let piece_data = download_piece(&mut stream, piece_index, piece_length, expected_piece_hash)?;

    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).open(output_name)?;
    file.write_all(&piece_data)?;
    file.sync_all()?;

    Ok(())
}

#[allow(unused_variables)]
pub fn cmd_download(output_name: &str, torrent_name: &str) -> Result<()> {
    let encoded_value = fs::read(torrent_name)?;
    let torrent: Torrent = serde_bencode::from_bytes(&encoded_value)?;
    let info_hash = torrent.info.calculate_info_hash()?;
    let left = torrent.info.files.length();

    let peers = get_peers_from_tracker(torrent.announce, &info_hash, left)?;
    let peer = peers[0];
    let mut stream = TcpStream::connect(peer)?;
    let _ = perform_handshake_with_peer(&mut stream, &info_hash)?;

    let bitfield = wait_for_bitfield(&mut stream)?;
    let bitfield_bytes = (torrent.info.pieces.0.len() + 7) / 8;
    if bitfield_bytes != bitfield.len() {
        return Err(anyhow!(
            "Expected bitfield of length {}, got {}", bitfield_bytes, bitfield.len()
        ));
    }
    eprintln!("Bitfield: {}", hex::encode(&bitfield));

    send_am_interested(&mut stream)?;

    wait_for_unchoke(&mut stream)?;

    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).open(output_name)?;
    let num_pieces = torrent.info.pieces.0.len();
    for i in 0..num_pieces {
        let piece_index = i as u32;
        let piece_length = (torrent.info.piece_length) as u32;
        let piece_length = cmp::min(piece_length, (torrent.info.files.length() as u32) - piece_index * piece_length);
        let expected_piece_hash = &torrent.info.pieces.0[piece_index as usize];
        let piece_data = download_piece(&mut stream, piece_index, piece_length, expected_piece_hash)?;

        file.write_all(&piece_data)?;
    }
    file.sync_all()?;

    Ok(())
}
