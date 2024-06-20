use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::workspace::{
    Workspace,
    WorkspaceListing,
    // WorkspaceRoutes,
    WorkspaceView,
    WorkspaceCommitPathView,
};

use crate::exposure::{
    ExposureListing,
    ExposureView,
    // ExposureRedirect,
    ExposurePathView,
};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let current_url = Signal::derive(move || use_location().pathname.get());

    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/pmrapp.css"/>

        // sets the document title
        <Title text="Physiome Model Repository"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <nav>
                <A href="/" active_class="active">"Home"</A>
                <a href="/workspace/"
                    class:active=move || current_url.get()
                        .starts_with("/workspace/")
                >"Workspace"</a>
                <a href="/exposure/"
                    class:active=move || current_url.get()
                        .starts_with("/exposure/")
                >"Exposure"</a>
            </nav>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>

                    // Workspaces
                    // this doesn't work because transparent components does
                    // not support multiple routes
                    // <WorkspaceRoutes/>
                    <Route path="/workspace/" view=WorkspaceListing trailing_slash=TrailingSlash::Exact/>
                    <Route path="/workspace/:id/" view=WorkspaceView trailing_slash=TrailingSlash::Exact/>
                    <Route path="/workspace/:id/file/:commit/*path" view=WorkspaceCommitPathView trailing_slash=TrailingSlash::Exact/>

                    // Exposures
                    <Route path="/exposure/" view=ExposureListing trailing_slash=TrailingSlash::Exact/>
                    <Route path="/exposure/:id/" view=ExposureView trailing_slash=TrailingSlash::Exact/>
                    <Route path="/exposure/:id/*path" view=ExposurePathView trailing_slash=TrailingSlash::Exact/>
                    // both of the following cannot work with axum, they duplicate
                    // moreover, suffix mapping doesn't work.
                    // <Route path="/exposure/:id/*path" view=ExposureRedirect trailing_slash=TrailingSlash::Exact/>
                    // <Route path="/exposure/:id/*path/view" view=ExposurePathView trailing_slash=TrailingSlash::Exact/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="main">
            <h1>"Physiome Model Repository"</h1>
            <p>
              "Welcome to the demo of the platform that will \n\
              power the next generation of the Physiome Model \n\
              Repository, written in Rust."
            </p>
            <p>
              "The code for this project may be found on "
              <a href="https://github.com/Physiome/pmrplatform/">
                "its project page on GitHub"
              </a>
              "."
            </p>
            <dl>
                <dt><a href="/workspace/">"Workspace Listing"</a></dt>
              <dd>"Listing of all workspaces in the repository."</dd>
            </dl>
        </div>
    }
}
