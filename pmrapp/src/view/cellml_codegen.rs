use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::ViewPath;
use crate::exposure::api::read_blob;
use crate::component::CodeBlock;

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
                    }.to_string();
                    match code.await {
                        Some(Ok(code)) => {
                            let code = String::from_utf8(code)
                                .map_err(|_| AppError::InternalServerError)?;
                            Ok(view! {
                                <div><a href="../cellml_codegen">Back</a></div>
                                <CodeBlock code lang/>
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
