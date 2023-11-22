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
