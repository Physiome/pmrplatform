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
    handle::ExposureCtrl,
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

// TODO impls for exposure_file
