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
        // TODO need a registry for files that provide a default value to the _current_ file name
        registry.register("files", handle.0.exposure.map_files_fs()?.into());
        let workspace_file_path = handle.0.exposure.ensure_fs()?.join(handle
            .exposure_file()
            .workspace_file_path()
            .to_string()
        ).display().to_string();
        registry.register("workspace_file_path", HashMap::from([
            ("workspace_file_path".to_string(), workspace_file_path),
        ]).into());
        registry.register("files_default", handle.0.exposure.map_files_fs()?.into());
        registry.select_keys("files_default", vec![
            handle.exposure_file()
                .workspace_file_path()
                .to_string()
        ]);
        Ok(registry)
    }
}
