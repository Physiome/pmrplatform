use axum::{
    extract::Request,
    http::HeaderValue,
    routing::get,
    response::Redirect,
    Router,
};
use clap::Parser;
use pmr2_gateway::{Cli, GitGateway};
use std::iter::empty;
use tower_http::trace::TraceLayer;

// Invalidation targets when some exposure has been modified.
static EXPOSURE_INVAL_TARGETS: [&str; 26] = [
    // Exposure main listings.
    "/e",
    "/exposure",
    "/full-list",

    // Exposure listing by "categories".
    "/calcium_dynamics",
    "/cardiovascular_circulation",
    "/cell_cycle",
    "/cell_migration",
    "/circadian_rhythms",
    "/electrophysiology",
    "/endocrine",
    "/excitation-contraction_coupling",
    "/gene_regulation",
    "/hepatology",
    "/immunology",
    "/ion_transport",
    "/mechanical_constitutive_laws",
    "/metabolism",
    "/msk",
    "/myofilament_mechanics",
    "/neurobiology",
    "/ph_regulation",
    "/PKPD",
    "/protein-modules",
    "/protein_modules",  // This is actually "Ion channels" for some reason...
    "/signal_transduction",
    "/synthetic_biology",
];
// Invalidation targets when some workspace has been modified.
static WORKSPACE_INVAL_TARGETS: [&str; 1] = [
    "/workspace"
];

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    // This gateway will be cloned because the semaphores must be shared in order for the rate-limiting
    // to function as intended.
    let gateway = cli.gateway();

    let git_gateway = GitGateway::new(
        gateway.clone(),
        &cli.pmr2_root,
    );

    // This is the gateway set up with the recursive invalidation method, this is done so for all exposure
    // content.  Ideally workspace cache invalidation should have a smaller scope (namely just the main
    // listing and the log/shortlog), but I do have time constraints to do this and this fine level of
    // invalidation will have to wait.
    let gateway_imr = gateway.with_recursive_invalidation_method();

    // These routers are to be nested; only a fallback is defined as that's the default handler, and it
    // will deal with the root route.  This is set up in such a manner because it isn't possible to define
    // a wildcard route with trailing static content, i.e. "/e/{exposure_id}/content_status_modify" will
    // conflict with "/e/{*path}".  Using a fallback on a nested router completely side-step this wildcard
    // routing conflict issue.
    let exposure_router = Router::new()
        .route_service(
            "/{exposure_id}/content_status_history",
            gateway_imr.with_invalidation_targets(EXPOSURE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{exposure_id}/content_status_modify",
            gateway_imr.with_invalidation_targets(EXPOSURE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{exposure_id}/wizard",
            gateway_imr.with_invalidation_targets(EXPOSURE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{exposure_id}/@@wizard",
            gateway_imr.with_invalidation_targets(EXPOSURE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{exposure_id}/{*path}",
            gateway_imr.clone(),
        )
        .route_service(
            "/{exposure_id}/",
            gateway_imr.with_single_invalidation_method(),
        );
    let mut workspace_router = Router::new()
        .route_service(
            "/{workspace_id}/content_status_history",
            gateway_imr.with_invalidation_targets(WORKSPACE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{workspace_id}/content_status_modify",
            gateway_imr.with_invalidation_targets(WORKSPACE_INVAL_TARGETS.into_iter()),
        )
        .route_service(
            "/{workspace_id}/{*path}",
            gateway_imr.clone(),
        )
        .route_service(
            "/{workspace_id}/",
            gateway_imr.with_single_invalidation_method(),
        );

    // Workspace within the user workspace are nested one extra level with an arbitrary identifier (user_id),
    // but unfortunately `Path` via the `FromRequestParts::from_request_parts` will end up capturing this
    // additional item.  While next version of `axum` will resolve this via `InnerPath`, this isn't released
    // yet so the handling is deferred to the configuration offered by the wrapper gateway service.
    let nested_workspace_router = workspace_router.clone()
        .route_service(
            "/{workspace_id}/download/{commit_id}/{*path}",
            git_gateway.nested().for_view("download"),
        )
        .route_service(
            "/{workspace_id}/@@download/{commit_id}/{*path}",
            git_gateway.nested().for_view("download"),
        )
        .route_service(
            "/{workspace_id}/rawfile/{commit_id}/{*path}",
            git_gateway.nested().for_view("rawfile"),
        )
        .route_service(
            "/{workspace_id}/@@rawfile/{commit_id}/{*path}",
            git_gateway.nested().for_view("rawfile"),
        )
        .route_service(
            "/{workspace_id}/archive/{commit_id}/tgz",
            git_gateway.nested().for_view("archive-tgz"),
        )
        .route_service(
            "/{workspace_id}/archive/{commit_id}/zip",
            git_gateway.nested().for_view("archive-zip"),
        )
        .route_service(
            "/{workspace_id}/@@archive/{commit_id}/tgz",
            git_gateway.nested().for_view("archive-tgz"),
        )
        .route_service(
            "/{workspace_id}/@@archive/{commit_id}/zip",
            git_gateway.nested().for_view("archive-zip"),
        );

    workspace_router = workspace_router
        .route_service(
            "/{workspace_id}/download/{commit_id}/{*path}",
            git_gateway.clone().for_view("download"),
        )
        .route_service(
            "/{workspace_id}/@@download/{commit_id}/{*path}",
            git_gateway.clone().for_view("download"),
        )
        .route_service(
            "/{workspace_id}/rawfile/{commit_id}/{*path}",
            git_gateway.clone().for_view("rawfile"),
        )
        .route_service(
            "/{workspace_id}/@@rawfile/{commit_id}/{*path}",
            git_gateway.clone().for_view("rawfile"),
        )
        .route_service(
            "/{workspace_id}/archive/{commit_id}/tgz",
            git_gateway.clone().for_view("archive-tgz"),
        )
        .route_service(
            "/{workspace_id}/archive/{commit_id}/zip",
            git_gateway.clone().for_view("archive-zip"),
        )
        .route_service(
            "/{workspace_id}/@@archive/{commit_id}/tgz",
            git_gateway.clone().for_view("archive-tgz"),
        )
        .route_service(
            "/{workspace_id}/@@archive/{commit_id}/zip",
            git_gateway.clone().for_view("archive-zip"),
        );

    let router = Router::new()
        // As this gateway is intended to handle the two other alternate hosts, this will need the appropriate
        // redirect that used to be done by the underlying vhost handler in Plone.  Incidentally this also
        // serves as an optimization to avoid a connection to that underlying instance.
        .route("/", get(|request: Request| async move {
            match request.headers().get("x-forwarded-host") {
                Some(s) if s == HeaderValue::from_static("models.cellml.org") =>
                    Redirect::permanent("/cellml"),
                Some(s) if s == HeaderValue::from_static("models.fieldml.org") =>
                    Redirect::permanent("/fieldml"),
                _ => Redirect::permanent("/welcome"),
            }
        }))
        .nest("/e/", exposure_router.clone())
        .nest("/exposure/", exposure_router)
        .nest("/w/{user_id}/", nested_workspace_router)
        .nest("/workspace/", workspace_router)
        // Ensure that modification to the root item don't just cause the site-wide invalidation.
        // Unfortunately, the following will not work until <https://github.com/ibraheemdev/matchit/pull/95>
        // is merged and `axum` has a release that uses the new release.  The goal is to have items that will
        // invalidate itself on specific endpoints.
        //
        // .route_service(
        //     "/{*path}/content_status_modify",
        //     gateway.with_invalidation_targets(empty::<String>())
        //         .with_single_invalidation_method(),
        // )
        // .route_service(
        //     "/{*path}/edit",
        //     gateway.with_invalidation_targets(empty::<String>())
        //         .with_single_invalidation_method(),
        // )
        //
        // Instead, the following suboptimal workaround is used.  The goal is to ensure that `POST` to the
        // endpoints off the root item will not be messed with as it would not have contain the additional
        // segments that would be captured by the wildcard `{*path}` match (e.g. `/login`), while any item
        // with any endpoint will be captured by this route (e.g. `/docs/edit`), which will invalidate the
        // item only.
        //
        // Naturally, doing so on a Folder type will invalidate all the content within the folder, but this
        // at least would be more contained than the root level.
        .route_service(
            "/{item}/{*path}",
            gateway
                .with_invalidation_targets(empty::<String>())
                .with_single_invalidation_method(),
        )
        .fallback_service(gateway)
        .layer(cli.rewrite_pmr2_layer())
        .layer(TraceLayer::new_for_http());

    let bind_addr = cli.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
