use axum::{
    body::HttpBody,
    http::uri::{Scheme, Uri},
};
use camino::Utf8PathBuf;
use clap::Parser;
use crate::gateway::{Gateway, GatewayBuilder};
use std::{
    net::SocketAddr,
    str::FromStr,
};

#[derive(Debug, Parser)]
pub struct Cli {
    /// Have this gateway connect to the specified Zope instance.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_ZOPE_ADDR",
        env = "PMR2_GATEWAY_ZOPE_ADDR",
        default_value = "127.0.0.0:8080",
    )]
    pub zope_addr: SocketAddr,

    /// The location to the Plone site.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_PLONE_SITE",
        env = "PMR2_GATEWAY_PLONE_SITE",
        default_value = "/plone",
    )]
    pub plone_site: String,

    /// The uri to serve the main site under.
    ///
    /// If none is provided, this defaults to `http://{bind_addr}`.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_SITE_URI",
        env = "PMR2_GATEWAY_SITE_URI",
    )]
    pub site_uri: Option<Uri>,

    /// Directory to cache the PMR2 data.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_CACHE_DIR",
        env = "PMR2_GATEWAY_CACHE_DIR",
    )]
    pub cache_dir: Utf8PathBuf,

    /// Specify the maximum concurrent requests permitted outbound to the Zope instance for requests that
    /// will not be cached.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_PASSTHRU_PERMITS",
        env = "PMR2_GATEWAY_PASSTHRU_PERMITS",
        default_value_t = 3,
    )]
    pub passthru_permits: usize,

    /// Specify the maximum concurrent requests permitted outbound to the Zope instance for requests to
    /// resources that have yet to be cached.  Requests to resources that have been cached will not have
    /// this concurrency restriction applied.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_CACHE_PERMITS",
        env = "PMR2_GATEWAY_CACHE_PERMITS",
        default_value_t = 3,
    )]
    pub cache_permits: usize,

    /// The address to bind this service under.
    #[clap(
        long,
        value_name = "PMR2_GATEWAY_BIND_ADDR",
        env = "PMR2_GATEWAY_BIND_ADDR",
    )]
    pub bind_addr: SocketAddr,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            zope_addr: SocketAddr::from_str("127.0.0.1:8080")
                .expect("provided static value is valid"),
            plone_site: "/plone".into(),
            site_uri: None,
            cache_dir: ".".into(),
            passthru_permits: 3,
            cache_permits: 3,
            bind_addr: SocketAddr::from_str("127.0.0.1:8000")
                .expect("provided static value is valid"),
        }
    }
}

impl Cli {
    /// Construct a gateway based on the specified options.
    ///
    /// This function panics.
    pub fn to_gateway<B>(&self) -> Gateway<B>
    where
        B: HttpBody + Send + std::fmt::Debug,
        <B as HttpBody>::Data: Send,
    {
        let host_base_uri = self.host_base_uri();
        #[cfg(feature = "trace")]
        tracing::info!("gateway build for {host_base_uri}");
        GatewayBuilder::new()
            .cache_dir(&self.cache_dir)
            .cache_permits(self.cache_permits)
            .passthru_permits(self.passthru_permits)
            .build_for(&host_base_uri)
            .expect(&format!(
                "Ensure either the cache dirs under {} may be created and that the generated host uri {} \
                is valid",
                self.cache_dir,
                host_base_uri,
            ))
    }

    /// Construct the expected base uri for the underlying PMR2 instance.  This uri will include the
    /// virtual host configuration.
    ///
    /// This function panics if the provided site_uri does not have a valid scheme or a valid authority,
    /// and if the final constructed `Uri` is invalid.
    pub fn host_base_uri(&self) -> Uri {
        let plone_site = self.plone_site
            .trim_start_matches('/')
            .trim_end_matches('/');
        let site_uri = self.site_uri
            .clone()
            .unwrap_or_else(|| {
                if self.bind_addr.ip().is_unspecified() {
                    panic!("`bind_addr`'s ip is the special unspecified address; please specify `site_uri`")
                }
                if self.bind_addr.port() == 0 {
                    panic!("`bind_addr`'s port is unspecified; please specify `site_uri`")
                }
                let uri = format!("http://{}", self.bind_addr);
                Uri::from_maybe_shared(uri)
                    .expect("bind_addr should form a valid uri")
                    .to_owned()
            });
        let scheme = site_uri
            .scheme()
            .expect("site_uri did not provide a scheme");
        let host = site_uri
            .host()
            .expect("site_uri did not provide a host");
        let port = site_uri
            .port_u16()
            .unwrap_or_else(|| match scheme {
                scheme if *scheme == Scheme::HTTP => 80,
                scheme if *scheme == Scheme::HTTPS => 443,
                scheme => panic!("site_uri provide an unsupported scheme: {scheme}"),
            });
        let uri = format!(
            "http://{}/VirtualHostBase/{scheme}/{host}:{port}/{plone_site}/VirtualHostRoot/",
            self.zope_addr,
        );
        Uri::from_maybe_shared(uri.to_owned())
            .expect(&format!("the values contained here constructed an invalid uri: {uri}"))
    }
}

#[cfg(test)]
mod test {
    use axum::body::Body;
    use super::*;

    #[test]
    fn host_base_uri() {
        let mut cli = Cli::default();
        let expected = Uri::from_static(
            "http://127.0.0.1:8080/VirtualHostBase/http/127.0.0.1:8000/plone/VirtualHostRoot/"
        );
        assert_eq!(cli.host_base_uri(), expected);

        // Check that this unspecified address/port won't cause issue for the specified site_uri cases.
        cli.bind_addr = SocketAddr::from_str("0.0.0.0:0")
            .expect("provided static value is valid");
        cli.site_uri = Some(Uri::from_static("http://models.cellml.org/"));
        cli.plone_site = "/pmr/";
        let expected = Uri::from_static(
            "http://127.0.0.1:8080/VirtualHostBase/http/models.cellml.org:80/pmr/VirtualHostRoot/"
        );
        assert_eq!(cli.host_base_uri(), expected);

        cli.site_uri = Some(Uri::from_static("https://models.physiomeproject.org/"));
        cli.plone_site = "pmr";
        let expected = Uri::from_static(
            "http://127.0.0.1:8080/VirtualHostBase/https/models.physiomeproject.org:443/pmr/VirtualHostRoot/"
        );
        assert_eq!(cli.host_base_uri(), expected);
    }

    #[test]
    #[should_panic]
    fn host_base_uri_unspecified_bind_addr_ip() {
        let mut cli = Cli::default();
        cli.bind_addr = SocketAddr::from_str("0.0.0.0:80")
            .expect("provided static value is valid");
        cli.host_base_uri();
    }

    #[test]
    #[should_panic]
    fn host_base_uri_unspecified_bind_addr_port() {
        let mut cli = Cli::default();
        cli.bind_addr = SocketAddr::from_str("127.0.0.1:0")
            .expect("provided static value is valid");
        cli.host_base_uri();
    }

    #[test]
    fn to_gateway() {
        let mut cli = Cli::default();
        cli.cache_dir = tempfile::tempdir()
            .expect("tempdir must be created here")
            .path()
            .to_owned()
            .try_into()
            .expect("created tempdir isn't utf8");
        cli.to_gateway::<Body>();
    }
}
