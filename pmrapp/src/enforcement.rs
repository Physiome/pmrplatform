use leptos::{
    context::use_context,
    prelude::{
        Resource,
        ServerFnError,
        Set as _,
    },
};
use pmrcore::ac::{
    genpolicy::Policy,
    workflow::State,
};
use serde::{
    Serialize,
    Deserialize,
};

use crate::{
    ac::AccountCtx,
    error::AppError,
};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PolicyState {
    pub policy: Option<Policy>,
    pub state: State,
}

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

    pub fn notify_into(self) -> T {
        if let Some(ctx) = use_context::<AccountCtx>() {
            leptos::logging::warn!("EnforcedOk::notify_into calling set_ps with {:?}", &self.ps);
            ctx.set_ps.set(self.ps);
        } else {
            leptos::logging::warn!("AccountCtx context is missing");
        }
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
