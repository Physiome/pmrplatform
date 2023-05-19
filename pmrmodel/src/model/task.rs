use futures::future;
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use pmrmodel_base::task::{
    Task,
    TaskArg,
};
use pmrmodel_base::task_template::{
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum BuildArgError {
    NotImplemented,
}

impl Display for BuildArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self {
            BuildArgError::NotImplemented => "not implemented",
        })
    }
}

fn build_arg(
    value: &str,
    arg: TaskTemplateArg,
) -> Result<Vec<String>, BuildArgError> {
    Err(BuildArgError::NotImplemented)
}


#[test]
fn test_build_arg() {
    assert_eq!(
        build_arg("foo", TaskTemplateArg { .. Default::default() }),
        Err(BuildArgError::NotImplemented),
    );
}
