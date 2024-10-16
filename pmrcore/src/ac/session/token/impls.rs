use serde::{
    de,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use std::{
    str::FromStr,
    fmt,
};
use crate::error::ValueError;
use super::*;

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SessionToken")
            .field(&self.to_string())
            .finish()
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl FromStr for SessionToken {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            (s.len() == 32)
                .then(|| u128::from_str_radix(s, 16).ok())
                .flatten()
                .ok_or_else(|| ValueError::Unsupported(
                    format!("{s} is not a 32 character long hexadecimal")
                ))?
        ))
    }
}

impl<'de> Deserialize<'de> for SessionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for SessionToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature="session")]
impl SessionToken {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn parse_token() -> anyhow::Result<()> {
        assert!(SessionToken::from_str("0").is_err());
        assert!(SessionToken::from_str("111111111111111111111111111111111").is_err());
        assert!(SessionToken::from_str("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
        let zeros = "00000000000000000000000000000000";
        let token = SessionToken::from_str(zeros)?;
        let s = token.to_string();
        assert_eq!(zeros, s);
        let p = SessionToken::from_str(&s)?;
        assert_eq!(token, p);
        Ok(())
    }
}
