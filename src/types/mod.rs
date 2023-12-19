use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::Digest;

mod hashes;
mod peers;
use self::{hashes::Hashes, peers::Peers};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
    #[serde(flatten)]
    pub files: Files,
}

impl Info {
    pub fn calculate_info_hash(&self) -> Result<[u8; 20]> {
        let encoded_info = serde_bencode::to_bytes(self)?;
        let mut hasher = sha1::Sha1::new();
        hasher.update(&encoded_info);
        Ok(hasher.finalize().into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Files {
    Single { length: usize },
    Multiple { files: Vec<File> },
}

impl Files {
    pub fn length(&self) -> usize {
        match self {
            Files::Single { length } => *length,
            Files::Multiple { files } => files.iter().map(|file| file.length).sum(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    pub interval: usize,
    pub peers: Peers,
}
