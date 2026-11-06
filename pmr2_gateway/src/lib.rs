#[cfg(feature = "clap")]
mod cli;
mod gateway;
mod git;
mod path;
mod proxy;
mod response;
mod rewrite;
mod strategy;
mod template;

#[cfg(feature = "clap")]
pub use cli::Cli;
pub use gateway::{Gateway, GatewayBuilder};
pub use git::GitGateway;
pub use proxy::Proxy;
pub use rewrite::{
    RewriteHref, RewriteHrefLayer, RewritePmr2, RewritePmr2Layer, RewriteTemplate, RewriteTemplateLayer,
};
pub use strategy::Strategy;
