use pmrcore::{
    ac::session::SessionFactory,
    platform::ACPlatform,
};
use pmrrbac::Builder as PmrRbacBuilder;

#[derive(Default)]
pub struct Builder {
    // platform
    ac_platform: Option<Box<dyn ACPlatform>>,
    // automatically purges all but the most recent passwords
    password_autopurge: bool,
    pmrrbac_builder: PmrRbacBuilder,
    session_factory: SessionFactory,
}

pub struct Platform {
    ac_platform: Box<dyn ACPlatform>,
    password_autopurge: bool,
    pmrrbac_builder: PmrRbacBuilder,
    session_factory: SessionFactory,
}

mod impls;
