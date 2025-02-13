// use leptos::logging;
use leptos::prelude::*;
// use leptos_meta::*;
use std::str::FromStr;

use crate::error::AppError;

use crate::view::cellml_codegen::CellMLCodegen;
use crate::view::cellml_math::CellMLMath;
use crate::view::view::View;

#[derive(Debug, PartialEq)]
pub enum EFView {
    CellMLCodegen,
    CellMLMath,
    View,
}

impl FromStr for EFView {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cellml_codegen" => Ok(EFView::CellMLCodegen),
            "cellml_math" => Ok(EFView::CellMLMath),
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
        EFView::CellMLCodegen => view! { <CellMLCodegen/> }.into_any(),
        EFView::CellMLMath => view! { <CellMLMath/> }.into_any(),
        EFView::View => view! { <View/> }.into_any(),
    }
}

mod cellml_codegen;
mod cellml_math;
mod view;
