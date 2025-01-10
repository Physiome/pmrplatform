use serde::{Serialize, Deserialize};
use crate::task_template::UserInputMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct ExposureFileProfile {
    pub id: i64,
    pub exposure_file_id: i64,
    pub profile_id: i64,
    pub user_input: UserInputMap,
    // Ref version of this may provide the additional linked info?
    // Linkage should be done directly to Profile, with the additional
    // data types like ViewTaskTemplates (and future additions e.g.
    // tags) will have additional associations, as there may be
    // differences between what's actually assigned to the profile vs.
    // the ones currently assigned to a given ExposureFile.
}

mod impls;
pub mod traits;
