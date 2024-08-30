use leptos::prelude::*;
use leptos_meta::{
    MetaTags,
    Stylesheet,
    Title,
    provide_meta_context,
};
use leptos_router::{
    components::{
        A,
        Route,
        Router,
        Routes,
    },
    StaticSegment,
};

use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::ExposureRoutes;
use crate::workspace::WorkspaceRoutes;

pub mod portlet;
use self::portlet::{
    navigation::{
        Navigation,
        NavigationCtx,
    },
    views_available::{
        ViewsAvailable,
        ViewsAvailableCtx,
    },
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // signals + contexts for portlets.
    let (navigation_ctx, set_navigation_ctx) = arc_signal(None::<Resource<NavigationCtx>>);
    provide_context(navigation_ctx);
    provide_context(set_navigation_ctx);
    let (views_available_ctx, set_views_available_ctx) = arc_signal(None::<Resource<ViewsAvailableCtx>>);
    provide_context(views_available_ctx);
    provide_context(set_views_available_ctx);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/pmrapp.css"/>

        // sets the document title
        <Title text="Physiome Model Repository"/>

        // content for this welcome page
        <Router>
            <nav>
                <A href="/">"Home"</A>
                <A href="/workspace/">"Workspace"</A>
                <A href="/exposure/">"Exposure"</A>
            </nav>
            <main>
                <article>
                    <Routes fallback=|| {
                        let mut errors = Errors::default();
                        errors.insert_with_default_key(AppError::NotFound);
                        view! {
                            <ErrorTemplate errors/>
                        }
                        .into_view()
                    }>
                        <Route path=StaticSegment("") view=HomePage/>
                        <WorkspaceRoutes/>
                        <ExposureRoutes/>
                    </Routes>
                </article>
                <aside>
                    <ViewsAvailable/>
                    <Navigation/>
                </aside>
                <footer>
                    <small>"Copyright 2024 IUPS Physiome Project"</small>
                </footer>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    expect_context::<ArcWriteSignal<Option<Resource<NavigationCtx>>>>().set(None);
    expect_context::<ArcWriteSignal<Option<Resource<ViewsAvailableCtx>>>>().set(None);
    view! {
        <Title text="Home — Physiome Model Repository"/>
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
