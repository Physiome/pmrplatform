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

mod api;

use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::component::{Redirect, RedirectTS};
use crate::exposure::api::{
    list,
    list_files,
    resolve_exposure_path,
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
}

#[component]
pub fn ExposureRoot() -> impl IntoView {
    // provide_meta_context();
    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Outlet/>
    }
}

#[component]
pub fn ExposureListing() -> impl IntoView {
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
    let listing = move || { exposures
        .get()
        .map(move |exposures| match exposures {
            Err(e) => view! {
                <pre class="error">"Server Error: " {e.to_string()}</pre>
            }
                .into_any(),
            Ok(exposures) => exposures
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
                .into_any()
        })
    };

    view! {
        <RedirectTS />
        <div class="main">
            <h1>"Listing of exposures"</h1>
            <div>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    <div>{listing}</div>
                </ErrorBoundary>
            </Suspense>
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
    provide_context(params);
    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Outlet/>
    }

}

#[component]
pub fn ExposureMain() -> impl IntoView {
    use std::iter::{repeat, zip};

    let params = expect_context::<Memo<Result<ExposureParams, ParamsError>>>();
    let id = move || params.get().map(|p| p.id);
    let files = Resource::new(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => list_files(id)
                    .await
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::NotFound),
            }
        }
    );
    let file_entry_view = move |(id, (file, flag)): (i64, (String, bool))| view! {
        <li>
            <a href=format!("/exposure/{id}/{file}")>
                {file.clone()}
            </a>
            " - "{flag}
        </li>
    };
    let listing = move || { files.get().map(
        move |files| match files {
            Err(_) => Err(AppError::NotFound),
            Ok(files) => {
                Ok(view! {
                    <RedirectTS/>
                    <h1>"Viewing exposure "{id}</h1>
                    <ul>{
                        zip(
                            repeat(id().unwrap().unwrap()),
                            files.into_iter(),
                        )
                            .map(file_entry_view)
                            .collect_view()
                    }</ul>
                }.into_view())
            }
        })
    };

    view! {
        <div class="main">
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    <div>{listing}</div>
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureFileParams {
    path: Option<String>,
}

#[component]
pub fn ExposureFile() -> impl IntoView {
    let root_params = expect_context::<Memo<Result<ExposureParams, ParamsError>>>();
    let params = use_params::<ExposureFileParams>();

    let file = Resource::new_blocking(
        move || (
            root_params.get().map(|p| p.id),
            params.get().map(|p| p.path),
        ),
        |p| async move {
            match p {
                (Ok(Some(id)), Ok(Some(path))) => resolve_exposure_path(id, path.clone())
                    .await
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let ep_view = Suspend::new(async move {
        match file.await {
            // TODO figure out how to redirect to the workspace.
            Ok(Ok((ef, efv))) => Ok(view! {
                <h1>
                    "Exposure "{ef.id}
                    " - ExposureFile "{ef.workspace_file_path}
                    " - ExposureFileView "{efv.view_key}
                </h1>
            }.into_any()),
            Ok(Err(e)) => match e {
                AppError::Redirect(path) => Ok(view! { <Redirect path/> }.into_any()),
                _ => Err(AppError::NotFound),
            }
            _ => Err(AppError::NotFound),
        }
    });

    view! {
        <div class="main">
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    <h1>ExposureFile</h1>
                    <div>{ep_view}</div>
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
