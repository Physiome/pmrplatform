use leptos::prelude::*;
use leptos_router::{
    components::{A, ParentRoute, Route},
    nested_router::Outlet,
    SsrMode,
    StaticSegment,
};
use leptos_sync_ssr::signal::SsrSignalResource;
use pmrcore::ac::{
    agent::Agent,
    user::User,
    workflow::state::Transition,
};

use crate::{
    enforcement::PolicyState,
    workflow::state::TRANSITIONS,
};

pub mod api;
use api::{
    SignInWithLoginPassword,
    SignOut,
    WorkflowTransition,
    current_user,
};

#[derive(Clone)]
pub struct AccountCtx {
    pub current_user: ArcResource<Result<Option<User>, ServerFnError>>,
    pub policy_state: SsrSignalResource<Option<PolicyState>>,
}

impl AccountCtx {
    pub fn cleanup_policy_state(&self) {
        self.policy_state.inner_write_only().set(Some(PolicyState::default()));
    }
}

pub fn provide_session_context() {
    let current_user = ArcResource::new_blocking(
        move || (),
        move |_| async move {
            current_user().await
        },
    );
    let policy_state = SsrSignalResource::new(None);

    provide_context(AccountCtx {
        current_user,
        policy_state,
    });
}

#[component(transparent)]
pub fn ACRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("auth") view=AuthRoot ssr>
            <Route path=StaticSegment("/") view=LoginPage/>
            <Route path=StaticSegment("login") view=LoginPage/>
            <Route path=StaticSegment("logged_out") view=LogoutPage/>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn WorkflowState() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let action = ServerAction::<WorkflowTransition>::new();

    let policy_state = account_ctx.policy_state.read_only();
    let res_ps = ArcResource::new_blocking(
        || (),
        move |_| {
            let policy_state = policy_state.clone();
            #[cfg(not(feature = "ssr"))]
            policy_state.track();
            async move {
                policy_state.await
            }
        }
    );

    let workflow_view = move || {
        let res_ps = res_ps.clone();
        Suspend::new(async move {
            // TODO figure out where/how to deal with error here
            let ps = action.value()
                .get()
                // we are just dropping error here, ideally we should check and
                // render a error tooltip under the workflow state if there was
                // a problem
                .transpose()
                .ok()
                .flatten()
                .unwrap_or(res_ps.await.unwrap_or_default());
            // Ensure the action value is always cleared; this is to ensure
            // reactivity be preserved for the menu, otherwise all further
            // rendering after an action is taken below will be result in the
            // returned value from the action be used.
            action.value().set(None);
            let workflow_state = ps.state;
            leptos::logging::log!("<WorkflowState> {workflow_state}");
            if let Some(policy) = ps.policy {
                (policy.agent != Agent::Anonymous).then(|| Some(view! {
                    <div class="flex-grow"></div>
                    <div id="content-action-wf-state"
                        class=format!("action state-{workflow_state}")
                    >
                        <span>{workflow_state.to_string()}</span>
                        <ActionForm action=action>
                            <input type="hidden" name="resource" value=policy.resource.clone()/>
                            {
                                TRANSITIONS.transitions_for(workflow_state, policy.to_roles())
                                    .into_iter()
                                    .map(|Transition { target, description, .. }| view! {
                                        <button type="submit" name="target" value=target.to_string()>
                                            {description.to_string()}
                                        </button>
                                    })
                                    .collect_view()
                            }
                        </ActionForm>
                    </div>
                }))
            } else {
                None
            }
        })
    };

    view! {
        <Transition>{workflow_view}</Transition>
    }
}

#[component]
pub fn SessionStatus() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let action = ServerAction::<SignOut>::new();
    let session_status_view = move || {
        let current_user = account_ctx.current_user.clone();
        Suspend::new(async move {
            current_user.await
                .map(|result| {
                    result
                        .map(|u| view! {
                            <span>"Logged in as "{u.name}</span>
                            <ActionForm action=action>
                                <button type="submit">"Sign out"</button>
                            </ActionForm>
                        }.into_any())
                        .unwrap_or_else(|| view! {
                            <A href="/auth/login">"Sign in"</A>
                        }.into_any())
                })
                .unwrap_or_else(|_| view! {
                    <div>"Error retrieving session info."</div>
                }.into_any())
        })
    };
    let account_ctx = expect_context::<AccountCtx>();
    view! {
        <div id="session-status">
            {move || match action.value().get() {
                Some(Ok(())) => account_ctx.current_user.refetch(),
                _ => (),
            }}
            <Suspense>{session_status_view}</Suspense>
        </div>
    }
}

#[component]
fn AuthRoot() -> impl IntoView {
    view! {
        <Outlet/>
    }
}

#[component]
fn AuthTop() -> impl IntoView {
    view! {
        <h1>"Account"</h1>
        // TODO fill in the rest
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let action = ServerAction::<SignInWithLoginPassword>::new();

    view! {
        <h1>"Login Form"</h1>
        <ActionForm attr:id="sign-in" action=action>
            <div>
                {move || {
                    match action.value().get() {
                        Some(Ok(s)) => {
                            account_ctx.current_user.refetch();
                            Some(view! {
                                <p class="standard ok">{s}</p>
                            })
                        }
                        Some(Err(e)) => Some(view! {
                            <p class="standard error">{format!("Error: {e}")}</p>
                        }),
                        None => None,
                    }
                }}
            </div>
            <div>
                <label for="login">"Login"</label>
                <input type="text" name="login" required/>
            </div>
            <div>
                <label for="password">"Password"</label>
                <input type="password" name="password" required/>
            </div>
            <div>
                <input type="submit" value="Login"/>
            </div>
        </ActionForm>
    }
}

#[component]
fn LogoutPage() -> impl IntoView {
    view! {
        <h1>"You are now logged out."</h1>
    }
}
