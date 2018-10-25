//! Tendermint blockchain identifiers

#[cfg(feature = "serializers")]
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    str::{self, FromStr},
};

use error::Error;

/// Maximum length of a `chain::Id` name. Matches `MaxChainIDLen` from:
/// <https://github.com/tendermint/tendermint/blob/develop/types/genesis.go>
// TODO: update this when `chain::Id` is derived from a digest output
pub const MAX_LENGTH: usize = 50;

/// Chain identifier (e.g. 'gaia-9000')
#[derive(Copy, Clone)]
pub struct Id([u8; MAX_LENGTH]);

impl Id {
    /// Create a new chain ID
    pub fn new(name: &str) -> Result<Self, Error> {
        if name.is_empty() || name.len() > MAX_LENGTH {
            return Err(Error::Length);
        }

        for byte in name.as_bytes() {
            match byte {
                b'a'...b'z' | b'0'...b'9' | b'-' | b'_' => (),
                _ => return Err(Error::Parse),
            }
        }

        let mut bytes = [0u8; MAX_LENGTH];
        bytes[..name.as_bytes().len()].copy_from_slice(name.as_bytes());
        Ok(Id(bytes))
    }

    /// Get the chain ID as a `str`
    pub fn as_str(&self) -> &str {
        let byte_slice = match self.0.as_ref().iter().position(|b| *b == b'\0') {
            Some(pos) => &self.0[..pos],
            None => self.0.as_ref(),
        };

        // We assert above the ID only has characters in the valid UTF-8 range,
        // so in theory this should never panic
        str::from_utf8(byte_slice).unwrap()
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "chain::Id({})", self.as_str())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> From<&'a str> for Id {
    fn from(s: &str) -> Id {
        Self::from_str(s).unwrap()
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::new(s)
    }
}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Id) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Id {}

#[cfg(feature = "serializers")]
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature = "serializers")]
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

/// Parse `chain::Id` from a type
pub trait ParseId {
    /// Parse `chain::Id`, or return an `Error` if parsing failed
    fn parse_chain_id(&self) -> Result<Id, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_CHAIN_ID: &str = "gaia-9000";

    #[test]
    fn parses_valid_chain_ids() {
        assert_eq!(
            Id::new(EXAMPLE_CHAIN_ID).unwrap().as_str(),
            EXAMPLE_CHAIN_ID
        );

        let long_id = String::from_utf8(vec![b'x'; MAX_LENGTH]).unwrap();
        assert_eq!(Id::new(&long_id).unwrap().as_str(), &long_id);
    }

    #[test]
    fn rejects_empty_chain_ids() {
        assert_eq!(Id::new(""), Err(Error::Length))
    }

    #[test]
    fn rejects_overlength_chain_ids() {
        let overlong_id = String::from_utf8(vec![b'x'; MAX_LENGTH + 1]).unwrap();
        assert_eq!(Id::new(&overlong_id), Err(Error::Length))
    }
}
