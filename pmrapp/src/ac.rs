use leptos::prelude::*;
use leptos_router::{
    components::Route,
    MatchNestedRoutes,
    StaticSegment,
};

mod api;
use api::{
    AuthenticateLoginPassword,
    current_user,
};

#[component]
pub fn ACRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <Route path=StaticSegment("login") view=LoginPage/>
        // <Route path=StaticSegment("logout") view=LogoutPage/>
    }
    .into_inner()
}

#[component]
fn LoginPage() -> impl IntoView {
    let action = ServerAction::<AuthenticateLoginPassword>::new();
    // TODO it should be some signal that have the user set already when hydrate
    // and be set up at the root App, sort of like the portlet
    let current_user = Resource::new_blocking(
        move || action.version().get(),
        move |_| async move {
            current_user().await
        },
    );
    view! {
        <h1>"Login Form"</h1>
        <Suspense>{
            move || Suspend::new(async move {
                current_user.await
                    .map(|result| {
                        result
                            .map(|u| view! {
                                <div>"You are logged in as "{u.name}"."</div>
                            }.into_any())
                            .unwrap_or_else(|| view! {
                                <div>"You are not logged in."</div>
                            }.into_any())
                    })
                    .unwrap_or_else(|_| view! {
                        <div>"Error retrieving user info."</div>
                    }.into_any())
            })
        }</Suspense>
        <ActionForm action=action>
            <div>
                <label>"Login"
                    <input type="text" name="login" required/>
                </label>
            </div>
            <div>
                <label>"Password"
                    <input type="password" name="password" required/>
                </label>
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
