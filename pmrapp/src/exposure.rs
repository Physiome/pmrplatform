use crate::error_template::{AppError, ErrorTemplate};
use serde::{Serialize, Deserialize};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;

use crate::exposure::api::{
    list_exposures,
};

#[component]
pub fn Exposure() -> impl IntoView {
    // provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        // <Stylesheet id="leptos" href="/pkg/workspace.css"/>

        // sets the document title
        <Title text="Physiome Model Repository > Exposure"/>
        <Outlet/>

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
    }
}

#[component]
pub fn ExposureListing() -> impl IntoView {
    let exposures = create_resource(
        move || (),
        move |_| async move {
            let result = list_exposures().await;
            match result {
                Ok(ref result) => logging::log!("{}", result.len()),
                Err(_) => logging::log!("error loading exposures"),
            };
            result
        },
    );

    view! {
        <div class="main">
            <h1>"Listing of exposures"</h1>
            <div>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors/> }
                }>
                    {move || {
                        logging::log!("rendering listing");
                        let listing = { move || { exposures
                            .get()
                            .map(move |exposures| match exposures {
                                Err(e) => {
                                    view! {
                                        <pre class="error">"Server Error: " {e.to_string()}</pre>
                                    }
                                        .into_view()
                                }
                                Ok(exposures) => {
                                    exposures
                                        .into_iter()
                                        .map(move |exposure| {
                                            view! {
                                                <div>
                                                    <div><a href=format!("/exposure/{}/", exposure.id)>
                                                        "Exposure "{exposure.id}
                                                    </a></div>
                                                    <div>{exposure.description}</div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }
                            })
                            .unwrap_or_default()
                        }};
                        view! { <div>{listing}</div> }
                    }}
                </ErrorBoundary>
            </Suspense>
            </div>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureParams {
    id: Option<String>,
}

#[component]
pub fn ExposureView() -> impl IntoView {
    let params = use_params::<ExposureParams>();
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
            <h1>"Viewing exposure "{id}</h1>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureViewParams {
    id: Option<String>,
    // path: Vec<String>,
}

#[component]
pub fn ExposurePathView() -> impl IntoView {
    let params = use_params_map();
    let id = move || {
        params.get().get("id").cloned().unwrap_or_default()
    };
    let path = move || {
        params.get().get("path").cloned().unwrap_or_default()
    };

    view! {
        <div class="main">
            <h1>"Viewing exposure "{id}" at "{path}</h1>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureComponentParams {
    id: Option<i64>,
    path: Option<String>,
}

// custom routing solution for exposures as the built-in version not
// fit for purpose
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExposureRouting {
    Listing,
    Exposure(i64),  // id
    File(i64, String), // id, path
}

#[component]
pub fn ExposureComponent() -> impl IntoView {
    let params = use_params::<ExposureComponentParams>();
    let route = create_resource(
        move || params.get().map(|p| (p.id, p.path)).unwrap_or_else(|_| (None, None)),
        |p| async move {
            match p {
                (None, None) => Ok(ExposureRouting::Listing),
                (Some(id), None) => Ok(ExposureRouting::Exposure(id)),
                (Some(id), Some(path)) => if path == "" {
                    Ok(ExposureRouting::Exposure(id))
                } else {
                    Ok(ExposureRouting::File(id, path))
                },
                _ => Err(AppError::NotFound),
            }
        }
    );

    let exposure_view = move || {
        match route.get() {
            Some(Ok(ExposureRouting::Listing)) => Ok(view! {
                <div>
                    <ExposureListing/>
                </div>
            }),
            // TODO probably need a dedicated function for resolving
            // whether it is in fact Ok (e.g. exposure actually exist
            Some(Ok(ExposureRouting::Exposure(_))) => {
                Ok(view! {
                    <div>
                        <ExposureView/>
                    </div>
                })
            }
            // likewise for the path
            Some(Ok(ExposureRouting::File(..))) => {
                Ok(view! {
                    <div>
                        <ExposurePathView/>
                    </div>
                })
            }
            _ => Err(AppError::InternalServerError),
        }
    };

    view! {
        <Suspense>
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
                {exposure_view}
            </ErrorBoundary>
        </Suspense>
    }

}
