use leptos::*;

#[cfg(feature = "ssr")]
use {
    crate::LOGIN_STATE_KEY,
    axum_extra::extract::cookie::{Cookie, SameSite},
    http::{header, HeaderValue},
    time::{Duration, OffsetDateTime},
};

#[component]
pub fn SpotifyButton() -> impl IntoView {
    view! {
        <Await future=get_login_url let:url_result>
            <a class="btn btn-primary" href=url_result.as_ref().expect("get login URL")>
                "Link to Spotify"
            </a>
        </Await>
    }
}

/// Creates a unique spotify login URL and attaches the current unix time
/// to the state-passthrough to the URL & client cookie to validate.
#[server(Login)]
pub async fn get_login_url() -> Result<String, ServerFnError> {
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
        let mut spotify = use_context::<crate::auth::AuthSession>()
            .expect("provide auth session")
            .backend.client_base();

        spotify.oauth.state = state;

        return Ok(spotify.get_authorize_url(true).expect("Client Error"));
    }
}
