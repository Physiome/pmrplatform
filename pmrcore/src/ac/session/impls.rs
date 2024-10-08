pub use super::SessionToken;

mod server {
    use rand::prelude::*;
    use super::*;

    impl SessionToken {
        pub fn new() -> Self {
            Self(rand::thread_rng().gen())
        }
    }
}

mod conversion {
    use std::{
        str::FromStr,
        fmt,
    };
    use crate::error::ValueError;
    use super::*;

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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn gen() -> anyhow::Result<()> {
        let token = SessionToken::new();
        let s = token.to_string();
        let p = SessionToken::from_str(&s)?;
        assert_eq!(token, p);
        Ok(())
    }

    #[test]
    fn parse() -> anyhow::Result<()> {
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
