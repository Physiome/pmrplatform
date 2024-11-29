use leptos::prelude::*;
use leptos_router::{
    components::{A, Route},
    MatchNestedRoutes,
    StaticSegment,
};
use pmrcore::ac::{
    agent::Agent,
    genpolicy::Policy,
    user::User,
    workflow::{
        State,
        state::Transition,
    },
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
    get_resource_policy_state,
};

#[derive(Clone)]
pub struct AccountCtx {
    pub current_user: ArcResource<Result<Option<User>, ServerFnError>>,
    pub set_ps: ArcWriteSignal<PolicyState>,
    pub res_ps: ArcResource<PolicyState>,
    // While the set_ps/res_ps does provide the PolicyState data, there
    // needs to be a way to refresh this as the state may change due to
    // user action (i.e. workflow state transitions); this provides the
    // means to refresh that.
    res_policy_state: ArcResource<()>,
    // This is used to signal an update to the above from the action.
    sig_policy_state: ArcRwSignal<PolicyState>,
}

pub fn provide_session_context() {
    let current_user = ArcResource::new_blocking(
        move || (),
        move |_| async move {
            current_user().await
        },
    );
    let sig_policy_state = ArcRwSignal::new(PolicyState::default());
    let (ps, set_ps) = arc_signal(PolicyState::default());

    let policy_state_update = sig_policy_state.clone();
    let ps_read = ps.clone();
    let res_ps = ArcResource::new_blocking(
        move || ps_read.get(),
        move |ps| {
            let policy_state_update = policy_state_update.clone();
            async move {
                policy_state_update.set(ps.clone());
                ps
            }
        },
    );

    let state_read = sig_policy_state.clone();
    let res_policy_state_update = set_ps.clone();
    let res_policy_state = ArcResource::new_blocking(
        move || (ps.get(), state_read.get()),
        move |(policy_state, sig_policy_state)| {
            let res_policy_state_update = res_policy_state_update.clone();
            async move {
                if sig_policy_state.policy.is_some() {
                    Effect::new(move |_| {
                        res_policy_state_update.set(sig_policy_state.clone());
                    });
                }
            }
        },
    );

    provide_context(AccountCtx {
        current_user,
        set_ps,
        res_ps,
        res_policy_state,
        sig_policy_state,
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
            action.version().get();
            let res_ps = res_ps.await;
            let res_ps_check = res_ps.clone();
            let workflow_state = res_ps.state;
            if let Some(policy) = res_ps.policy {
                (policy.agent != Agent::Anonymous).then(|| Some(view! {
                    <div class="flex-grow"></div>
                    <div id="content-action-wf-state"
                        class=format!("action state-{workflow_state}")
                    >
                        <span>{workflow_state.to_string()}</span>
                        <ActionForm action=action>
                            {move || {
                                match action.value().get() {
                                    Some(Ok(policy_state)) => {
                                        leptos::logging::log!("policy_state.state = {:?}", policy_state);
                                        leptos::logging::log!("got state={}", policy_state.state);

                                        let ctx = expect_context::<AccountCtx>();
                                        ctx.sig_policy_state.set(policy_state);
                                        // upon hydration the signal is empty? Also this must be
                                        // set _after_ the previous one, but this does cause a weird
                                        // double fetch issue when that applies.
                                        ctx.set_ps.set(res_ps_check.clone());

                                        // To ensure that we don't loop, otherwise this arm will be
                                        // triggered once more when this whole suspense is re-rendered;
                                        // safe to do as the value has been handled.
                                        action.value().set(None);
                                    }
                                    // TODO have this set an error somewhere?
                                    // Some(Err(ServerFnError::WrappedServerError(e))) => e.to_string(),
                                    _ => ()
                                }
                            }}
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
                        Some(Err(ServerFnError::WrappedServerError(e))) => Some(view! {
                            <p class="standard error">{format!("{e}")}</p>
                        }),
                        Some(Err(e)) => Some(view! {
                            <p class="standard error">{format!("System Error: {e:?}")}</p>
                        }),
                        _ => None,
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
