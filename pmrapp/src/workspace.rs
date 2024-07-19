use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pmrcore::repo::{
    PathObjectInfo,
    RepoResult,
    TreeInfo,
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
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
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
            </Suspense>
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
    let resource = create_resource(
        move || params.get().map(|p| p.id),
        |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => get_workspace_info(id, None, None)
                    .await
                    .map_err(|_| AppError::NotFound),
            }
        }
    );

    let info = move || match resource.get() {
        Some(Ok(v)) => Ok(v),
        Some(Err(AppError::NotFound)) => Err(AppError::NotFound),
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
                        <WorkspaceListingView repo_result=info/>
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
                        {// This will not hoist the 404 to the main page?
                        move || errors.get()
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
fn WorkspaceListingView(repo_result: RepoResult) -> impl IntoView {
    let workspace_id = repo_result.workspace.id;
    let commit_id = repo_result.commit.commit_id.clone();
    let path = repo_result.path.clone();
    let pardir = repo_result.path != "";
    let pardir = move || { pardir.then(|| view! {
        <WorkspaceTreeInfoRow
            workspace_id=workspace_id
            commit_id=commit_id.as_str()
            path=path.as_str()
            kind="pardir"
            name=".."/>
    })};

    let commit_id = repo_result.commit.commit_id.clone();
    let path = repo_result.path.clone();
    view! {
        <table class="file-listing">
            <thead>
                <tr>
                    <th>"Filename"</th>
                    <th>"Size"</th>
                    <th>"Date"</th>
                </tr>
            </thead>
            <tbody>
            {pardir}
            {
                match repo_result.target {
                    PathObjectInfo::TreeInfo(tree_info) =>
                        Some(view! {
                            <WorkspaceTreeInfoRows
                                workspace_id=workspace_id
                                commit_id=commit_id
                                path=path
                                tree_info
                                />
                        }),
                    _ => None,
                }
            }
            </tbody>
        </table>
    }
}

#[component]
fn WorkspaceRepoResultView(repo_result: RepoResult) -> impl IntoView {
    match repo_result.target {
        PathObjectInfo::TreeInfo(_) =>
            Some(view! { <div><WorkspaceListingView repo_result/></div> }),
        PathObjectInfo::FileInfo(ref file_info) => {
            let href = format!(
                "/workspace/{}/rawfile/{}/{}",
                &repo_result.workspace.id,
                &repo_result.commit.commit_id,
                &repo_result.path,
            );
            let info = format!("{:?}", file_info);
            Some(view! {
                <div>
                <div>{info}</div>
                <div>
                    <a href=&href target="_self">"download"</a>
                </div>
                {
                    (file_info.mime_type[..5] == *"image").then(||
                        view! {
                            <div>
                                <p>"Preview"</p>
                                <img src=&href />
                            </div>
                        }
                    )
                }
                </div>
            })
        }
        _ => None,
    }
}

#[component]
fn WorkspaceTreeInfoRows(
    workspace_id: i64,
    #[prop(into)]
    commit_id: String,
    #[prop(into)]
    path: String,
    tree_info: TreeInfo,
) -> impl IntoView {
    view! {{
        tree_info.entries
            .iter()
            .map(|info| view! {
                <WorkspaceTreeInfoRow
                    workspace_id=workspace_id
                    commit_id=commit_id.as_str()
                    path=path.as_str()
                    kind=info.kind.as_str()
                    name=info.name.as_str()/>
            })
            .collect_view()
    }}
}

#[component]
fn WorkspaceTreeInfoRow(
    workspace_id: i64,
    #[prop(into)]
    commit_id: String,
    #[prop(into)]
    path: String,
    #[prop(into)]
    kind: String,
    #[prop(into)]
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
    id: Option<i64>,
    commit: Option<String>,
    path: Option<String>,
}

#[component]
pub fn WorkspaceCommitPathView() -> impl IntoView {
    let params = use_params::<WorkspaceCommitPathParams>();
    let resource = create_resource(
        move || params.get().map(|p| (p.id, p.commit, p.path)),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok((Some(id), commit, path)) => get_workspace_info(id, commit, path)
                    .await
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::NotFound),
            }
        }
    );
    let info = move || match resource.get() {
        Some(Ok(v)) => Ok(v),
        Some(Err(AppError::NotFound)) => Err(AppError::NotFound),
        _ => Err(AppError::InternalServerError),
    };

    let view = move || {
        info().map(|info| {
            // FIXME the other is generated even if unused...
            let href = format!("/workspace/{}/", &info.workspace.id);
            let other = format!("Workspace {}", &info.workspace.id);
            let desc = info.workspace.description
                .clone()
                .unwrap_or(other);
            view! {
                <h1><a href=href>{desc}</a></h1>
                <div class="workspace-pathinfo">
                    <WorkspaceRepoResultView repo_result=info/>
                </div>
            }
        })
    };

    view! {
        <Suspense fallback=move || view! { <p>"Loading info..."</p> }>
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
                {view}
            </ErrorBoundary>
        </Suspense>
    }
}
