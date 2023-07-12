use std::ops::{
    Deref,
    DerefMut,
};
use crate::workspace::*;

impl From<Vec<Workspace>> for Workspaces {
    fn from(args: Vec<Workspace>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[Workspace; N]> for Workspaces {
    fn from(args: [Workspace; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for Workspaces {
    type Target = Vec<Workspace>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Workspaces {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
