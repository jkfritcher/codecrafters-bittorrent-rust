use anyhow::{anyhow, Result};
use sha1::Digest;
use std::{io::{Read, Write}, net::{SocketAddrV4, TcpStream}};

use crate::types::TrackerResponse;

pub const CHUNK_LEN: u32 = 16_384;

pub fn urlencode_u8_slice(slice: &[u8]) -> String {
    let mut escaped_slice = String::with_capacity(slice.len() * 3);
    for byte in slice {
        escaped_slice.push_str(format!("%{:02X}", byte).as_str());
    }
    escaped_slice
}

pub fn get_peers_from_tracker(announce_url: String, info_hash: &[u8; 20], left: usize) -> Result<Vec<SocketAddrV4>> {
    // Build url with query parameters
    let mut url = announce_url;
    if url.contains('?') {
        url.push('&');
    } else {
        url.push('?');
    }
    url.push_str(format!("info_hash={}", urlencode_u8_slice(info_hash)).as_str());
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

pub fn perform_handshake_with_peer(stream: &mut TcpStream, info_hash: &[u8; 20]) -> Result<Vec<u8>> {
    stream.write_all(&[19])?;
    stream.write_all(b"BitTorrent protocol")?;
    stream.write_all(&[0; 8])?;
    stream.write_all(info_hash)?;
    stream.write_all(b"01234567890123456789")?;
    let mut handshake = [0; 68];
    stream.read_exact(&mut handshake)?;
    if info_hash != &handshake[28..48] {
        return Err(anyhow!("Peer sent wrong info hash"));
    }

    Ok(handshake[48..68].to_vec())
}

pub fn wait_for_bitfield(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    let len = u32::from_be_bytes(buf) as usize;
    if len == 0 {
        return Err(anyhow!("Peer closed connection"));
    }
    let mut buf = vec!(0u8; len);
    stream.read_exact(&mut buf)?;
    if buf[0] != 5 {
        return Err(anyhow!("Expected bitfield, got message with id {}", buf[4]));
    }
    Ok(buf[1..].to_vec())
}

pub fn send_am_interested(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0u8; 5];
    buf[0..4].copy_from_slice(1u32.to_be_bytes().as_ref());
    buf[4] = 2;
    stream.write_all(&buf)?;
    Ok(())
}

pub fn wait_for_unchoke(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    let len = u32::from_be_bytes(buf) as usize;
    if len == 0 {
        return Err(anyhow!("Peer closed connection"));
    }
    let mut buf = vec!(0u8; len);
    stream.read_exact(&mut buf)?;
    if buf[0] != 1 {
        return Err(anyhow!("Expected unchoke, got message with id {}", buf[4]));
    }
    Ok(())
}

pub fn download_piece(stream: &mut TcpStream, piece_index: u32, piece_length: u32, piece_hash: &[u8; 20]) -> Result<Vec<u8>> {
    let mut piece: Vec<u8> = vec![0u8; piece_length as usize];
    let whole_chunks: u32 = piece_length / CHUNK_LEN;
    let last_chunk_len: u32 = piece_length % CHUNK_LEN;

    let mut buf = [0u8; 17];
    // Static portion of the request buffer
    buf[0..4].copy_from_slice(13u32.to_be_bytes().as_ref());
    buf[4] = 6;
    buf[5..9].copy_from_slice(piece_index.to_be_bytes().as_ref());

    // Send chunk requests
    if whole_chunks > 0 {
        for i in 0..whole_chunks {
            buf[9..13].copy_from_slice((i * CHUNK_LEN).to_be_bytes().as_ref());
            buf[13..17].copy_from_slice(CHUNK_LEN.to_be_bytes().as_ref());
            stream.write_all(&buf)?;
        }
    }
    if last_chunk_len > 0 {
        buf[9..13].copy_from_slice((whole_chunks * CHUNK_LEN).to_be_bytes().as_ref());
        buf[13..17].copy_from_slice(last_chunk_len.to_be_bytes().as_ref());
        stream.write_all(&buf)?;
    }

    // Receive chunks
    let chunks_to_receive = if last_chunk_len > 0 { whole_chunks + 1 } else { whole_chunks };
    let mut recv_buf: Vec<u8> = vec![0u8; (CHUNK_LEN + 9) as usize];
    for _ in 0..chunks_to_receive {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        let len = u32::from_be_bytes(buf);
        if len == 0 {
            return Err(anyhow!("Peer closed connection"));
        }
        stream.read_exact(&mut recv_buf[0..(len as usize)])?;
        if recv_buf[0] != 7 {
            return Err(anyhow!("Expected piece, got message with id {}", recv_buf[0]));
        }
        if u32::from_be_bytes([recv_buf[1], recv_buf[2], recv_buf[3], recv_buf[4]]) != piece_index as u32 {
            return Err(anyhow!("Expected piece with index {}, got {}", piece_index, u32::from_be_bytes([recv_buf[1], recv_buf[2], recv_buf[3], recv_buf[4]])));
        }
        let chunk_len = len - 9;
        if chunk_len > CHUNK_LEN {
            return Err(anyhow!("Received chunk with length {}, but max chunk length is {}", chunk_len, CHUNK_LEN));
        }
        let chunk_index = u32::from_be_bytes([recv_buf[5], recv_buf[6], recv_buf[7], recv_buf[8]]);
        if chunk_index % CHUNK_LEN != 0 {
            return Err(anyhow!("Expected chunk with index {} to be a multiple of {}, but it's not", chunk_index, CHUNK_LEN));
        }
        if chunk_index + chunk_len > piece_length {
            return Err(anyhow!("Expected chunk with index {} and length {} to fit in piece of length {}", chunk_index, chunk_len, piece_length));
        }
        if chunk_len < CHUNK_LEN && (chunk_index + chunk_len) != piece_length {
            return Err(anyhow!("Received chunk with length {}, but it's not the last chunk", chunk_len));
        }

        let chunk_len = chunk_len as usize;
        let offset = chunk_index as usize;
        piece[offset..offset+chunk_len].copy_from_slice(&recv_buf[9..9+chunk_len]);
    }

    // Verify piece hash
    let mut hasher = sha1::Sha1::new();
    hasher.update(&piece);
    let new_piece_hash = hasher.finalize();
    if new_piece_hash.as_slice() != piece_hash {
        return Err(anyhow!("Piece hash mismatch"));
    }

    Ok(piece)
}
