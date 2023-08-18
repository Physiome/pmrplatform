use std::ops::Deref;
use crate::task::*;

impl From<[&str; 2]> for TaskArg {
    fn from(s: [&str; 2]) -> Self {
        Self { arg: s[0].to_owned() + s[1], .. Default::default() }
    }
}

impl From<&str> for TaskArg {
    fn from(s: &str) -> Self {
        Self { arg: s.into(), .. Default::default() }
    }
}

impl From<Vec<TaskArg>> for TaskArgs {
    fn from(args: Vec<TaskArg>) -> Self {
        Self(args)
    }
}

impl Deref for TaskArgs {
    type Target = Vec<TaskArg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
