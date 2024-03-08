use pmrcore::{
    exposure::traits::ExposureFile,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrmodel::registry::{
    ChoiceRegistry,
    PreparedChoiceRegistry,
};
use std::{
    collections::HashMap,
    ops::Deref,
};

use crate::{
    error::PlatformError,
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
};

// TODO figure out how to define a registry for registration, i.e. there
// needs to be a mapping of registry names to the relevant items, done
// in a more definition driven manner.
//
// An idea for this might be a default method and the impl TryFrom for
// each of them would flag the relevant ones on or off.

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TryFrom<&ExposureCtrl<'p, 'db, MCP, TMP>> for PreparedChoiceRegistry
where
    'p: 'db
{
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureCtrl<'p, 'db, MCP, TMP>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("files", handle.map_files_fs()?.into());
        // TODO figure out if we want to reuse the registry for the required
        // data, i.e. the current exposure file associated with the task and
        // the commit
        Ok(registry)
    }
}

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TryFrom<&ExposureFileCtrl<'p, 'db, MCP, TMP>> for PreparedChoiceRegistry
where
    'p: 'db
{
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureFileCtrl<'p, 'db, MCP, TMP>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("files", handle.exposure.map_files_fs()?.into());
        let workspace_file_path = handle.exposure.ensure_fs()?.join(handle
            .exposure_file()
            .workspace_file_path()
            .to_string()
        ).display().to_string();
        registry.register("workspace_file_path", HashMap::from([
            ("workspace_file_path".to_string(), workspace_file_path),
        ]).into());
        Ok(registry)
    }
}
