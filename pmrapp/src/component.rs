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
        let window = leptos::prelude::window();
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
        <Transition fallback=|| view! {}>
            {move || Suspend::new(async move {
                let path = res_path.await;
                show_link.then(|| view! {
                    "Redirecting to "<a href=path.clone()>{path.clone()}</a>
                })
            })}
        </Transition>
    }
}

#[component]
pub fn RedirectTS() -> impl IntoView {
    Signal::derive(move || use_location().pathname.get())
        .with(|url| {(url.chars().last() != Some('/')).then(|| view! {
            <Redirect path=format!("{url}/")/>
        })})
}

#[component]
pub fn CodeBlock(code: String, lang: String) -> impl IntoView {
    let (inner, set_inner) = signal(String::new());
    #[cfg(feature = "ssr")]
    {
        // this is unused.
        drop(lang);
        set_inner.set(html_escape::encode_text(&code).into_owned());
    }
    #[cfg(not(feature = "ssr"))]
    {
        let result = crate::client::wbg::highlight(code, lang);
        Effect::new(move |_| {
            match result.clone() {
                Ok(r) => set_inner.set(r),
                Err(e) => logging::error!("{e:?}"),
            }
        });
    }
    view! {
        <pre><code inner_html=inner></code></pre>
    }
}
