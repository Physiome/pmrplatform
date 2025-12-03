use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use crate::error::AppError;

/// Used to disambiguate a query for an alias or the real identifier
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Id {
    Aliased(String),
    Number(String),
}

#[cfg(feature = "ssr")]
impl Id {
    pub async fn resolve(
        self,
        platform: &pmrctrl::platform::Platform,
        kind: &'static str,
    ) -> Result<i64, AppError> {
        Ok(match self {
            Id::Number(s) => s.parse().map_err(|_| AppError::NotFound)?,
            Id::Aliased(s) => platform
                .mc_platform
                .resolve_alias(kind, &s)
                .await
                .map_err(|e| {
                    dbg!(e);
                    AppError::InternalServerError
                })?
                .ok_or(AppError::NotFound)?,
        })
    }
}
