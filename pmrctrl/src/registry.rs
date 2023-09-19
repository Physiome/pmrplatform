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

pub(crate) fn make_choice_registry<'a, MCP, TMP>(
    handle: &ExposureCtrl<'a, MCP, TMP>,
) -> Result<PreparedChoiceRegistry, PlatformError>
where
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
{
    let mut registry = PreparedChoiceRegistry::new();

    // TODO figure out how to expose/define this registration in a more
    // agnostic context?
    registry.register("files", handle.list_files()?.into());

    Ok(registry)
}
