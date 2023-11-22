mod hashes;
mod peers;

use serde::{Deserialize, Serialize};
use hashes::Hashes;
use peers::Peers;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Files {
    Single { length: usize },
    Multiple { files: Vec<File> },
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
