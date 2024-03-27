use std::collections::HashMap;

use crate::exposure::profile::ExposureFileProfile;

impl ExposureFileProfile {
    pub fn new(
        id: i64,
        exposure_file_id: i64,
        profile_id: i64,
    ) -> Self {
        Self {
            id,
            exposure_file_id,
            profile_id,
            user_input: HashMap::new(),
        }
    }
}
