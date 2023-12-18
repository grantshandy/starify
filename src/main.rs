use std::{env, net::SocketAddr};

use axum::{
    body::Body as AxumBody,
    extract::{Path, RawQuery, State},
    http::{header, HeaderMap, Request, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use color_eyre::eyre;
use leptos_axum::{generate_route_list, LeptosRoutes};

use musiscope::{app::App, AppState};
use rspotify::Credentials;

/// CLI for musiscope
#[derive(argh::FromArgs)]
struct Args {
    #[argh(option, description = "socket to serve on")]
    socket: Option<SocketAddr>,

    #[argh(option, description = "spotify client id (also set through SPOTIFY_CLIENT_ID)")]
    client_id: Option<String>,

    #[argh(option, description = "spotify client secret (also set through SPOTIFY_CLIENT_SECRET)")]
    client_secret: Option<String>,

    #[argh(
        option,
        description = "what domain the site is served on (defaults to socket)"
    )]
    domain: Option<String>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let args: Args = argh::from_env();

    // get leptos configuration from environment variables injected by cargo-leptos
    let mut conf = leptos::get_configuration(None).await.unwrap();

    if let Some(socket) = args.socket {
        conf.leptos_options.site_addr = socket;
    }

    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: conf.leptos_options.clone(),
        routes: routes.clone(),
        spotify_credentials: Credentials {
            id: args.client_id.unwrap_or(env::var("SPOTIFY_CLIENT_ID")?),
            secret: Some(
                args.client_secret
                    .unwrap_or(env::var("SPOTIFY_CLIENT_SECRET")?),
            ),
        },
    };

    let router = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(static_handler)
        .with_state(app_state);

    tracing::info!("Listening on http://{addr}/");
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

#[derive(rust_embed::RustEmbed)]
#[folder = "$LEPTOS_SITE_ROOT/"]
struct Asset;

async fn static_handler(
    uri: Uri,
    State(state): State<AppState>,
    req: Request<AxumBody>,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => leptos_axum::render_app_to_stream(
            state.leptos_options.to_owned(),
            move || leptos::view! { <App /> },
        )(req)
        .await
        .into_response(),
    }
}

async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> impl IntoResponse {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || provide_state_context(app_state.clone()),
        App,
    );

    handler(req).await
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    path: Path<String>,
    headers: HeaderMap,
    raw_query: RawQuery,
    req: Request<AxumBody>,
) -> impl IntoResponse {
    leptos_axum::handle_server_fns_with_context(
        path,
        headers,
        raw_query,
        move || provide_state_context(app_state.clone()),
        req,
    )
    .await
}

fn provide_state_context(app_state: AppState) {
    leptos::provide_context(app_state.spotify_credentials);
    leptos::provide_context(app_state.leptos_options);
}
