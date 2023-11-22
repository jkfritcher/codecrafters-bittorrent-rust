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
