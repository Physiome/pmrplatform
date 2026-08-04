use std::fmt;
use http::Uri;

#[derive(Clone)]
pub struct Source(pub Uri);

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
