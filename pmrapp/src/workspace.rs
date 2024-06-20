use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;

use crate::workspace::api::list_workspaces;

/*
#[component(transparent)]
pub fn WorkspaceRoutes() -> impl IntoView {
    view! {
        <Route path="/workspace" view=Workspace>
            <Route path="/" view=WorkspaceListing trailing_slash=TrailingSlash::Exact/>
            <Route path="listing" view=WorkspaceListing/>
            <Route path=":id?" view=WorkspaceView>
                <Route path="" view=WorkspaceView/>
            </Route>
        </Route>
        // <Route path="/workspace/" view=WorkspaceListing trailing_slash=TrailingSlash::Exact/>
        // <Route path="/workspace/:id/" view=WorkspaceView trailing_slash=TrailingSlash::Exact/>
    }
}
*/

#[component]
pub fn Workspace() -> impl IntoView {
    // provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        // <Stylesheet id="leptos" href="/pkg/workspace.css"/>

        // sets the document title
        <Title text="Physiome Model Repository > Workspace"/>
        <p>Before Workspace outlet</p>
        <Outlet/>
        <p>After Workspace outlet</p>

        /*
        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
            </main>
        </Router>
        */
    }
}

#[component]
pub fn WorkspaceListing() -> impl IntoView {
    let workspaces = create_resource(
        move || (),
        move |_| async move {
            let result = list_workspaces().await;
            match result {
                Ok(ref result) => logging::log!("{}", result.len()),
                Err(_) => logging::log!("error loading workspaces"),
            };
            result
        },
    );

    view! {
        <div class="main">
            <h1>"Listing of workspaces"</h1>
            <div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors/> }
                }>
                    {move || {
                        logging::log!("rendering listing");
                        let workspace_listing = { move || { workspaces
                            .get()
                            .map(move |workspaces| match workspaces {
                                Err(e) => {
                                    view! {
                                        <pre class="error">"Server Error: " {e.to_string()}</pre>
                                    }
                                        .into_view()
                                } 
                                Ok(workspaces) => {
                                    workspaces
                                        .into_iter()
                                        .map(move |workspace| {
                                            view! {
                                                <div>
                                                    <div><a href=format!("/workspace/{}/", workspace.id)>
                                                        "Workspace "{workspace.id}
                                                    </a></div>
                                                    <div>{workspace.url}</div>
                                                    <div>{workspace.description}</div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }
                            })
                            .unwrap_or_default()
                        }};
                        view! { <div>{workspace_listing}</div> }
                    }}
                </ErrorBoundary>
            </Transition>
            </div>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct WorkspaceParams {
    id: Option<String>,
}

#[component]
pub fn WorkspaceView() -> impl IntoView {
    let params = use_params::<WorkspaceParams>();
    let id = move || {
        params.with(|params| {
            params.as_ref()
                .map(|params| params.id.clone())
                .ok()
                .flatten()
        })
    };

    view! {
        <div class="main">
            <h1>"Viewing workspace "{id}</h1>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct WorkspaceCommitPathParams {
    id: Option<String>,
    commit: Option<String>,
    path: Option<String>,
}

#[component]
pub fn WorkspaceCommitPathView() -> impl IntoView {
    let params = use_params::<WorkspaceCommitPathParams>();
    let id = move || {
        params.with(|params| {
            params.as_ref()
                .map(|params| params.id.clone())
                .ok()
                .flatten()
        })
    };
    let commit = move || {
        params.with(|params| {
            params.as_ref()
                .map(|params| params.commit.clone())
                .ok()
                .flatten()
        })
    };
    let path = move || {
        params.with(|params| {
            params.as_ref()
                .map(|params| params.path.clone())
                .ok()
                .flatten()
        })
    };

    view! {
        <div class="main">
            <h1>"Viewing workspace "{id}</h1>
            <h2>"@"{commit}" / "{path}</h2>
        </div>
    }
}
