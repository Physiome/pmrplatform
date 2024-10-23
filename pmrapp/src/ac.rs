use leptos::prelude::*;
use leptos_router::{
    components::{A, Route},
    MatchNestedRoutes,
    StaticSegment,
};
use pmrcore::ac::user::User;

pub mod api;
use api::{
    SignInWithLoginPassword,
    SignOut,
    current_user,
};

#[derive(Clone)]
pub struct AccountCtx {
    pub current_user: ArcResource<Result<Option<User>, ServerFnError>>,
}

pub fn provide_session_context() {
    let current_user = ArcResource::new_blocking(
        move || (),
        move |_| async move {
            current_user().await
        },
    );
    provide_context(AccountCtx {
        current_user,
    });
}

#[component]
pub fn ACRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <Route path=StaticSegment("login") view=LoginPage/>
        // <Route path=StaticSegment("logout") view=LogoutPage/>
    }
    .into_inner()
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
                            s
                        }
                        Some(Err(ServerFnError::WrappedServerError(e))) => e.to_string(),
                        _ => String::new(),
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
        <h1>"You are logged out"</h1>
    }
}
