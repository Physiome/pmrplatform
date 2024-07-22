use parking_lot::Mutex;
use pmrcore::exposure::ExposureRef;
use pmrrepo::handle::GitHandle;
use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::{
    platform::Platform,
    handle::ExposureFileCtrl,
};

pub(crate) struct RawExposureCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) git_handle: GitHandle<'p>,
    pub(crate) exposure: ExposureRef<'p>,
    pub(crate) exposure_file_ctrls: Arc<Mutex<HashMap<String, ExposureFileCtrl<'p>>>>,
}

pub struct ExposureCtrl<'p>(pub(crate) Arc<RawExposureCtrl<'p>>);

mod impls;
