use serde::{Deserialize, Serialize};
use gix::Object;

pub struct PathObject<'a> {
    pub path: &'a str,
    pub object: Object<'a>,
}
