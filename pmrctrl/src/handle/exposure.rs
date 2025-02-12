use parking_lot::Mutex;
use pmrcore::exposure::ExposureRef;
use pmrrepo::handle::GitHandle;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        OnceLock,
    },
};

use crate::{
    platform::Platform,
    handle::{
        EFViewTaskTemplatesCtrl,
        ExposureFileCtrl,
    },
};

pub(crate) struct RawExposureCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) git_handle: GitHandle<'p>,
    pub(crate) exposure: ExposureRef<'p>,
    pub(crate) exposure_file_ctrls: Arc<Mutex<HashMap<String, ExposureFileCtrl<'p>>>>,
    pub(crate) efvttcs: OnceLock<Vec<(String, Option<EFViewTaskTemplatesCtrl<'p>>)>>,
    // this isn't exactly possible due to the registry borrows.
    // pub(crate) efvttcs: Arc<Mutex<HashMap<String, EFViewTaskTemplatesCtrl<'p>>>>,
}

pub struct ExposureCtrl<'p>(pub(crate) Arc<RawExposureCtrl<'p>>);

mod impls;
