use pmrcore::{
    exposure::{
        task::ExposureFileViewTask,
        traits::ExposureFileBackend,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use super::ExposureFileCtrl;
use crate::{
    error::PlatformError,
};

impl<
    'db,
    'repo,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileCtrl<'db, 'repo, MCP, TMP> {
    fn create_view_task(
        exposure_file_view_id: i64,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewTask, PlatformError> {
        todo!();
    }
}
