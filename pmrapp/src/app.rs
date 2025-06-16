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
use leptos_sync_ssr::component::SyncSsrSignal;

use crate::ac::{
    ACRoutes,
    SessionStatus,
    provide_session_context,
};
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::ExposureRoutes;
use crate::workspace::WorkspaceRoutes;

pub mod portlet;
use self::portlet::{
    ContentAction,
    ExposureSource,
    Navigation,
    ViewsAvailable,
    provide_portlet_context,
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

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/pmrapp.css"/>

        // sets the document title
        <Title text="Physiome Model Repository"/>

        // content for this welcome page
        <Router>
            <SyncSsrSignal setup=|| {
                provide_portlet_context();
                provide_session_context();
            }>
                <header>
                    <nav>
                        <A href="/">"Home"</A>
                        <A href="/workspace/">"Workspace"</A>
                        <A href="/exposure/">"Exposure"</A>
                        <div class="flex-grow"></div>
                        <SessionStatus/>
                    </nav>
                </header>
                <main>
                    <section>
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
                                <ACRoutes/>
                            </Routes>
                        </article>
                        <aside>
                            <ContentAction/>
                        </aside>
                    </section>
                    <aside>
                        <ExposureSource/>
                        <ViewsAvailable/>
                        <Navigation/>
                    </aside>
                </main>
            </SyncSsrSignal>
            <footer>
                <small>"Copyright 2024 IUPS Physiome Project"</small>
            </footer>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
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
