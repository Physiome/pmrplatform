use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::api::read_blob;

#[derive(serde::Deserialize)]
struct Pmr2Cmeta {
    #[serde(rename = "citation_bibliographicCitation")]
    citation_bibliographic_citation: Option<String>,
    // names (last, first, second)
    citation_authors: Option<Vec<(String, String, String)>>,
    citation_title: Option<String>,
    citation_id: Option<String>,
    // citation_issued: Option<String>,
    model_author: Option<String>,
    model_author_org: Option<String>,
    model_title: Option<String>,
    // keyword (location, identifier)
    keywords: Option<Vec<(String, String)>>,
}

#[component]
pub fn CellMLMetadata() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let cmeta = Resource::new_blocking(
        move || (),
        move |_| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                read_blob(ef.exposure_id, workspace_file_path, efv.id, "cmeta.json".to_string()).await
            }
        }
    );

    let cmeta_view = move || {
        Suspend::new(async move {
            match cmeta.await {
                Ok(cmeta) => {
                    let cmeta: Pmr2Cmeta = serde_json::from_slice(&cmeta)
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(view! {
                        <h3>"CellML Model Authorship"</h3>
                        <dl>
                            <dt>"Title:"</dt>
                            <dd>{cmeta.model_title}</dd>
                            <dt>"Author:"</dt>
                            <dd>{cmeta.model_author}</dd>
                            <dt>"Organisation:"</dt>
                            <dd>{cmeta.model_author_org}</dd>
                        </dl>

                        <h3>"Citation"</h3>
                        <dl>
                            <dt>"Authors:"</dt>
                            <dd>
                            {cmeta.citation_authors.map(|value| view! {
                                <ul>{
                                    value.into_iter()
                                        .map(|(last, first, second)| view! {
                                            <li>{last}", "{first}" "{second}</li>
                                        })
                                        .collect_view()
                                }</ul>
                            })}
                            </dd>
                            <dt>"Title:"</dt>
                            <dd>{cmeta.citation_title}</dd>
                            <dt>"Source:"</dt>
                            <dd>{cmeta.citation_bibliographic_citation}</dd>
                            <dt>"Identifier:"</dt>
                            // TODO provide <a> linkage to actual resource
                            <dd>{cmeta.citation_id}</dd>
                            <dt>"Model Keywords:"</dt>
                            <dd>
                            {cmeta.keywords.map(|value| {
                                value.into_iter()
                                    .map(|(_, identifier)|
                                        identifier.trim()
                                            .replace(' ', "_")
                                            .to_lowercase()
                                    )
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })}
                            </dd>
                        </dl>
                    })
                },
                _ => Err(AppError::NotFound)
            }
        })
    };

    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
            " - CellML metadata"
        </h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {cmeta_view}
            </ErrorBoundary>
        </Transition>
    }
}
