use std::{
    fmt,
    str::FromStr,
};
use crate::error::ValueError;
use super::State;

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<State> for &'static str {
    fn from(state: State) -> &'static str {
        match state {
            State::Private => "Private",
            State::Pending => "Pending",
            State::Published => "Published",
            State::Expired => "Expired",
            State::Unknown => "Unknown",
        }
    }
}

impl FromStr for State {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Private" => Ok(State::Private),
            "Pending" => Ok(State::Pending),
            "Published" => Ok(State::Published),
            "Expired" => Ok(State::Expired),
            // Unknown,
            s => Err(ValueError::Unsupported(s.to_string())),
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
        assert_eq!(State::Private.to_string(), "Private");
        assert_eq!(State::Private, State::from_str("Private")?);
        assert_eq!(State::Published.to_string(), "Published");
        assert_eq!(State::Published, State::from_str("Published")?);

        // error conversion
        assert!(State::from_str("Unknown").is_err());
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
