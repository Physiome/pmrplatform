// use leptos::logging;
use leptos::prelude::*;
// use leptos_meta::*;
use std::str::FromStr;

use crate::error::AppError;

use argon_sds_archive::ArgonSdsArchive;
use crate::view::cellml_codegen::CellMLCodegen;
use crate::view::cellml_math::CellMLMath;
use crate::view::cellml_metadata::CellMLMetadata;
use crate::view::license_citation::LicenseCitation;
use crate::view::view::View;

#[derive(Debug, PartialEq)]
pub enum EFView {
    ArgonSdsArchive,
    CellMLCodegen,
    CellMLMath,
    CellMLMetadata,
    LicenseCitation,
    View,
}

impl FromStr for EFView {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "argon_sds_archive" => Ok(EFView::ArgonSdsArchive),
            "cellml_codegen" => Ok(EFView::CellMLCodegen),
            "cellml_math" => Ok(EFView::CellMLMath),
            "cellml_metadata" => Ok(EFView::CellMLMetadata),
            "license_citation" => Ok(EFView::LicenseCitation),
            "view" => Ok(EFView::View),
            _ => Err(AppError::ViewNotImplemented),
        }
    }
}

impl std::fmt::Display for EFView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[component]
pub fn ExposureFileView(view_key: EFView) -> impl IntoView {
    match view_key {
        EFView::ArgonSdsArchive => view! { <ArgonSdsArchive/> }.into_any(),
        EFView::CellMLCodegen => view! { <CellMLCodegen/> }.into_any(),
        EFView::CellMLMath => view! { <CellMLMath/> }.into_any(),
        EFView::CellMLMetadata => view! { <CellMLMetadata/> }.into_any(),
        EFView::LicenseCitation => view! { <LicenseCitation/> }.into_any(),
        EFView::View => view! { <View/> }.into_any(),
    }
}

mod argon_sds_archive;
mod cellml_codegen;
mod cellml_math;
mod cellml_metadata;
mod license_citation;
mod view;
