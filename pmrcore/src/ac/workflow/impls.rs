use std::{
    fmt,
    str::FromStr,
};
use crate::error::ValueError;
use super::State;

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}

impl From<State> for String {
    fn from(state: State) -> String {
        format!("{state}")
    }
}

impl From<State> for &'static str {
    fn from(state: State) -> &'static str {
        match state {
            State::Private => "private",
            State::Pending => "pending",
            State::Published => "published",
            State::Expired => "expired",
            State::Unknown => "unknown",
        }
    }
}

impl FromStr for State {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "private" => Ok(State::Private),
            "pending" => Ok(State::Pending),
            "published" => Ok(State::Published),
            "expired" => Ok(State::Expired),
            // Unknown,
            s => Err(ValueError::Unsupported(s.to_string())),
        }
    }
}

#[cfg(feature = "clap")]
mod clap {
    use ::clap::{
        ValueEnum,
        builder::PossibleValue,
    };
    use super::*;

    impl ValueEnum for State {
        fn value_variants<'a>() -> &'a [Self] {
            &[
                State::Private,
                State::Pending,
                State::Published,
                State::Expired,
            ]
        }

        fn to_possible_value(&self) -> Option<PossibleValue> {
            match self {
                State::Private => Some(PossibleValue::new("private")),
                State::Pending => Some(PossibleValue::new("pending")),
                State::Published => Some(PossibleValue::new("published")),
                State::Expired => Some(PossibleValue::new("expired")),
                State::Unknown => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::State;
    use crate::error::ValueError;

    #[test]
    fn smoke() -> anyhow::Result<()> {
        // sample of standard conversions
        assert_eq!(State::Private.to_string(), "private");
        assert_eq!(State::Private, State::from_str("private")?);
        assert_eq!(State::Published.to_string(), "published");
        assert_eq!(State::Published, State::from_str("published")?);

        // error conversion
        assert!(State::from_str("unknown").is_err());
        assert!(matches!(
            State::from_str("no_such_workflow_state")
                .expect_err("should be an error"),
            ValueError::Unsupported(s) if s == "no_such_workflow_state".to_string(),
        ));

        // infallable conversion
        assert_eq!(
            State::from_str("no_such_wf_state")
                .unwrap_or_default(),
            State::Unknown,
        );
        Ok(())
    }
}
