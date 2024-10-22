use crate::error::AppError;
use http::status::StatusCode;
use leptos::{logging::log, prelude::*};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying them.
#[component]
pub fn ErrorTemplate(
    #[prop(into)] errors: MaybeSignal<Errors>,
) -> impl IntoView {
    // Get Errors from Signal
    // Downcast lets us take a type that implements `std::error::Error`
    let errors = Memo::new(move |_| {
        errors
            .get_untracked()
            .into_iter()
            .filter_map(|(_, v)| v.downcast_ref::<AppError>().cloned())
            .collect::<Vec<_>>()
    });
    log!("Errors: {:#?}", &*errors.read_untracked());

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors.read_untracked()[0].status_code());
        }
    }

    view! {
        {move || {
            errors.get()
                .into_iter()
                .map(|error| {
                    let error_code = error.status_code();
                    let error_string = (error_code == StatusCode::INTERNAL_SERVER_ERROR)
                        .then(|| format!("Error: {error}"));
                    view! {
                        <h1>{error_code.to_string()}</h1>
                        <p>{error_string}</p>
                    }
                })
                .collect_view()
        }}
    }
}
