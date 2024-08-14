use pmrcore::exposure::ExposureFileRef;
use pmrrepo::handle::GitHandleResult;
use std::sync::Arc;

use crate::{
    platform::Platform,
    handle::ExposureCtrl,
};

pub(crate) struct RawExposureFileCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) exposure: ExposureCtrl<'p>,
    pub(crate) exposure_file: ExposureFileRef<'p>,
    pub(crate) pathinfo: GitHandleResult<'p>,
}

pub struct ExposureFileCtrl<'p>(pub(crate) Arc<RawExposureFileCtrl<'p>>);

mod impls;
