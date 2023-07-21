use std::ops::{Deref, DerefMut};
use crate::profile::{
    ViewTaskTemplate,
    ViewTaskTemplates,
};

impl From<Vec<ViewTaskTemplate>> for ViewTaskTemplates {
    fn from(args: Vec<ViewTaskTemplate>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[ViewTaskTemplate; N]> for ViewTaskTemplates {
    fn from(args: [ViewTaskTemplate; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ViewTaskTemplates {
    type Target = Vec<ViewTaskTemplate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ViewTaskTemplates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
