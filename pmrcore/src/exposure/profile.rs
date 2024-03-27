use crate::task_template::UserInputMap;

pub struct ExposureFileProfile {
    pub id: i64,
    pub exposure_file_id: i64,
    pub profile_id: i64,
    pub user_input: UserInputMap,
}

mod impls;
pub mod traits;
