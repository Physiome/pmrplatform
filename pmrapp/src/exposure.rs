use crate::error_template::{AppError, ErrorTemplate};
use leptos::logging;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{
        ParentRoute,
        Redirect,
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
    StaticSegment,
    WildcardSegment,
};

mod api;

use crate::exposure::api::{
    list,
    list_files,
    get_file,
};

#[component]
pub fn ExposureRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
    view! {
        <ParentRoute path=StaticSegment("/exposure") view=ExposureRoot>
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
        <div class="main">
            <h1>"Listing of exposures"</h1>
            <div>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors.into()/> }
                }>
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
          // <li>{file} - {flag}</li>
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
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors.into()/> }
                }>
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

    let file = Resource::new(
        move || (
            root_params.get().map(|p| p.id),
            params.get().map(|p| p.path),
        ),
        |p| async move {
            // FIXME this needs to resolve the path down - we need to manually
            // disambiguate the view suffix here as it isn't possible to do
            // suffix after wildcard paths.
            match p {
                (Ok(Some(id)), Ok(Some(path))) => get_file(id, path.clone())
                    .await
                    .map_err(|_| AppError::NotFound),
                // can't acquire the required parameters
                _ => Err(AppError::InternalServerError),
            }
        }
    );
    let ep_view = move || { file.get().map(
        move |file| match file {
            // TODO figure out how to redirect to the workspace.
            Ok(value) => Ok(match value {
                // Again, need to figure out the disambiguation handling
                Ok(file) => (),
                // view! {
                //     <h1>
                //         "Exposure "{file.id}
                //         " - ExposureFile "{file.workspace_file_path}
                //     </h1>
                // }.into_any(),
                Err(path) => (),
                // view! {
                //     <h1>"path "{path}</h1>
                //     // <Redirect
                //     //     path=path
                //     // />
                // }.into_any()
            }),
            _ => Err(AppError::NotFound),
        })
    };

    view! {
        <div class="main">
            <h1>ExposureFile</h1>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors.into()/> }
                }>
                    <div>{ep_view}</div>
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
