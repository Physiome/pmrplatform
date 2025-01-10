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
    params::Params,
    MatchNestedRoutes,
    ParamSegment,
    SsrMode,
    StaticSegment,
    WildcardSegment,
};
use pmrcore::exposure;
use std::str::FromStr;

pub mod api;

use crate::{
    ac::AccountCtx,
    component::{Redirect, RedirectTS},
    error::AppError,
    error_template::ErrorTemplate,
    enforcement::{
        EnforcedOk,
        PolicyState,
    },
    exposure::api::{
        list,
        get_exposure_info,
        resolve_exposure_path,
        wizard,
        ExposureInfo,
    },
    view::{
        EFView,
        ExposureFileView,
    },
    app::portlet::{
        ContentActionCtx,
        ContentActionItem,
        ExposureSourceCtx,
        ExposureSourceItem,
        NavigationCtx,
        NavigationItem,
        ViewsAvailableCtx,
    },
};

mod types {
    use pmrcore::exposure::{
        ExposureFile,
        ExposureFileView,
    };

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub enum ResolvedExposurePath {
        Target(ExposureFile, Result<(ExposureFileView, Option<String>), Vec<String>>),
        Redirect(String),
    }
}

pub use types::ResolvedExposurePath;

#[component]
pub fn ExposureRoutes() -> impl MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/exposure") view=ExposureRoot ssr>
            <Route path=StaticSegment("/") view=ExposureListing/>
            <Route path=StaticSegment("") view=RedirectTS/>
            <ParentRoute path=ParamSegment("id") view=Exposure>
                <Route path=StaticSegment("/") view=ExposureMain/>
                <Route path=StaticSegment("") view=RedirectTS/>
                <Route path=(StaticSegment("+"), StaticSegment("wizard")) view=Wizard/>
                <Route path=WildcardSegment("path") view=ExposureFile/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn ExposureRoot() -> impl IntoView {
    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Outlet/>
    }
}

#[component]
pub fn ExposureListing() -> impl IntoView {
    let exposures = Resource::new_blocking(
        move || (),
        move |_| async move {
            let result = list().await;
            match result {
                Ok(ref result) => logging::log!("{}", result.inner.len()),
                Err(_) => logging::log!("error loading exposures"),
            };
            result.map(EnforcedOk::notify_into)
        },
    );
    let exposure_listing = move || Suspend::new(async move {
        exposures.await.map(|exposures| exposures
            .into_iter()
            .map(move |exposure| view! {
                <div>
                    <div><a href=format!("/exposure/{}/", exposure.id)>
                        "Exposure "{exposure.id}
                    </a></div>
                    <div>{exposure.description}</div>
                </div>
            })
            .collect_view()
        )
    });

    view! {
        <div class="main">
            <h1>"Listing of exposures"</h1>
            <div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {exposure_listing}
                </ErrorBoundary>
            </Transition>
            </div>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureParams {
    id: Option<i64>,
}

#[component]
pub fn Exposure() -> impl IntoView {
    on_cleanup(move || {
        leptos::logging::log!("on_cleanup <Exposure>");
        use_context::<WriteSignal<ExposureSourceCtx>>()
            .map(|ctx| ctx.update(ExposureSourceCtx::clear));
        use_context::<WriteSignal<NavigationCtx>>()
            .map(|ctx| ctx.update(NavigationCtx::clear));
        // FIXME when ContentAction is introduced here, use that for implicit cleanup.
        // if let Some(account_ctx) = use_context::<AccountCtx>() {
        //     leptos::logging::log!("used context AccountCtx to set_ps");
        //     account_ctx.set_ps.update(|ctx| *ctx = PolicyState::default());
        // }
    });
    let params = use_params::<ExposureParams>();
    provide_context(Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => get_exposure_info(id)
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(AppError::from),
                _ => Err(AppError::NotFound),
            }
        }
    ));
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();

    let portlets = move || {
        Suspend::new(async move {
            let exposure_info = exposure_info.await;
            let resource = exposure_info.as_ref().ok().map(|info| {
                format!("/exposure/{}/", info.exposure.id)
            });
            expect_context::<WriteSignal<ContentActionCtx>>()
                .update(|ctx| ctx.replace(resource
                    .map(|resource| {
                        on_cleanup(move || {
                            expect_context::<WriteSignal<ContentActionCtx>>().update(|ctx| {
                                ctx.reset_for("/exposure/{id}/");
                            });
                        });

                        let mut actions = vec![];
                        actions.push(ContentActionItem {
                            href: resource.clone(),
                            text: "Exposure Top".to_string(),
                            title: None,
                            req_action: None,
                        });
                        actions.push(ContentActionItem {
                            href: format!("{resource}+/wizard"),
                            text: "Wizard".to_string(),
                            title: Some("Build this exposure".to_string()),
                            req_action: Some("edit".to_string()),
                        });
                        ContentActionCtx::new("/exposure/{id}/".into(), actions)
                    })
                    .unwrap_or_default()
                ));
            expect_context::<WriteSignal<ExposureSourceCtx>>()
                .update(|ctx| ctx.replace(exposure_info.as_ref()
                    .map(|info| {
                        logging::log!("building ExposureSourceItem");
                        ExposureSourceItem {
                            commit_id: info.exposure.commit_id.clone(),
                            workspace_id: info.exposure.workspace_id.to_string(),
                            // TODO put in the workspace title.
                            workspace_title: info.workspace.description.clone().unwrap_or(
                                format!("Workspace {}", info.exposure.workspace_id)),
                        }.into()
                    })
                    .ok()
                    .into()
                ));
            expect_context::<WriteSignal<NavigationCtx>>()
                .update(|ctx| ctx.replace(exposure_info
                    .map(|info| {
                        let exposure_id = info.exposure.id;
                        logging::log!("building NavigationCtx");
                        // TODO should derive from exposure.files when it contains title/description
                        info.files
                            .into_iter()
                            .filter_map(move |(file, flag)| {
                                flag.then(|| {
                                    let href = format!("/exposure/{exposure_id}/{file}/");
                                    let text = file.clone();
                                    let title = None;
                                    NavigationItem { href, text, title }
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .ok()
                    .into()
                ));
        })
    };

    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Suspense>
            {portlets}
        </Suspense>
        <Outlet/>
    }
}

#[component]
pub fn ExposureFileListing(id: i64, files: Vec<(String, bool)>) -> impl IntoView {
    view! {
        <ul>{files.into_iter()
            .map(|(file, flag)| view! {
                <li>
                    <a href=format!("/exposure/{id}/{file}")>
                        {file.clone()}
                    </a>
                    " - "{flag.then(|| view! {
                        <a href=format!("/exposure/{id}/{file}/")>
                            {flag}
                        </a>
                    }.into_any()).unwrap_or("false".into_any())}
                </li>
            })
            .collect_view()
        }</ul>
    }
}

#[component]
pub fn ExposureMain() -> impl IntoView {
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();
    let file_listing = move || Suspend::new(async move {
        exposure_info.await.map(|info| view! {
            <h1>"Viewing exposure "{info.exposure.id}</h1>
            <ExposureFileListing id=info.exposure.id files=info.files/>
        })
    });

    view! {
        <div class="main">
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {file_listing}
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureFileParams {
    path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ViewPath(pub Option<String>);

#[component]
pub fn ExposureFile() -> impl IntoView {
    on_cleanup(|| {
        use_context::<WriteSignal<ViewsAvailableCtx>>()
            .map(|ctx| ctx.update(ViewsAvailableCtx::clear));
    });
    let params = use_params::<ExposureFileParams>();
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();
    let file = Resource::new_blocking(
        move || params.get().map(|p| p.path),
        move |p| async move {
            match (exposure_info.await, p) {
                (Ok(info), Ok(Some(path))) => resolve_exposure_path(info.exposure.id, path.clone())
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view_key_entry = move |(ef, view_key): (&exposure::ExposureFile, String)| view! {
        <li>
            <a href=format!("/exposure/{}/{}/{}", ef.exposure_id, ef.workspace_file_path, view_key)>
                {view_key.clone()}
            </a>
        </li>
    };

    let ep_view = move || Suspend::new(async move {
        match file.await
            .map_err(|_| AppError::NotFound)
        {
            // TODO figure out how to redirect to the workspace.
            Ok(ResolvedExposurePath::Target(ef, Ok((efv, view_path)))) => {
                expect_context::<WriteSignal<ViewsAvailableCtx>>()
                    .update(|ctx| ctx.replace((&ef).into()));
                let view_key = efv.view_key.clone();
                let view_key = EFView::from_str(&view_key
                    .expect("API failed to produce a fully formed ExposureFileView")
                )?;
                provide_context(ef);
                provide_context(efv);
                provide_context(ViewPath(view_path));
                Ok(view! {
                    <ExposureFileView view_key/>
                }.into_any())
            }
            Ok(ResolvedExposurePath::Target(ef, Err(view_keys))) => {
                expect_context::<WriteSignal<ViewsAvailableCtx>>()
                    .update(|ctx| ctx.replace((&ef).into()));
                Ok(view! {
                    <h1>
                        "Exposure "{ef.exposure_id}
                        " - ExposureFile "{ef.workspace_file_path.clone()}
                        " - Listing of all views"
                    </h1>
                    <ul>{
                        view_keys.into_iter()
                            .map(|k| view_key_entry((&ef, k)))
                            .collect_view()
                    }</ul>
                }.into_any())
            }
            Ok(ResolvedExposurePath::Redirect(path)) => {
                Ok(view! { <Redirect path show_link=true/> }.into_any())
            }
            _ => Err(AppError::NotFound),
        }
    });

    view! {
        <div class="main">
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {ep_view}
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[component]
pub fn Wizard() -> impl IntoView {
    let params = use_params::<ExposureParams>();
    let wizard_res = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => wizard(id)
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(AppError::from),
                _ => Err(AppError::NotFound),
            }
        }
    );

    let wizard_view = move || Suspend::new(async move {
        wizard_res.await.map(|info| {
            let unassigned_files = info.files.iter()
                .filter_map(|(name, status)| status.is_none().then_some(name.as_str()))
                .collect::<Vec<_>>();
            let files = unassigned_files.iter()
                .map(|name| view! { <li>{name.to_string()}</li>} )
                .collect_view();
            view! {
                <ul>{files}</ul>
            }
        })
    });

    view! {
        <h1>"Exposure Wizard"</h1>
        <Transition>
            {wizard_view}
        </Transition>
    }
}
