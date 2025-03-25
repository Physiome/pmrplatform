use leptos::prelude::*;
use leptos_router::{
    components::{A, Route},
    MatchNestedRoutes,
    StaticSegment,
};
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
    pub set_ps: ArcWriteSignal<PolicyState>,
    pub res_ps: ArcResource<PolicyState>,
}

pub fn provide_session_context() {
    let current_user = ArcResource::new_blocking(
        move || (),
        move |_| async move {
            current_user().await
        },
    );
    let (ps, set_ps) = arc_signal(PolicyState::default());

    let ps_read = ps.clone();
    let res_ps = ArcResource::new_blocking(
        move || ps_read.get(),
        move |ps| async move { ps },
    );

    provide_context(AccountCtx {
        current_user,
        set_ps,
        res_ps,
    });
}

#[component]
pub fn ACRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <Route path=StaticSegment("login") view=LoginPage/>
        <Route path=StaticSegment("logged_out") view=LogoutPage/>
    }
    .into_inner()
}

#[component]
pub fn WorkflowState() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let action = ServerAction::<WorkflowTransition>::new();

    let workflow_view = move || {
        let res_ps = account_ctx.res_ps.clone();
        // leptos::logging::log!("{res_ps:?}");
        Suspend::new(async move {
            // TODO figure out where/how to deal with error here
            let res_ps = action.value()
                .get()
                // we are just dropping error here, ideally we should check and
                // render a error tooltip under the workflow state if there was
                // a problem
                .transpose()
                .ok()
                .flatten()
                .unwrap_or(res_ps.await);
            let workflow_state = res_ps.state;
            if let Some(policy) = res_ps.policy {
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
                            <A href="/login">"Sign in"</A>
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
