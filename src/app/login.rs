use std::collections::HashSet;

use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;

use crate::{
    errors::{AppError, ErrorTemplate},
    CALLBACK_ENDPOINT, LOGIN_STATE_KEY, SPOTIFY_SCOPES,
};

cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use rspotify::{AuthCodeSpotify, Config, OAuth, scopes, Credentials};
    use http::{header, HeaderValue};
    use time::{Duration, OffsetDateTime};
}}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="grow hero">
            <div class="hero-content flex-col lg:flex-row-reverse">
                <img src="https://static.observableusercontent.com/thumbnail/58460abd4408b66660e76009e84ac91f2f27bb2ab789c09512cffe13ffe48725.jpg" class="max-w-sm rounded-lg shadow-2xl" />
                <div class="space-y-6">
                    <h1 class="text-5xl font-bold">"Musiscope"</h1>
                    <p>"View Artists in Constellations"</p>
                    <div class="flow-root">
                        <Await
                            future=|| get_login_url()
                            let:url_result
                        >
                            <a class="float-left btn btn-primary" href=url_result.as_ref().expect("get login URL")>"Link to Spotify"</a>
                        </Await>
                        <A href="/about" class="float-right btn">"About"</A>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Creates a unique spotify login URL and attaches the current unix time
/// to the state-passthrough to the URL & client cookie to validate.
#[server(Login)]
async fn get_login_url() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = OffsetDateTime::now_utc().unix_timestamp().to_string();
        let site_addr = use_context::<LeptosOptions>()
            .expect("no leptos options provided")
            .site_addr;

        expect_context::<leptos_axum::ResponseOptions>().insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_KEY, &state)
                    .max_age(Duration::hours(1))
                    .path("/")
                    .same_site(SameSite::None)
                    .domain(site_addr.ip().to_string())
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );

        let creds = use_context::<Credentials>().expect("no spotify credentials provided");
        let oauth = OAuth {
            redirect_uri: format!("http://{site_addr}{CALLBACK_ENDPOINT}"),
            state,
            scopes: HashSet::from(SPOTIFY_SCOPES.map(|s| s.into())),
            ..Default::default()
        };
        let config = Config {
            token_cached: false,
            ..Default::default()
        };

        let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
        return Ok(spotify.get_authorize_url(true).expect("Client Error"));
    }
}
