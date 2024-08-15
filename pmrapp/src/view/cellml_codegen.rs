use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::ViewPath;
use crate::exposure::api::read_blob;

#[component]
pub fn CellMLCodegen() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let view_path = expect_context::<ViewPath>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let code = Resource::new_blocking(
        move || view_path.clone().0,
        move |k| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                let k = k.map(|k| {
                    match k.as_str() {
                        "c" => Some("code.C.c"),
                        "c_ida" => Some("code.C_IDA.c"),
                        "fortran" => Some("code.F77.f77"),
                        "matlab" => Some("code.MATLAB.m"),
                        "python" => Some("code.Python.py"),
                        _ => None
                    }
                }).flatten();
                if let Some(k) = k {
                    Some(read_blob(ef.exposure_id, workspace_file_path, efv.id, k.to_string()).await)
                } else {
                    None
                }
            }
        }
    );

    let view_path = expect_context::<ViewPath>().0;
    let code_view = move || {
        let view_path = view_path.clone();
        Suspend::new(async move {
            // FIXME the links only work if route is not with a trailing slash,
            // i.e. cellml_codegen/
            match view_path {
                None => Ok(view! {
                    <ul>
                        <li><a href="cellml_codegen/c">"C"</a></li>
                        <li><a href="cellml_codegen/c_ida">"C (Implicit Differential Algebraic equation system solver)"</a></li>
                        <li><a href="cellml_codegen/fortran">"Fortran 77"</a></li>
                        <li><a href="cellml_codegen/matlab">"MATLAB"</a></li>
                        <li><a href="cellml_codegen/python">"Python"</a></li>
                    </ul>
                }.into_any()),
                Some(view_path) => {
                    let lang = match view_path.as_str() {
                        "c_ida" => "c",
                        "fortran" => "fortran",
                        v => v,
                    };
                    match code.await {
                        Some(Ok(code)) => {
                            let code = String::from_utf8(code.into_vec())
                                .map_err(|_| AppError::InternalServerError)?;
                            Ok(view! {
                                <div><a href="../cellml_codegen">Back</a></div>
                                <pre><code class=format!("language-{lang}")>{code}</code></pre>
                                <link rel="stylesheet" href="/highlight-github.min.css"/>
                                <script async id="hljs" src="/highlight-bundle.min.js"></script>
                                <script>"
                                var events = [];
                                if (!window.hljs.highlightAll) {
                                    events.push(new Promise(function(r) {
                                        document.querySelector('#hljs').addEventListener('load', r, false);
                                    }));
                                }
                                if (!window._hydrated) {
                                    events.push(new Promise(function(r) {
                                        document.addEventListener('_hydrated', r, false);
                                    }));
                                }
                                Promise.all(events).then(function() {
                                    hljs.highlightAll();
                                });
                                if (events.length) {
                                    console.log(`waiting on ${events.length} events`);
                                }
                                "</script>
                            }.into_any())
                        },
                        _ => Err(AppError::NotFound)
                    }
                },
            }
        })
    };

    let view_path = expect_context::<ViewPath>().0;
    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
        </h1>
        <h3>"This is "{efv.view_key}", at path "{view_path}</h3>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {code_view}
            </ErrorBoundary>
        </Transition>
    }
}