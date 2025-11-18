use leptos::prelude::*;
use leptos_router::{
    components::{ParentRoute, Route},
    hooks::use_params,
    nested_router::Outlet,
    params::Params,
    ParamSegment, SsrMode, StaticSegment,
};

pub mod api;

use crate::{
    error::AppError,
    error_template::ErrorTemplate,
};
use api::{list_citations, list_citation_resources};

#[component]
pub fn ListingRoutes() -> impl MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/listing") view=ListingRoot ssr>
            <Route path=StaticSegment("/") view=ListingIndex/>
            <ParentRoute path=StaticSegment("by-reference") view=Outlet>
                <Route path=StaticSegment("/") view=ListingByReference />
                <Route path=ParamSegment("id") view=ReferenceDetails/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn ListingRoot() -> impl IntoView {
    view! {
        <Outlet/>
    }
}

#[component]
pub fn ListingIndex() -> impl IntoView {
    view! {
        <ul>
            <li><a href="/listing/by-reference/">"By Reference"</a></li>
        </ul>
    }
}

// TODO components for references should be in own module

#[component]
pub fn ListingByReference() -> impl IntoView {
    let citation_listing = Resource::new_blocking(
        move || (),
        move |_| {
            async move {
                list_citations().await
            }
        },
    );

    let citation_listing_view = move || Suspend::new(async move {
        citation_listing.await.map(|citation| citation
            .into_iter()
            .map(move |citation| view! {
                <li><a href=format!("/listing/by-reference/{}", citation.identifier)>
                    {citation.identifier.clone()}
                </a></li>
            })
            .collect_view()
        )
    });

    view! {
        <h1>"Citation Referenced by data in this repository:"</h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                <ul>
                    {citation_listing_view}
                </ul>
            </ErrorBoundary>
        </Transition>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct CitationParams {
    id: Option<String>,
}

#[component]
pub fn ReferenceDetails() -> impl IntoView {
    let params = use_params::<CitationParams>();

    let resources = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        move |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => {
                    Ok((id.clone(), list_citation_resources(id).await?))
                }
            }
        }
    );

    let view = move || Suspend::new(async move {
        resources.await.map(|(id, resources)| {
            let items = resources
                .into_iter()
                .map(move |resource| view! {
                    <li><a href=format!("{resource}/")>{resource.clone()}</a></li>
                })
                .collect_view();
            view! {
                <h1>"Listing of resources that cites reference "{id}</h1>
                <ul>
                    {items}
                </ul>
            }
        })
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}
