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

use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::component::{Redirect, RedirectTS};
use crate::exposure::api::{
    list,
    get_exposure_info,
    resolve_exposure_path,
    ExposureInfo,
};
use crate::view::{
    EFView,
    ExposureFileView,
};
use crate::app::portlet::{
    ExposureSourceCtx,
    ExposureSourceItem,
    NavigationCtx,
    NavigationItem,
    ViewsAvailableCtx,
};

#[component]
pub fn ExposureRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/exposure") view=ExposureRoot ssr>
            <Route path=StaticSegment("/") view=ExposureListing/>
            <ParentRoute path=ParamSegment("id") view=Exposure>
                <Route path=StaticSegment("/") view=ExposureMain/>
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
    // TODO figure out what kind of navigation needed here.
    expect_context::<WriteSignal<Option<ExposureSourceCtx>>>().set(None);
    expect_context::<WriteSignal<Option<NavigationCtx>>>().set(None);
    expect_context::<WriteSignal<Option<ViewsAvailableCtx>>>().set(None);

    let exposures = Resource::new(
        move || (),
        move |_| async move {
            let result = list().await;
            match result {
                Ok(ref result) => logging::log!("{}", result.len()),
                Err(_) => logging::log!("error loading exposures"),
            };
            result
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
        <RedirectTS />
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
    let params = use_params::<ExposureParams>();
    provide_context(Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => get_exposure_info(id)
                    .await
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::NotFound),
            }
        }
    ));

    let portlets = move || Suspend::new(async move {
        let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>()
            .await;
        expect_context::<WriteSignal<Option<ExposureSourceCtx>>>()
            .set(exposure_info.as_ref().map(|info| {
                ExposureSourceItem {
                    commit_id: info.exposure.commit_id.clone(),
                    workspace_id: info.exposure.workspace_id.to_string(),
                    // TODO put in the workspace title.
                    workspace_title: info.workspace.description.clone().unwrap_or(
                        format!("Workspace {}", info.exposure.workspace_id)),
                }.into()
            }).ok());
        expect_context::<WriteSignal<Option<NavigationCtx>>>()
            .set(exposure_info.map(|info| {
                let exposure_id = info.exposure.id;
                // TODO should derive from exposure.files when it contains title/description
                NavigationCtx(Some(info.files
                    .into_iter()
                    .filter_map(move |(file, flag)| {
                        flag.then(|| {
                            let href = format!("/exposure/{exposure_id}/{file}/");
                            let text = file.clone();
                            let title = None;
                            NavigationItem { href, text, title }
                        })
                    })
                    .collect::<Vec<_>>()))
            }).ok());
    });

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
    expect_context::<WriteSignal<Option<ViewsAvailableCtx>>>().set(None);

    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();
    let file_listing = move || Suspend::new(async move {
        exposure_info.await.map(|info| view! {
            <h1>"Viewing exposure "{info.exposure.id}</h1>
            <ExposureFileListing id=info.exposure.id files=info.files/>
        })
    });

    view! {
        <RedirectTS/>
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
    let params = use_params::<ExposureFileParams>();
    let file = Resource::new_blocking(
        move || params.get().map(|p| p.path),
        |p| async move {
            let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>().await;
            match (exposure_info, p) {
                (Ok(info), Ok(Some(path))) => resolve_exposure_path(info.exposure.id, path.clone())
                    .await
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
        match file.await {
            // TODO figure out how to redirect to the workspace.
            Ok(Ok((ef, Ok((efv, view_path))))) => {
                expect_context::<WriteSignal<Option<ViewsAvailableCtx>>>()
                    .set(Some((&ef).into()));
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
            Ok(Ok((ef, Err(view_keys)))) => {
                expect_context::<WriteSignal<Option<ViewsAvailableCtx>>>()
                    .set(Some((&ef).into()));
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
            },
            Ok(Err(e)) => match e {
                AppError::Redirect(path) => Ok(view! { <Redirect path show_link=true/> }.into_any()),
                _ => Err(AppError::NotFound),
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
