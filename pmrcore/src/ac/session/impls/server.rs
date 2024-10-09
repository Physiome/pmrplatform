pub use super::*;

mod token {
    use rand::prelude::*;
    use super::*;

    impl SessionToken {
        pub fn new() -> Self {
            Self(rand::thread_rng().gen())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn gen_token() -> anyhow::Result<()> {
        let token = SessionToken::new();
        let s = token.to_string();
        let p = SessionToken::from_str(&s)?;
        assert_eq!(token, p);
        Ok(())
    }

}
