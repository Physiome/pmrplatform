use std::fmt::{
    Display,
    Formatter,
    Result,
};
use crate::exposure::*;

impl Display for Exposure {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {} - {} - {}",
            self.id,
            self.workspace_id,
            self.workspace_tag_id
                .map(|x| x.to_string())
                .unwrap_or("<unset>".to_string()),
            &self.commit_id,
        )
    }
}
