use leptos::{IntoView, component, logging, view};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Redirect(
    path: String,
    #[prop(optional)] show_link: bool,
) -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    {
        logging::log!("Redirecting CSR to {path}...");
        let window = leptos::prelude::tachys::dom::window();
        if let Err(_) = window.location().replace(&path) {
            logging::error!("fail to replace location with {path}");
        };
    }
    let res_path = Resource::new_blocking(move || path.clone(), |path| async move {
        #[cfg(feature = "ssr")]
        {
            logging::log!("Redirecting SSR to {path}...");
            let res = expect_context::<leptos_axum::ResponseOptions>();
            res.set_status(axum::http::StatusCode::FOUND);
            res.insert_header(
                axum::http::header::LOCATION,
                axum::http::header::HeaderValue::from_str(&path)
                    .expect("Failed to create HeaderValue"),
            );
        }
        path
    });
    view! {
        <Suspense fallback=|| view! {}>
            {move || Suspend::new(async move {
                let path = res_path.await;
                show_link.then(|| view! {
                    "Redirecting to "<a href=path.clone()>{path.clone()}</a>
                })
            })}
        </Suspense>
    }
}

#[component]
pub fn RedirectTS() -> impl IntoView {
    Signal::derive(move || use_location().pathname.get())
        .with(|url| {(url.chars().last() != Some('/')).then(|| view! {
            <Redirect path=format!("{url}/")/>
        })})
}
