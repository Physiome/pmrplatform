use std::sync::Arc;
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
    MatchNestedRoutes,
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

mod api;

use crate::{
    ac::AccountCtx,
    component::RedirectTS,
    enforcement::{
        EnforcedOk,
        PolicyState,
    },
    error::AppError,
    error_template::ErrorTemplate,
    exposure::api::CreateExposure,
    workspace::api::{
        list_workspaces,
        list_aliased_workspaces,
        get_log_info,
        get_workspace_info,
        workspace_root_policy_state,
        CreateWorkspace,
        Synchronize,
    },
    app::{
        portlet::{
            ContentActionCtx,
            ContentActionItem,
        },
        id::Id,
        Root,
    },
};

#[component]
pub fn WorkspaceRoutes() -> impl MatchNestedRoutes + Clone {
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
fn WorkspaceViewRoutes() -> impl MatchNestedRoutes + Clone {
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
    #[cfg(not(feature = "ssr"))]
    {
        let account_ctx = expect_context::<AccountCtx>();
        on_cleanup({
            move || account_ctx.cleanup_policy_state()
        });
    }

    view! {
        <Title text="Workspace — Physiome Model Repository"/>
        <Provider value=Root::Aliased("/workspace/")>
            <Outlet/>
        </Provider>
    }
}

#[component]
pub fn WorkspaceIdRoot() -> impl IntoView {
    view! {
        <Provider value=Root::Id("/workspace/:/id/")>
            <Outlet/>
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
    let account_ctx = expect_context::<AccountCtx>();
    let root = expect_context::<Root>();

    // TODO when appropriate, consider moving this to the root as a context,
    // so that the policy_state may potentially be shared, but this can also
    // cause a conflict given the design is that there's really only one
    // policy active at a time.  That also require the policy to be scoped
    // or named much like content actions.
    let workspaces = Resource::new(
        move || (),
        move |_| {
            let set_ps = account_ctx.policy_state.write_only();
            async move {
                provide_context(set_ps);
                let result = match root {
                    Root::Id(_) => list_workspaces().await,
                    Root::Aliased(_) => list_aliased_workspaces().await,
                };
                match result {
                    Ok(ref result) => logging::log!("loaded {} workspace entries", result.inner.len()),
                    Err(_) => logging::log!("error loading workspaces"),
                };
                let result = result.map(EnforcedOk::notify_into);
                let _ = take_context::<SsrWriteSignal<Option<PolicyState>>>();
                result
            }
        },
    );

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
                            .map(EnforcedOk::notify_into)
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
                    resource.await.ok().map(|info| {
                        let resource = format!("{root}{}/", info.workspace.id);
                        vec![
                            ContentActionItem {
                                href: resource.clone(),
                                text: "Main View".to_string(),
                                title: Some("Return to the top level workspace view".to_string()),
                                req_action: None,
                            },
                            ContentActionItem {
                                href: format!("{resource}log"),
                                text: "History".to_string(),
                                title: None,
                                req_action: None,
                            },
                            ContentActionItem {
                                href: format!("{resource}synchronize"),
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
        <Title text="Workspace — Physiome Model Repository"/>
        <Outlet/>
    }
}

#[component]
pub fn WorkspaceMain() -> impl IntoView {
    let resource = expect_context::<Resource<Result<RepoResult, AppError>>>();
    let workspace_params = expect_context::<Memo<Result<WorkspaceParams, ParamsError>>>();
    let root = use_context::<Root>().unwrap_or(Root::Aliased("/workspace/"));

    let workspace_view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let base_href = root.build_href(workspace_params.get()
                .expect("this should be a valid id")
                .id
                .expect("this should be a valid id")
            );
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
                        .map(EnforcedOk::notify_into)
                        .map_err(AppError::from)
                }
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view = move || Suspend::new(async move {
        resource.await.map(|info| {
            let base_href = root.build_href(workspace_params.get()
                .expect("this should be a valid id")
                .id
                .expect("this should be a valid id")
            );
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
                        .map(EnforcedOk::notify_into)
                        .map_err(AppError::from)
                }
            }
        }
    );

    let view = move || Suspend::new(async move {
        let log_info = log_info.await?;
        repo_result.await.map(|info| {
            let href = root.build_href(workspace_params.get()
                .expect("this should be a valid id")
                .id
                .expect("this should be a valid id")
            );
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
                        .map(EnforcedOk::notify_into)
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
                    <input type="hidden" name="id" value=info.workspace.id/>
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
