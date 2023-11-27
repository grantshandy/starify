use axum::{
    body::Body,
    extract::State,
    http::{
        header,
        Request, Uri,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use leptos::LeptosOptions;
use leptos_axum::{generate_route_list, LeptosRoutes};

use musiscope::app::App;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // get leptos configuration from environment variables injected by cargo-leptos
    let conf = leptos::get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;

    let router = Router::new()
        .route(
            "/api/*fn_name",
            get(leptos_axum::handle_server_fns).post(leptos_axum::handle_server_fns),
        )
        .leptos_routes(&conf.leptos_options, generate_route_list(App), App)
        .fallback(static_handler)
        .with_state(conf.leptos_options);

    tracing::info!("Listening on http://{addr}/");
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("serve HTTP")
}

#[derive(rust_embed::RustEmbed)]
#[folder = "$LEPTOS_SITE_ROOT/"]
struct Asset;

async fn static_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => leptos_axum::render_app_to_stream(
            options.to_owned(),
            move || leptos::view! { <App /> },
        )(req)
        .await
        .into_response(),
    }
}
