pub mod app;
pub mod errors;
pub mod client;

#[cfg(feature = "ssr")]
pub mod auth;

pub const CALLBACK_ENDPOINT: &str = "/authorize";
pub const LOGIN_STATE_KEY: &str = "login_state";
pub const SPOTIFY_SCOPES: [&str; 2] = ["user-top-read", "user-follow-read"];

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

