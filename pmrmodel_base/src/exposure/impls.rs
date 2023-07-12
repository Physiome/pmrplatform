use std::ops::{Deref, DerefMut};
use crate::exposure::*;

impl From<Vec<Exposure>> for Exposures {
    fn from(args: Vec<Exposure>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[Exposure; N]> for Exposures {
    fn from(args: [Exposure; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for Exposures {
    type Target = Vec<Exposure>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Exposures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ExposureFile>> for ExposureFiles {
    fn from(args: Vec<ExposureFile>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[ExposureFile; N]> for ExposureFiles {
    fn from(args: [ExposureFile; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ExposureFiles {
    type Target = Vec<ExposureFile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExposureFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ExposureFileView>> for ExposureFileViews {
    fn from(args: Vec<ExposureFileView>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[ExposureFileView; N]> for ExposureFileViews {
    fn from(args: [ExposureFileView; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ExposureFileViews {
    type Target = Vec<ExposureFileView>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExposureFileViews {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
