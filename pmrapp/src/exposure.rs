use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

/*
#[component(transparent)]
pub fn ExposureRoutes() -> impl IntoView {
    view! {
    }
}
*/

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
pub fn ExposureListing() -> impl IntoView {
    view! {
        <div class="main">
            <h1>"Listing of exposures"</h1>
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

