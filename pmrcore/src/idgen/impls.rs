use super::*;

impl From<i64> for HexId {
    fn from(id: i64) -> Self {
        Self { id }
    }
}

mod display {
    use std::fmt::{Display, Formatter, Result};
    use super::*;

    impl Display for HexId {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{:x}", self.id)
        }
    }
}
