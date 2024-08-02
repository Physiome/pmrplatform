use pmrcore::exposure::ExposureFileViewRef;

use crate::{
    handle::ExposureFileCtrl,
    platform::Platform,
};

pub struct ExposureFileViewCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) exposure_file_view: ExposureFileViewRef<'p>,
    pub(crate) exposure_file: ExposureFileCtrl<'p>,
    pub(crate) view_path: Option<String>,
}

mod impls;
