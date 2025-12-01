use std::{
    collections::HashMap,
    sync::Arc,
};
use chrono::{
    TimeZone,
    Utc,
};
use leptos::logging;
use leptos::context::Provider;
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
    ParamSegment,
    SsrMode,
    StaticSegment,
    WildcardSegment,
};
use leptos_sync_ssr::signal::SsrWriteSignal;
use pmrcore::repo::{
    LogEntryInfo,
    PathObjectInfo,
    RepoResult,
    TreeInfo,
};

pub(crate) mod api;

use crate::{
    ac::AccountCtx,
    component::RedirectTS,
    enforcement::{
        EnforcedOk,
        PolicyState,
    },
    error::AppError,
    error_template::ErrorTemplate,
    exposure::api::{
        CreateExposure,
        list_aliased_exposures_for_workspace,
    },
    workspace::api::{
        list_workspaces,
        list_aliased_workspaces,
        get_log_info,
        get_workspace_info,
        workspace_root_policy_state,
        CreateWorkspace,
        Synchronize,
        Workspaces,
    },
    app::{
        portlet::{
            ContentActionCtx,
            ContentActionItem,
        },
        EntityRoot,
        Root,
    },
};

#[component]
pub fn WorkspaceRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/workspace") view=WorkspaceRoot ssr>
            <Route path=StaticSegment("/") view=WorkspaceListing/>
            <Route path=StaticSegment("") view=RedirectTS/>
            <ParentRoute path=StaticSegment(":") view=Outlet>
                <Route path=StaticSegment("add") view=WorkspaceAdd/>
                <ParentRoute path=StaticSegment("id") view=WorkspaceIdRoot>
                    <Route path=StaticSegment("/") view=WorkspaceListing/>
                    <WorkspaceViewRoutes/>
                </ParentRoute>
            </ParentRoute>
            // this should have a default root provided, or at the higher level?
            <WorkspaceViewRoutes/>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
fn WorkspaceViewRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=ParamSegment("id") view=Workspace ssr>
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
    }
    .into_inner()
}

#[component]
pub fn WorkspaceRoot() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    #[cfg(not(feature = "ssr"))]
    {
        let account_ctx = account_ctx.clone();
        on_cleanup(move || account_ctx.cleanup_policy_state());
    }

    let workspaces: Resource<Result<Workspaces, AppError>> = Resource::new(
        move || (),
        move |_| {
            let set_ps = account_ctx.policy_state.write_only();
            async move {
                // The main workspace root will provide the aliased workspaces
                // as a resource, with the notify version done.
                provide_context(set_ps);
                let result = list_aliased_workspaces()
                    .await
                    .map(EnforcedOk::notify_into_inner);
                // ensure the `SsrWriteSignal` is dropped...
                let _ = take_context::<SsrWriteSignal<Option<PolicyState>>>();
                // ... before returning via `?`.
                let mut result = result?;
                result.sort_unstable_by(|a, b| a.entity.description
                    .as_deref()
                    .map(str::to_lowercase)
                    .cmp(&b.entity.description.as_deref().map(str::to_lowercase)));
                Ok(result)
            }
        },
    );

    view! {
        <Title text="Workspace — Physiome Model Repository"/>
        <Provider value=Root::Aliased("/workspace/")>
            <Provider value=workspaces>
                <Outlet/>
            </Provider>
        </Provider>
    }
}

#[component]
pub fn WorkspaceIdRoot() -> impl IntoView {
    // TODO this should be able to reuse the aliased one to convert into
    // standard to potentially save on an additional call, but this isn't
    // typically used so the small bit of bandwidth inefficiency will have
    // to be tolerated for now.
    let workspaces: Resource<Result<Workspaces, AppError>> = Resource::new(
        move || (),
        move |_| {
            async move {
                // The id workspace root will provide the standard list of
                // workspaces as a resource, without the notify version done.
                list_workspaces()
                    .await
                    .map(EnforcedOk::into_inner)
            }
        },
    );

    view! {
        <Provider value=Root::Id("/workspace/:/id/")>
            <Provider value=workspaces>
                <Outlet/>
            </Provider>
        </Provider>
    }
}

fn workspace_root_page_ctx() -> impl IntoView {
    let content_action_ctx = ContentActionCtx::expect();
    #[cfg(not(feature = "ssr"))]
    let account_ctx = expect_context::<AccountCtx>();
    let root = expect_context::<Root>();

    #[cfg(not(feature = "ssr"))]
    on_cleanup({
        let account_ctx = account_ctx.clone();
        let content_action_ctx = content_action_ctx.clone();
        move || {
            account_ctx.cleanup_policy_state();
            content_action_ctx.inner_write_signal().update(
                move |content_actions| if let Some(content_actions) = content_actions {
                    content_actions.update(root, None);
                }
            );
        }
    });

    content_action_ctx.update_with(
        move || {
            async move {
                Some(vec![
                    ContentActionItem {
                        href: format!("{root}:/add"),
                        text: "Add Workspace".to_string(),
                        title: Some("Add a new workspace".to_string()),
                        req_action: Some("create".to_string()),
                    },
                ])
            }
        },
        move |content_actions, new_actions| {
            if let Some(content_actions) = content_actions {
                content_actions.update(root, new_actions);
            } else {
                *content_actions = Some((root.as_ref(), new_actions).into());
            }
        },
    )
}

#[component]
pub fn WorkspaceListing() -> impl IntoView {
    let root = expect_context::<Root>();
    let workspaces = expect_context::<Resource<Result<Workspaces, AppError>>>();

    let workspace_listing = move || {
        Suspend::new(async move {
            workspaces.await.map(|workspaces| workspaces
                .into_iter()
                .map(move |workspace| {
                    view! {
                        <div>
                            // TODO the link should point to the primary alias
                            <div><a href=format!("{root}{}/", workspace.alias)>
                                {workspace.entity.description.unwrap_or_else(
                                    || format!("Workspace {}", workspace.entity.id))}
                            </a></div>
                            <div>{workspace.entity.url}</div>
                            <div>{workspace.entity.long_description.unwrap_or("".to_string())}</div>
                        </div>
                    }
                })
                .collect_view()
            )
        })
    };

    view! {
        <div class="main">
            {workspace_root_page_ctx()}
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
    let account_ctx = expect_context::<AccountCtx>();

    let policy_state = Resource::new(
        move || (),
        move |_| {
            let set_ps = account_ctx.policy_state.write_only();
            async move {
                set_ps.set(workspace_root_policy_state().await.ok());
            }
        },
    );

    view! {
        <div class="main">
            <Suspense>
                {move || Suspend::new(async move {
                    // TODO apply this to control the visibility of the form.
                    policy_state.await
                })}
            </Suspense>
            {workspace_root_page_ctx()}
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
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct WorkspaceParams {
    id: Option<String>,
}

#[component]
pub fn Workspace() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let root = expect_context::<Root>();
    let content_action_ctx = ContentActionCtx::expect();
    let params = use_params::<WorkspaceParams>();

    let entity_root: Memo<EntityRoot> = Memo::new(move |_| {
        root.build_entity_root(params.get()
            .expect("conversion to string must be infallible")
            .id
            .expect("this must be used inside a route with an id parameter")
        )
    });
    provide_context(entity_root);

    let resource = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        move |id| {
            let set_ps = account_ctx.policy_state.write_only();
            async move {
                logging::log!("processing requested workspace {:?}", &id);
                provide_context(set_ps);
                let result = match id {
                    Err(_) => Err(AppError::InternalServerError),
                    Ok(None) => Err(AppError::NotFound),
                    Ok(Some(id)) => {
                        let id = root.build_id(id)?;
                        get_workspace_info(id, None, None)
                            .await
                            .map(EnforcedOk::notify_into_inner)
                            .map_err(AppError::from)
                    }
                };
                let _ = take_context::<SsrWriteSignal<Option<PolicyState>>>();
                result
            }
        }
    );

    #[cfg(not(feature = "ssr"))]
    on_cleanup({
        let content_action_ctx = content_action_ctx.clone();
        move || {
            content_action_ctx.inner_write_signal().update(
                move |content_actions| if let Some(content_actions) = content_actions {
                    let route = format!("{root}{{id}}/");
                    content_actions.update(&route, None);
                }
            );
        }
    });

    provide_context(resource);
    provide_context(params);

    view! {
        {content_action_ctx.update_with(
            move || {
                async move {
                    resource.await.ok().map(|_| {
                        let base_href = entity_root.read();
                        vec![
                            ContentActionItem {
                                href: format!("{base_href}/"),
                                text: "Main View".to_string(),
                                title: Some("Return to the top level workspace view".to_string()),
                                req_action: None,
                            },
                            ContentActionItem {
                                href: format!("{base_href}/log"),
                                text: "History".to_string(),
                                title: None,
                                req_action: None,
                            },
                            ContentActionItem {
                                href: format!("{base_href}/synchronize"),
                                text: "Synchronize".to_string(),
                                title: Some("Synchronize with the stored Git Repository URI".to_string()),
                                req_action: Some("protocol_write".to_string()),
                            },
                        ]
                    })
                }
            },
            move |content_actions, new_actions| {
                let route = format!("{root}{{id}}/");
                if let Some(content_actions) = content_actions {
                    content_actions.update(&route, new_actions);
                } else {
                    *content_actions = Some((route.as_ref(), new_actions).into());
                }
            },
        )}
        <Outlet/>
    }
}

#[component]
pub fn WorkspaceMain() -> impl IntoView {
    let resource = expect_context::<Resource<Result<RepoResult, AppError>>>();
    let entity_root = expect_context::<Memo<EntityRoot>>();

    let workspace_view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let base_href = entity_root.get().to_string();
            view! {
                // render content
                <h1>{info.workspace.description.clone().unwrap_or(
                    format!("Workspace {}", info.workspace.id))}</h1>
                <dl>
                    <dt>"Git Repository URI"</dt>
                    <dd>{info.workspace.url.clone()}</dd>
                    <div class="workspace-pathinfo">
                        <WorkspaceListingView repo_result=info base_href/>
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
fn WorkspaceListingView(
    repo_result: RepoResult,
    #[prop(into)]
    base_href: Arc<str>,
) -> impl IntoView {
    let commit_id = repo_result.commit
        .clone()
        .map(|commit| commit.commit_id)
        .unwrap_or_else(|| "<none>".to_string());
    let path = repo_result.path.clone().unwrap_or_else(|| String::new());
    let pardir = path != "";
    let pardir = {
        let base_href = base_href.clone();
        move || { pardir.then(|| view! {
            <WorkspaceTreeInfoRow
                base_href=base_href.clone()
                commit_id=commit_id.as_str()
                path=path.as_str()
                kind="pardir"
                name=".."/>
        })}
    };

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
                                base_href=base_href
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
fn WorkspaceFileInfoView(
    repo_result: RepoResult,
    #[prop(into)]
    base_href: Arc<str>,
) -> impl IntoView {
    let path = repo_result.path.clone().unwrap_or_else(|| String::new());
    let commit_id = repo_result.commit
        .map(|commit| commit.commit_id)
        .unwrap_or_else(|| "<none>".to_string())
        .clone();
    match repo_result.target {
        Some(PathObjectInfo::FileInfo(ref file_info)) => {
            let href = format!("{base_href}/rawfile/{commit_id}/{path}");
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
fn WorkspaceRepoResultView(
    repo_result: RepoResult,
    #[prop(into)]
    base_href: Arc<str>,
) -> impl IntoView {
    match repo_result.target {
        Some(PathObjectInfo::TreeInfo(_)) =>
            Some(view! { <div><WorkspaceListingView repo_result base_href/></div> }.into_any()),
        Some(PathObjectInfo::FileInfo(_)) =>
            Some(view! { <div><WorkspaceFileInfoView repo_result base_href/></div> }.into_any()),
        _ => None,
    }
}

#[component]
fn WorkspaceTreeInfoRows(
    #[prop(into)]
    base_href: Arc<str>,
    #[prop(into)]
    commit_id: Arc<str>,
    #[prop(into)]
    path: Arc<str>,
    tree_info: TreeInfo,
) -> impl IntoView {
    tree_info.entries
        .into_iter()
        .map(|info| view! {
            <WorkspaceTreeInfoRow
                base_href=base_href.clone()
                commit_id=commit_id.clone()
                path=path.clone()
                kind=info.kind
                name=info.name/>
        })
        .collect_view()
}

#[component]
fn WorkspaceTreeInfoRow(
    #[prop(into)]
    base_href: Arc<str>,
    #[prop(into)]
    commit_id: Arc<str>,
    #[prop(into)]
    path: Arc<str>,
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
            format!("{name}/")
        } else {
            format!("{name}")
        })
    };
    let href = format!("{base_href}/file/{commit_id}/{path_name}");
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
    let root = expect_context::<Root>();
    let entity_root = expect_context::<Memo<EntityRoot>>();
    logging::log!("in <WorkspaceCommitPath>");
    let workspace_params = expect_context::<Memo<Result<WorkspaceParams, ParamsError>>>();
    let params = use_params::<WorkspaceCommitPathParams>();

    let resource = Resource::new_blocking(
        move || (
            workspace_params.get().map(|p| p.id),
            params.get().map(|p| (p.commit, p.path)),
        ),
        move |p| async move {
            match p {
                (Ok(Some(id)), Ok((commit, path))) => {
                    let id = root.build_id(id)?;
                    get_workspace_info(id, commit, path).await
                        .map(EnforcedOk::notify_into_inner)
                        .map_err(AppError::from)
                }
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let base_href = entity_root.get().to_string();
            let desc = info.workspace.description
                .clone()
                .unwrap_or_else(
                    || format!("Workspace {}", &info.workspace.id)
                );
            view! {
                <h1><a href=base_href.clone()>{desc}</a></h1>
                <div class="workspace-pathinfo">
                    <WorkspaceRepoResultView repo_result=info base_href/>
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
    let root = expect_context::<Root>();
    let entity_root = expect_context::<Memo<EntityRoot>>();

    let repo_result = expect_context::<Resource<Result<RepoResult, AppError>>>();
    let log_info = Resource::new_blocking(
        move || workspace_params.get().map(|p| p.id),
        move |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => {
                    let id = root.build_id(id)?;
                    get_log_info(id)
                        .await
                        .map(EnforcedOk::notify_into_inner)
                        .map_err(AppError::from)
                }
            }
        }
    );

    let exposure_map = Resource::new_blocking(
        move || workspace_params.get().map(|p| p.id),
        move |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => {
                    let id = root.build_id(id)?;
                    let mut result = HashMap::new();
                    for entry in list_aliased_exposures_for_workspace(id).await?.into_iter() {
                        result.entry(entry.entity.commit_id)
                            .or_insert_with(Vec::new)
                            .push((
                                format!("/exposure/{}/", entry.alias),
                                entry.entity.description.unwrap_or(format!("<Exposure {}>", entry.entity.id)),
                            ))
                    }
                    Ok(result)
                }
            }
        }
    );

    let view = move || Suspend::new(async move {
        let log_info = log_info.await?;
        let mut exposure_map = exposure_map.await?;
        repo_result.await.map(|_| {
            let href = entity_root.read();
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
                                        <a href=format!("{href}/file/{commit_id}/")>"[files]"</a>
                                        <a href=format!("{href}/create_exposure/{commit_id}/")>"[create_exposure]"</a>
                                    </td>
                                    <td>{
                                        exposure_map.remove(&commit_id).map(|entries| {
                                            entries.into_iter()
                                                .map(|(href, description)| view! {
                                                    // FIXME <div> tags are placeholder wrapper
                                                    <div><a href=href>{description}</a></div>
                                                })
                                                .collect_view()
                                        })
                                    }</td>
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
    let root = expect_context::<Root>();

    let resource = Resource::new_blocking(
        move || (
            workspace_params.get().map(|p| p.id),
            params.get().map(|p| p.commit),
        ),
        move |p| async move {
            match p {
                (Ok(Some(id)), Ok(commit)) => {
                    let id = root.build_id(id)?;
                    get_workspace_info(id, commit, None).await
                        .map(EnforcedOk::notify_into_inner)
                        .map_err(AppError::from)
                }
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
                    <input type="hidden" name="workspace_id" value=info.workspace.id/>
                    <input type="hidden" name="commit_id" value=commit_id/>
                    <button type="submit">"Create Exposure"</button>
                </ActionForm>
                <div>
                    {move || {
                        match action.value().get() {
                            Some(Err(e)) => Some(view! {
                                <p class="standard error">{format!("Error: {e:?}")}</p>
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
