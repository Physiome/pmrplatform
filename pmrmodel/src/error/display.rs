use std::fmt::{
    Display,
    Formatter,
    Result,
};
use super::BuildArgErrors;

impl Display for BuildArgErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "BuildArgErrors: [{}]",
            self.0.iter()
                .map(|e| e.to_string() + ", ")
                .collect::<String>()
        )
    }
}

