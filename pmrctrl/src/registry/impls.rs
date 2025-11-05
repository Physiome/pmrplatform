use pmrcore::exposure::traits::ExposureFile;
use pmrmodel::registry::{
    ChoiceRegistry,
    PreparedChoiceRegistry,
};
use std::collections::HashMap;

use crate::{
    error::PlatformError,
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
};

// TODO figure out how to define a registry for registration, i.e. there
// needs to be a mapping of registry names to the relevant items, done
// in a more definition driven manner (registry of registries)
//
// An idea for this might be a default method and the impl TryFrom for
// each of them would flag the relevant ones on or off.

impl<'p> TryFrom<&ExposureCtrl<'p>> for PreparedChoiceRegistry {
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureCtrl<'p>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("files", handle.map_files_fs()?.into());
        // TODO figure out if we want to reuse the registry for the required
        // data, i.e. the current exposure file associated with the task and
        // the commit
        Ok(registry)
    }
}

impl<'p> TryFrom<&ExposureFileCtrl<'p>> for PreparedChoiceRegistry {
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureFileCtrl<'p>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        // `files` is the registry for full listing of files
        registry.register("files", handle.0.exposure.map_files_fs()?.into());

        // `workspace_file_path` provides a single default entry that points to the
        // `workspace_file_path` of the given `exposure_file` on the filesystem.
        let workspace_file_path = handle.0.exposure.ensure_fs()?.join(handle
            .exposure_file()
            .workspace_file_path()
        ).display().to_string();
        registry.register("workspace_file_path", HashMap::from([
            ("workspace_file_path".to_string(), workspace_file_path),
        ]).into());
        // TODO `workspace_file_path` should include `fs` to denote it being on the filesystem.
        // Perhaps `_path` is the path related to things on filesystem but it should really
        // have a dedicated prefix or suffix.

        // `files_default` is the registry for full listing of files with the current
        // file being selected as the default.
        registry.register("files_default", handle.0.exposure.map_files_fs()?.into());
        registry.select_keys("files_default", vec![
            handle.exposure_file()
                .workspace_file_path()
                .to_string()
        ]);

        // `exposure_id` is the id of the underlying exposure
        registry.register("exposure_id", HashMap::from([
            ("exposure_id".to_string(), handle.exposure_file().exposure_id().to_string())
        ]).into());

        // `exposure_file` is the path to the current exposure file.
        registry.register("exposure_file", HashMap::from([
            ("exposure_file".to_string(), handle.exposure_file().workspace_file_path().to_string()),
        ]).into());

        Ok(registry)
    }
}
