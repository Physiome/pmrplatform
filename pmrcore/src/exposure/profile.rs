use crate::task_template::UserInputMap;

pub struct ExposureFileProfile {
    pub id: i64,
    pub exposure_file_id: i64,
    pub profile_id: i64,
    pub user_input: UserInputMap,
    // Ref version of this may provide the additional linked info?
}

mod impls;
pub mod traits;
