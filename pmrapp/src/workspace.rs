use chrono::{
    TimeZone,
    Utc,
};
use leptos::logging;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{
        ParentRoute,
        Route,
    },
    hooks::use_params,
    nested_router::Outlet,
    params::{
        Params,
        ParamsError,
    },
    MatchNestedRoutes,
    ParamSegment,
    SsrMode,
    StaticSegment,
    WildcardSegment,
};
use pmrcore::repo::{
    LogEntryInfo,
    PathObjectInfo,
    RepoResult,
    TreeInfo,
};

mod api;

use crate::ac::AccountCtx;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::component::RedirectTS;
use crate::workspace::api::{
    list_workspaces,
    get_log_info,
    get_workspace_info,
    CreateWorkspace,
    Synchronize,
};
use crate::exposure::api::CreateExposure;
use crate::app::portlet::{
    ContentActionCtx,
    ContentActionItem,
};

#[component]
pub fn WorkspaceRoutes() -> impl MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/workspace") view=WorkspaceRoot ssr>
            <Route path=StaticSegment("/") view=WorkspaceListing/>
            <Route path=StaticSegment("") view=RedirectTS/>
            <Route path=(StaticSegment("+"), StaticSegment("add")) view=WorkspaceAdd/>
            <ParentRoute path=ParamSegment("id") view=Workspace>
                <Route path=StaticSegment("/") view=WorkspaceMain/>
                <Route path=StaticSegment("") view=RedirectTS/>
                <Route path=StaticSegment("synchronize") view=WorkspaceSynchronize/>
                <Route
                    path=(StaticSegment("file"), ParamSegment("commit"), WildcardSegment("path"),)
                    view=WorkspaceCommitPath
                    />
                <Route
                    path=(StaticSegment("create_exposure"), ParamSegment("commit"),)
                    view=WorkspaceCreateExposure
                    />
                <Route path=StaticSegment("log") view=WorkspaceLog/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn WorkspaceRoot() -> impl IntoView {
    view! {
        <Title text="Workspace — Physiome Model Repository"/>
        <Outlet/>
    }
}

fn workspace_root_page_ctx() {
    on_cleanup(move || {
        let account_ctx = expect_context::<AccountCtx>();
        account_ctx.set_resource.set(None);
        expect_context::<WriteSignal<Option<ContentActionCtx>>>().set(None);
    });
    // doing it here is a way to ensure this is set reactively, however this isn't reacting
    // to the account changes, and it leaves a compulsory empty bar when this action isn't
    // available.
    // this should be a push? children elements will need to also use this and that's a
    // conflict.
    let account_ctx = expect_context::<AccountCtx>();
    let set_resource = account_ctx.set_resource.clone();
    set_resource.set(Some("/workspace/".to_string()));
    expect_context::<WriteSignal<Option<ContentActionCtx>>>()
        .set(Some(ContentActionCtx(Some(vec![
            ContentActionItem {
                href: format!("/workspace/+/add"),
                text: "Add Workspace".to_string(),
                title: Some("Add a new workspace".to_string()),
                req_action: Some("create".to_string()),
            },
        ]))));
}

#[component]
pub fn WorkspaceListing() -> impl IntoView {
    let workspaces = Resource::new(
        move || (),
        move |_| async move {
            let result = list_workspaces().await;
            match result {
                Ok(ref result) => logging::log!("loaded {} workspace entries", result.len()),
                Err(_) => logging::log!("error loading workspaces"),
            };
            result
        },
    );

    let workspace_listing = move || {
        Suspend::new(async move {
            workspaces.await.map(|workspaces| workspaces
                .into_iter()
                .map(move |workspace| {
                    view! {
                        <div>
                            <div><a href=format!("/workspace/{}/", workspace.id)>
                                {workspace.description.unwrap_or_else(
                                    || format!("Workspace {}", workspace.id))}
                            </a></div>
                            <div>{workspace.url}</div>
                            <div>{workspace.long_description.unwrap_or("".to_string())}</div>
                        </div>
                    }
                })
                .collect_view()
            )
        })
    };

    view! {
        <div class="main">
            {workspace_root_page_ctx}
            <h1>"Listing of workspaces"</h1>
            <div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {workspace_listing}
                </ErrorBoundary>
            </Transition>
            </div>
        </div>
    }
}

#[component]
pub fn WorkspaceAdd() -> impl IntoView {
    let action = ServerAction::<CreateWorkspace>::new();

    view! {
        {workspace_root_page_ctx}
        <h1>"Add a workspace"</h1>
        <ActionForm attr:class="standard" action=action>
            <div>
                <label for="uri">"Remote Git URI"</label>
                <input type="text" name="uri" required/>
            </div>
            <div>
                <label for="description">"Description"</label>
                <input type="text" name="description" required/>
            </div>
            <div>
                <label for="long_description">"Long Description"</label>
                <input type="text" name="long_description"/>
            </div>
            <div>
                <input type="submit" value="Add Workspace"/>
            </div>
        </ActionForm>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct WorkspaceParams {
    id: Option<i64>,
}

#[component]
pub fn Workspace() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let set_resource = account_ctx.set_resource.clone();

    on_cleanup(move || {
        expect_context::<WriteSignal<Option<ContentActionCtx>>>().set(None);
        set_resource.set(None);
    });

    let params = use_params::<WorkspaceParams>();
    let resource: Resource<Result<RepoResult, AppError>> = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |id| async move {
            logging::log!("processing requested workspace {:?}", &id);
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => get_workspace_info(id, None, None)
                    .await
                    .map_err(AppError::from),
            }
        }
    );
    let portlets = move || {
        let set_resource = account_ctx.set_resource.clone();

        Suspend::new(async move {
            let repo_result = resource.await;
            // TODO see if a check on existing value may avoid setting thus avoid a refetch upon hydration?
            let resource = repo_result.as_ref().ok().map(|info| {
                format!("/workspace/{}/", info.workspace.id)
            });
            set_resource.set(resource.clone());

            // This supposedly normally may be written outside of a suspense, but this does
            // depend on the above resource (repo_result), as the future work may have this
            // be loaded as an alias and the resource need to be the canonical URI.  Leaving
            // this for now in here.
            expect_context::<WriteSignal<Option<ContentActionCtx>>>()
                .set({
                    let mut actions = vec![];
                    if let Some(resource) = resource {
                        actions.push(ContentActionItem {
                            href: resource.clone(),
                            text: "Main View".to_string(),
                            title: Some("Return to the top level workspace view".to_string()),
                            req_action: None,
                        });
                        actions.push(ContentActionItem {
                            href: format!("{resource}log"),
                            text: "History".to_string(),
                            title: None,
                            req_action: None,
                        });
                        actions.push(ContentActionItem {
                            href: format!("{resource}synchronize"),
                            text: "Synchronize".to_string(),
                            title: Some("Synchronize with the stored Git Repository URI".to_string()),
                            req_action: Some("protocol_write".to_string()),
                        });
                    }
                    Some(ContentActionCtx(Some(actions)))
                })
        })
    };

    provide_context(resource);
    provide_context(params);

    view! {
        <Title text="Workspace — Physiome Model Repository"/>
        <Suspense>
            {portlets}
        </Suspense>
        <Outlet/>
    }
}

#[component]
pub fn WorkspaceMain() -> impl IntoView {
    let resource = expect_context::<Resource<Result<RepoResult, AppError>>>();

    let workspace_view = move || Suspend::new(async move {
        resource.await.map(|info| {
            view! {
                // render content
                <h1>{info.workspace.description.clone().unwrap_or(
                    format!("Workspace {}", info.workspace.id))}</h1>
                <dl>
                    <dt>"Git Repository URI"</dt>
                    <dd>{info.workspace.url.clone()}</dd>
                    <div class="workspace-pathinfo">
                        <WorkspaceListingView repo_result=info/>
                    </div>
                </dl>
            }
        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading workspace..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {workspace_view}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn WorkspaceSynchronize() -> impl IntoView {
    let resource = expect_context::<Resource<Result<RepoResult, AppError>>>();
    let action = ServerAction::<Synchronize>::new();

    Effect::new(move |_| {
        if action.version().get() > 0 {
            resource.refetch();
        }
    });

    let workspace_view = move || Suspend::new(async move {
        resource.await.map(|info| {
            view! {
                // render content
                <h1>"Synchronize Workspace: "{info.workspace.description.clone().unwrap_or(
                    format!("Workspace {}", info.workspace.id))}</h1>
                <ActionForm action=action>
                    <input type="hidden" name="id" value=info.workspace.id/>
                    <button type="submit">"Synchronize"</button>
                </ActionForm>
            }
        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading workspace..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {workspace_view}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn WorkspaceListingView(repo_result: RepoResult) -> impl IntoView {
    let workspace_id = repo_result.workspace.id;
    let commit_id = repo_result.commit
        .clone()
        .map(|commit| commit.commit_id)
        .unwrap_or_else(|| "<none>".to_string());
    let path = repo_result.path.clone().unwrap_or_else(|| String::new());
    let pardir = path != "";
    let pardir = move || { pardir.then(|| view! {
        <WorkspaceTreeInfoRow
            workspace_id=workspace_id
            commit_id=commit_id.as_str()
            path=path.as_str()
            kind="pardir"
            name=".."/>
    })};

    let path = repo_result.path.clone().unwrap_or_else(|| String::new());
    let commit_id = repo_result.commit
        .clone()
        .map(|commit| commit.commit_id)
        .unwrap_or_else(|| "<none>".to_string());
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
                    Some(PathObjectInfo::TreeInfo(tree_info)) =>
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
fn WorkspaceFileInfoView(repo_result: RepoResult) -> impl IntoView {
    let path = repo_result.path.clone().unwrap_or_else(|| String::new());
    let commit_id = repo_result.commit
        .map(|commit| commit.commit_id)
        .unwrap_or_else(|| "<none>".to_string())
        .clone();
    match repo_result.target {
        Some(PathObjectInfo::FileInfo(ref file_info)) => {
            let href = format!(
                "/workspace/{}/rawfile/{}/{}",
                &repo_result.workspace.id,
                &commit_id,
                &path,
            );
            let info = format!("{:?}", file_info);
            Some(view! {
                <div>
                <div>{info}</div>
                <div>
                    <a href=href.clone() target="_self">"download"</a>
                </div>
                {
                    (file_info.mime_type[..5] == *"image").then(||
                        view! {
                            <div>
                                <p>"Preview"</p>
                                <img src=href.clone() />
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
fn WorkspaceRepoResultView(repo_result: RepoResult) -> impl IntoView {
    match repo_result.target {
        Some(PathObjectInfo::TreeInfo(_)) =>
            Some(view! { <div><WorkspaceListingView repo_result/></div> }.into_any()),
        Some(PathObjectInfo::FileInfo(_)) =>
            Some(view! { <div><WorkspaceFileInfoView repo_result/></div> }.into_any()),
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
        // FIXME this assumes the incoming path has the trailing slash set.
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
pub struct WorkspaceCommitParams {
    commit: Option<String>,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct WorkspaceCommitPathParams {
    commit: Option<String>,
    path: Option<String>,
}

#[component]
pub fn WorkspaceCommitPath() -> impl IntoView {
    logging::log!("in <WorkspaceCommitPath>");
    let workspace_params = expect_context::<Memo<Result<WorkspaceParams, ParamsError>>>();
    let params = use_params::<WorkspaceCommitPathParams>();

    let resource = Resource::new_blocking(
        move || (
            workspace_params.get().map(|p| p.id),
            params.get().map(|p| (p.commit, p.path)),
        ),
        |p| async move {
            match p {
                (Ok(Some(id)), Ok((commit, path))) => get_workspace_info(id, commit, path).await
                    .map_err(AppError::from),
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let href = format!("/workspace/{}/", &info.workspace.id);
            let desc = info.workspace.description
                .clone()
                .unwrap_or_else(
                    || format!("Workspace {}", &info.workspace.id)
                );
            view! {
                <h1><a href=href>{desc}</a></h1>
                <div class="workspace-pathinfo">
                    <WorkspaceRepoResultView repo_result=info/>
                </div>
            }
        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading info..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn WorkspaceLog() -> impl IntoView {
    let workspace_params = expect_context::<Memo<Result<WorkspaceParams, ParamsError>>>();

    let repo_result = expect_context::<Resource<Result<RepoResult, AppError>>>();
    let log_info = Resource::new_blocking(
        move || workspace_params.get().map(|p| p.id),
        |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => get_log_info(id)
                    .await
                    .map_err(AppError::from),
            }
        }
    );

    let view = move || Suspend::new(async move {
        let log_info = log_info.await?;
        repo_result.await.map(|info| {
            let href = format!("/workspace/{}/", &info.workspace.id);
            view! {
                <table class="log-listing">
                    <thead>
                        <tr>
                            <th>"Date"</th>
                            <th>"Author"</th>
                            <th>"Log"</th>
                            <th>"Options"</th>
                            <th>"Exposure"</th>
                        </tr>
                    </thead>
                    <tbody>{
                        log_info.entries
                            .into_iter()
                            .map(|LogEntryInfo {
                                commit_timestamp,
                                author,
                                message,
                                commit_id,
                                ..
                            }| view! {
                                <tr>
                                    <td>{
                                        Utc.timestamp_opt(commit_timestamp, 0)
                                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                                            .single()
                                            .unwrap_or_else(|| "????-??-??".to_string())
                                    }</td>
                                    <td>{author.clone()}</td>
                                    <td>{message.clone()}</td>
                                    <td>
                                        <a href=format!("{href}file/{commit_id}/")>"[files]"</a>
                                        <a href=format!("{href}create_exposure/{commit_id}/")>"[create_exposure]"</a>
                                    </td>
                                    <td></td>
                                </tr>
                            })
                            .collect_view()
                    }</tbody>
                </table>
            }

        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading info..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn WorkspaceCreateExposure() -> impl IntoView {
    let workspace_params = expect_context::<Memo<Result<WorkspaceParams, ParamsError>>>();
    let params = use_params::<WorkspaceCommitParams>();
    let action = ServerAction::<CreateExposure>::new();

    let resource = Resource::new_blocking(
        move || (
            workspace_params.get().map(|p| p.id),
            params.get().map(|p| p.commit),
        ),
        |p| async move {
            match p {
                (Ok(Some(id)), Ok(commit)) => get_workspace_info(id, commit, None).await
                    .map_err(AppError::from),
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let desc = info.workspace.description
                .clone()
                .unwrap_or_else(
                    || format!("Workspace {}", &info.workspace.id)
                );
            let commit_id = info
                .commit
                .expect("commit shouldn't be missing here!")
                .commit_id;
            view! {
                <h1>"Creating Exposure for "{desc}" at commit "{commit_id.clone()}</h1>
                <ActionForm attr:class="standard" action=action>
                    <input type="hidden" name="id" value=info.workspace.id/>
                    <input type="hidden" name="commit_id" value=commit_id/>
                    <button type="submit">"Create Exposure"</button>
                </ActionForm>
                <div>
                    {move || {
                        match action.value().get() {
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
            }
        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading info..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}
