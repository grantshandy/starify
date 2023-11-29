pub mod app;
pub mod error_template;

cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::extract::FromRef;
    
    #[derive(FromRef, Debug, Clone)]
    pub struct AppState {
        pub leptos_options: leptos::LeptosOptions,
        pub routes: Vec<leptos_router::RouteListing>,
        pub spotify_credentials: rspotify::Credentials,
        pub domain: Domain,
    }

    #[derive(FromRef, Debug, Clone)]
    pub struct Domain(pub String);
}}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    #[cfg(debug_assertions)]
    let trace_level = tracing::Level::DEBUG;
    #[cfg(not(debug_assertions))]
    let trace_level = tracing::Level::INFO;

    tracing_subscriber::fmt()
        .with_writer(
            tracing_subscriber_wasm::MakeConsoleWriter::default().map_trace_level_to(trace_level),
        )
        .without_time()
        .init();

    console_error_panic_hook::set_once();

    tracing::info!("Mounting Leptos!");

    leptos::mount_to_body(app::App);
}
