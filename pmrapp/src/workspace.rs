use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pmrcore::repo::{
    PathObjectInfo,
    RepoResult,
};

mod api;

use crate::workspace::api::{
    list_workspaces,
    get_workspace_info,
};

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
    id: Option<i64>,
}

#[component]
pub fn WorkspaceView() -> impl IntoView {
    let params = use_params::<WorkspaceParams>();
    let id = move || {
        params.with(|p| {
            p.as_ref()
                .map(|p| p.id.unwrap_or_default())
                .map_err(|_| AppError::NotFound)
        })
    };

    let resource = create_resource(id, |id| async move {
        match id {
            Err(e) => Err(e),
            Ok(id) => get_workspace_info(id)
                .await
                .map_err(|_| AppError::InternalServerError),
        }
    });

    let info = move || match resource.get() {
        Some(Ok(v)) => Ok(v),
        _ => Err(AppError::InternalServerError),
    };

    let workspace_view = move || {
        info().map(|info| {
            view! {
                // render content
                <h1>{info.workspace.description.as_ref().unwrap_or(
                    &format!("Workspace {}", &info.workspace.id))}</h1>
                <dl>
                    <dt>"Git Repository URI"</dt>
                    <dd>{&info.workspace.url}</dd>
                    <div class="workspace-pathinfo">
                        <WorkspaceFileTable repo_result=info/>
                    </div>
                </dl>
            }
        })
    };

    view! {
        <Suspense fallback=move || view! { <p>"Loading workspace..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        <h1>"Something went wrong."</h1>
                        <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, error)| view! { <li>{error.to_string()} </li> })
                            .collect_view()
                        }
                        </ul>
                    </div>
                }
            }>
                {workspace_view}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn WorkspaceFileTable(repo_result: RepoResult) -> impl IntoView {
    view! {
        <table class="file-listing">
            <thead>
                <tr>
                    <th>"Filename"</th>
                    <th>"Size"</th>
                    <th>"Date"</th>
                </tr>
            </thead>
            {
                match repo_result.target {
                    PathObjectInfo::TreeInfo(tree_info) => {
                        view! {
                            <tbody>
                            {
                                tree_info.entries.iter().map(|info| view! {
                                    <WorkspaceFileRow
                                        workspace_id=repo_result.workspace.id
                                        commit_id=repo_result.commit.commit_id.clone()
                                        path=repo_result.path.clone()
                                        kind=info.kind.clone()
                                        name=info.name.clone()/>
                                })
                                .collect_view()
                            }
                            </tbody>
                        }
                    },
                    _ => view! { <tbody></tbody> },
                }
            }
        </table>
    }
}

#[component]
fn WorkspaceFileRow(
    workspace_id: i64,
    commit_id: String,
    path: String,
    kind: String,
    name: String,
) -> impl IntoView {
    let path_name = if name == ".." {
        let idx = path[0..path.len() - 1].rfind('/').unwrap_or(0);
        if idx == 0 {
            "".to_string()
        } else {
            format!("{}/", &path[0..idx])
        }
    } else {
        format!("{}{}", path, if kind == "tree" {
            format!("{}/", name)
        } else {
            format!("{}", name)
        })
    };
    let href = format!("/workspace/{}/file/{}/{}", workspace_id, commit_id, path_name);
    view! {
        <tr>
            <td class=format!("gitobj-{}", kind)>
                <span><a href=href>{name}</a></span>
            </td>
            <td></td>
            <td></td>
        </tr>
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
