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

mod hashes {
    use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct Hashes(pub Vec<[u8; 20]>);
    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice = self.0.concat();
            serializer.serialize_bytes(&slice)
        }
    }

    struct HashesVisitor;
    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string with a length divisible by 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!(
                    "expected a byte string with a length divisible by 20, got {}",
                    v.len()
                )));
            }
            Ok(Hashes(
                v.chunks(20)
                    .map(|chunk| {
                        chunk.try_into().expect("chunk is always 20 bytes")
                    })
                    .collect(),
            ))
        }
    }
}

mod peers {
    use serde::{Deserialize, Deserializer, de::Visitor};
    use std::{fmt, net::{Ipv4Addr, SocketAddrV4}};

    #[derive(Debug, Clone)]
    pub struct Peers(pub Vec<SocketAddrV4>);
    impl<'de> Deserialize<'de> for Peers {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(PeersVisitor)
        }
    }

    struct PeersVisitor;
    impl<'de> Visitor<'de> for PeersVisitor {
        type Value = Peers;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string with a length divisible by 6")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.len() % 6 != 0 {
                return Err(E::custom(format!(
                    "expected a byte string with a length divisible by 6, got {}",
                    v.len()
                )));
            }
            Ok(Peers(
                v.chunks(6)
                    .map(|chunk| {
                        let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                        SocketAddrV4::new(ip, port)
                    })
                    .collect(),
            ))
        }
    }
}
