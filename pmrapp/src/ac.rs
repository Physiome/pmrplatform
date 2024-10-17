use leptos::prelude::*;
use leptos_router::{
    components::{A, Route},
    MatchNestedRoutes,
    StaticSegment,
};
use pmrcore::ac::user::User;

mod api;
use api::{
    SignInWithLoginPassword,
    SignOut,
    current_user,
};

#[derive(Clone)]
pub struct AccountCtx {
    pub current_user: ArcResource<Result<Option<User>, ServerFnError>>,
    pub sign_in_with_login_password: ServerAction<SignInWithLoginPassword>,
    pub sign_out: ServerAction<SignOut>,
}

pub fn provide_session_context() {
    let sa_siwlp = ServerAction::<SignInWithLoginPassword>::new();
    let sa_so = ServerAction::<SignOut>::new();
    let sign_in_with_login_password = sa_siwlp.clone();
    let sign_out = sa_so.clone();
    let current_user = ArcResource::new_blocking(
        move || (
            sa_siwlp.version().get(),
            sa_so.version().get(),
        ),
        move |_| async move {
            current_user().await
        },
    );
    provide_context(AccountCtx {
        current_user,
        sign_in_with_login_password,
        sign_out,
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
    let action = account_ctx.sign_out.clone();
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
    view! {
        <div id="session-status">
            <Suspense>{session_status_view}</Suspense>
        </div>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let action = account_ctx.sign_in_with_login_password.clone();

    let login_result = RwSignal::new(String::new());

    Effect::new(move |_| {
        action.version().get();
        match action.value().get() {
            Some(Ok(true)) => login_result.set("You are logged in.".to_string()),
            Some(Ok(false)) => login_result.set("Invalid credentials provided.".to_string()),
            Some(Err(ServerFnError::ServerError(e))) => login_result.set(e.to_string()),
            _ => return,
        };
    });

    view! {
        <h1>"Login Form"</h1>
        <div>
            {login_result}
        </div>
        <ActionForm attr:id="sign-in" action=action>
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
