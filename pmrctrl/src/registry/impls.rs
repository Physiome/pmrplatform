use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrmodel::registry::{
    ChoiceRegistry,
    PreparedChoiceRegistry,
};
use crate::{
    error::PlatformError,
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
};

impl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TryFrom<&ExposureCtrl<'a, MCP, TMP>> for PreparedChoiceRegistry {
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureCtrl<'a, MCP, TMP>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        // TODO figure out how to expose/define this registration in a more
        // agnostic context?
        registry.register("files", handle.list_files()?.into());
        // TODO figure out if we want to reuse the registry for the required
        // data, i.e. the current exposure file associated with the task and
        // the commit
        Ok(registry)
    }
}

impl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TryFrom<&ExposureFileCtrl<'a, MCP, TMP>> for PreparedChoiceRegistry {
    type Error = PlatformError;

    fn try_from(
        handle: &ExposureFileCtrl<'a, MCP, TMP>,
    ) -> Result<Self, Self::Error> {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("files", handle.pathinfo.files()?.into());
        Ok(registry)
    }
}
