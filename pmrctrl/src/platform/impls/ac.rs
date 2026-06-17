use pmrcore::{
    ac::{
        genpolicy::Policy,
        user::User,
        workflow::{
            State,
            state::transition::StateTransitions,
        },
    },
};
use crate::{
    error::PlatformError,
    platform::Platform,
};

impl Platform {
    /// Attempt to make a workflow transition using `User` on a `resource` to some target `State`.
    ///
    /// Returns the new `Policy` based on what the `User` may do on the `resource`, if the user has the
    /// permission to effect the transiation, otherwise `None` is returned.
    pub async fn workflow_transition(
        &self,
        user: &User,
        resource: String,
        target_state: State,
    ) -> Result<Option<Policy>, PlatformError> {
        let transitions = StateTransitions::default();

        let state = self
            .ac_platform
            .get_wf_state_for_res(&resource)
            .await?;
        let roles = self
            .ac_platform
            .generate_policy_for_agent_res(&(user.clone().into()), resource.clone())
            .await?
            .to_roles();
        if transitions.validate(roles, state, target_state) {
            let ts = self.ac_platform.set_wf_state_for_res(&resource, target_state).await?;

            // Only clear the state if not expired, as it assumes the published date should remain in the
            // index.
            if target_state != State::Expired {
                self.pc_platform.forget_resource_path(
                    Some("published_date"),
                    &resource,
                ).await?;
            }
            // Only index when new state is published.
            if target_state == State::Published {
                self.pc_platform.resource_link_kind_with_term(
                    &resource,
                    "published_date",
                    &ts.to_string(),
                ).await?;
            }

            let policy = self
                .ac_platform
                .generate_policy_for_agent_res(&(user.clone()).into(), resource)
                .await?;
            Ok(Some(policy))
        } else {
            Ok(None)
        }
    }
}
