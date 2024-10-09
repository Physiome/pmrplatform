use std::fmt::{
    Display,
    Formatter,
    Result,
};
use crate::task_template::*;

impl Display for TaskTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "\
            TaskTemplate {{ id: {}, version_id: {:?}, ... }}\n\
            {}{}{}\
            ",
            self.id,
            &self.version_id,
            &self.bin_path,
            &(match &self.args {
                Some(args) => format!("{}", args.iter().fold(
                    String::new(), |acc, arg| [acc, arg.to_string()].join(" "))),
                None => "?arguments missing?".to_string(),
            }),
            if self.final_task_template_arg_id.is_some() {
                ""
            }
            else {
                " ?not finalized?"
            },
        )
    }
}

impl Display for TaskTemplateArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match (
            &self.flag, self.flag_joined,
            match (
                &self.prompt,
                &self.default,
                self.choice_fixed,
                &self.choice_source.as_deref(),
            ) {
                (None, None, _, _) => None,
                (None, Some(default), _, _) =>
                    Some(format!(">?{:?}?<", &default)),
                (Some(prompt), None, false, _) =>
                    Some(format!("<{}>", &prompt)),
                (Some(prompt), Some(default), false, _) =>
                    Some(format!("[<{}>;default={:?}]",
                        &prompt, &default)),
                (Some(prompt), None, true, None) =>
                    Some(format!("<{};choices={{...}}>", &prompt)),
                (Some(prompt), Some(default), true, None) =>
                    Some(format!("[<{}>;default={:?};choices={{...}}]",
                        &prompt, &default)),
                (Some(prompt), None, true, Some("")) =>
                    Some(format!("<{};choices={{...}}>", &prompt)),
                (Some(prompt), None, true, Some(source)) =>
                    Some(format!("<{};choices={{source:'{}'}}>",
                        &prompt, &source)),
                (Some(prompt), Some(default), true, Some("")) =>
                    Some(format!("[<{}>;default={:?};choices={{...}}]",
                        &prompt, &default)),
                (Some(prompt), Some(default), true, Some(source)) =>
                    Some(format!("<<{}>;default={:?};choices={{source:'{}'}}>",
                        &prompt, &default, &source)),
            }
        ) {
            (None, _, None) => write!(f, ""),
            (Some(flag), _, None) => write!(f, "{}", flag),
            (None, _, Some(arg)) => write!(f, "{}", arg),
            (Some(flag), false, Some(arg)) => write!(f, "{} {}", flag, arg),
            (Some(flag), true, Some(arg)) => write!(f, "{}{}", flag, arg),
        }
    }
}

impl Display for TaskTemplateArgChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} => {}",
            match self.to_arg.as_deref() {
                Some(s) => format!("{:?}", s),
                None => "<OMITTED>".into(),
            },
            &self.label,)
    }
}
