use pmrcore::exposure::ExposureFileViewRef;

use crate::platform::Platform;

pub struct ExposureFileViewCtrl<'p> {
    pub(crate) platform: &'p Platform,
    // TODO there needs to be an Arc<ExposureFileCtrl> stored here
    pub exposure_file_view: ExposureFileViewRef<'p>,
}

mod impls;
