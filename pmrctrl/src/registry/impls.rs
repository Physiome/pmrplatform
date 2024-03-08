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
use std::ops::Deref;
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
> TryFrom<&ExposureFileCtrl<'p, 'db, MCP, TMP>> for PreparedChoiceRegistry {
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureFileCtrl<'p, 'db, MCP, TMP>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("files", handle.pathinfo.files()?.into());
        registry.register("workspace_file_path", vec![
            handle
                .exposure_file
                .workspace_file_path()
                .to_string()
        ].into());
        // TODO figure out how to find roughly this
        // let file_on_fs = handle.platform.working_dir.join(
        //     "workspace",
        //     handle.exposure.workspace_id,
        //     handle.exposure.commit_id,
        //     handle.pathinfo.path
        // )
        Ok(registry)
    }
}
