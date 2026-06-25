use pmrcore::exposure::traits::ExposureFile as _;

use crate::{
    error::PlatformError,
    platform::Platform,
};

impl Platform {
    /// Add a workspace alias while also index its aliased_uri.
    pub async fn add_workspace_alias(
        &self,
        workspace_id: i64,
        alias: &str
    ) -> Result<(), PlatformError> {
        self.mc_platform.add_alias("workspace", workspace_id, alias).await?;
        self.pc_platform.resource_link_kind_with_term(
            &format!("/workspace/{workspace_id}/"),
            "aliased_uri",
            &format!("/workspace/{alias}/"),
        )
        .await?;
        Ok(())
    }

    /// Add an exposure alias while also index its aliased_uri.
    pub async fn add_exposure_alias(
        &self,
        exposure_id: i64,
        alias: &str
    ) -> Result<(), PlatformError> {
        self.mc_platform.add_alias("exposure", exposure_id, alias).await?;
        self.pc_platform.resource_link_kind_with_term(
            &format!("/exposure/{exposure_id}/"),
            "aliased_uri",
            &format!("/exposure/{alias}/"),
        )
        .await?;

        for exposure_file in self.mc_platform.list_for_exposure(exposure_id).await?.iter() {
            let workspace_file_path = exposure_file.workspace_file_path();
            let resource_path = format!("/exposure/{exposure_id}/{workspace_file_path}");
            let aliased_uri = format!("/exposure/{alias}/{workspace_file_path}");
            self.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "exposure_alias",
                &alias,
            )
            .await?;
            self.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "aliased_uri",
                &aliased_uri,
            )
            .await?;
        }
        Ok(())
    }
}
