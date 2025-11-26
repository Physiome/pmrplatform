use leptos::prelude::{
    Resource,
    ServerFnError,
    Set as _,
    take_context,
};
use leptos_sync_ssr::signal::SsrWriteSignal;
use pmrcore::ac::{
    genpolicy::Policy,
    workflow::State,
};
use serde::{
    Serialize,
    Deserialize,
};

use crate::error::AppError;

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PolicyState {
    pub policy: Option<Policy>,
    pub state: State,
}

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EnforcedOk<T> {
    pub inner: T,
    pub ps: PolicyState,
}

impl PolicyState {
    pub fn new(policy: Option<Policy>, state: State) -> Self {
        Self { policy, state }
    }

    pub fn to_enforced_ok<T>(self, inner: T) -> EnforcedOk<T> {
        EnforcedOk::new(inner, self)
    }
}

impl<T> EnforcedOk<T> {
    pub fn new(inner: T, ps: PolicyState) -> Self {
        Self { inner, ps }
    }

    pub fn notify_into_inner(self) -> T {
        if let Some(ctx) = take_context::<SsrWriteSignal<Option<PolicyState>>>() {
            leptos::logging::warn!("EnforcedOk::notify_into_inner calling set_ps with {:?}", &self.ps);
            ctx.set(Some(self.ps));
        } else {
            leptos::logging::warn!("SsrWriteSignal<Option<PolicyState>> not provided as a context to be taken");
        }
        self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> From<T> for EnforcedOk<T> {
    fn from(inner: T) -> Self {
        PolicyState::default()
            .to_enforced_ok(inner)
    }
}

pub type Result<T> = std::result::Result<EnforcedOk<T>, ServerFnError<AppError>>;
pub type ResourceResult<T> = Resource<std::result::Result<EnforcedOk<T>, AppError>>;
