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
use api::{
    list_citations,
    list_indexes,
    list_index_terms,
    list_indexed_resources_by_kind_term,
};

#[component]
pub fn ListingRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/listing") view=ListingRoot ssr>
            <Route path=StaticSegment("/") view=Listing/>
            <ParentRoute path=StaticSegment("by-reference") view=Outlet>
                <Route path=StaticSegment("/") view=ListingByReference />
                <Route path=ParamSegment("id") view=ReferenceDetails/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn CatalogRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/catalog") view=CatalogRoot ssr>
            <Route path=StaticSegment("/") view=IndexListing/>
            <ParentRoute path=ParamSegment("kind") view=Outlet>
                <Route path=StaticSegment("/") view=KindListing />
                <Route path=ParamSegment("term") view=TermListing/>
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
pub fn Listing() -> impl IntoView {
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
                <li><a href=format!("/listing/by-reference/{}", citation.id)>
                    {citation.title}" - "{citation.id.clone()}
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

    let resource_set = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        move |id| async move {
            match id {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(id)) => {
                    Ok(list_indexed_resources_by_kind_term(
                        String::from("citation_id"),
                        id,
                    ).await?)
                }
            }
        }
    );

    let view = move || Suspend::new(async move {
        Ok::<_, AppError>(resource_set.await?.map(|resource_set| {
            let items = resource_set.resource_paths
                .into_iter()
                .filter_map(move |resource| {
                    let uri = resource.data.get("aliased_uri");
                    let title = resource.data.get("description");
                    if let Some(uri) = uri && let Some(title) = title {
                        if let Some(uri) = uri.get(0) && let Some(title) = title.get(0) {
                            return Some(view! {
                                <li><a href=format!("{uri}/")>{title.clone()}</a></li>
                            });
                        }
                    }
                    None
                })
                .collect_view();
            view! {
                <h1>"Listing of resources that cites reference "{resource_set.term}</h1>
                <ul>
                    {items}
                </ul>
            }
        }))
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn CatalogRoot() -> impl IntoView {
    view! {
        <Outlet/>
    }
}

#[component]
pub fn IndexListing() -> impl IntoView {
    let index_listing = Resource::new_blocking(
        move || (),
        move |_| {
            async move {
                list_indexes().await
            }
        },
    );

    let index_listing_view = move || Suspend::new(async move {
        index_listing.await.map(|kind| kind
            .into_iter()
            .map(move |kind| view! {
                <li><a href=format!("/catalog/{}/", kind)>
                    {kind.clone()}
                </a></li>
            })
            .collect_view()
        )
    });

    view! {
        <h1>"Listing of indexes"</h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                <ul>
                    {index_listing_view}
                </ul>
            </ErrorBoundary>
        </Transition>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct KindParams {
    kind: Option<String>,
}

#[component]
pub fn KindListing() -> impl IntoView {
    let params = use_params::<KindParams>();

    let index_terms = Resource::new_blocking(
        move || params.get().map(|p| p.kind),
        move |kind| async move {
            match kind {
                Err(_) => Err(AppError::InternalServerError),
                Ok(None) => Err(AppError::NotFound),
                Ok(Some(kind)) => {
                    Ok(list_index_terms(kind).await?)
                }
            }
        }
    );

    let view = move || Suspend::new(async move {
        index_terms.await.map(|index_terms| {
            match index_terms {
                Some(index_terms) => {
                    let view = index_terms.terms
                        .into_iter()
                        .map(|term| view! {
                            <li><a href=format!("/catalog/{}/{}/", &index_terms.kind.description, term)>
                                {term.clone()}
                            </a></li>
                        })
                        .collect_view();
                    view! {
                        <h1>"Listing of terms in the "{index_terms.kind.description}" index"</h1>
                        <ul>{view}</ul>
                    }
                    .into_any()
                }
                None => {
                    // TODO this should be a Not Found
                    view! {
                        <h1>"No such kind"</h1>
                    }
                    .into_any()
                }
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

#[derive(Params, PartialEq, Clone, Debug)]
pub struct TermParams {
    kind: Option<String>,
    term: Option<String>,
}

#[component]
pub fn TermListing() -> impl IntoView {
    let params = use_params::<TermParams>();

    let resource_set = Resource::new_blocking(
        move || params.get().map(|p| (p.kind, p.term)),
        move |args| async move {
            match args {
                Err(_) => Err(AppError::InternalServerError),
                Ok((Some(kind), Some(term))) => {
                    Ok(list_indexed_resources_by_kind_term(kind, term).await?)
                }
                _ => Err(AppError::NotFound),
            }
        }
    );

    let view = move || Suspend::new(async move {
        resource_set.await.map(|resource_set| {
            match resource_set {
                Some(resource_set) => {
                    let view = resource_set.resource_paths
                        .into_iter()
                        .map(move |info| {
                            let href = match info.data.get("aliased_uri") {
                                Some(items) => items.get(0)
                                    .map(String::as_str)
                                    .unwrap_or(info.resource_path.as_str()),
                                None => info.resource_path.as_str(),
                            }
                            .to_string();
                            view! {
                                <li><a href=href>
                                    {href.clone()}
                                </a></li>
                            }
                        })
                        .collect_view();
                    view! {
                        <h1>
                            "Listing of resources found in the "{resource_set.kind.description}
                            " index with the term "{resource_set.term}"."
                        </h1>
                        <ul>{view}</ul>
                    }
                    .into_any()
                }
                None => {
                    // TODO this should be a Not Found
                    view! {
                        <h1>"No such kind or term"</h1>
                    }
                    .into_any()
                }
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
