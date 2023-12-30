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
    let url = create_resource(|| (), |_| async move { get_login_url().await });

    view! {
        <Suspense fallback=|| {
            view! { <span class="btn">"Loading Link..."</span> }
        }>
            {url
                .get()
                .map(|res| {
                    res
                        .map(|url| {
                            view! {
                                <a href=url class="btn btn-primary">
                                    "Link to Spotify"
                                </a>
                            }
                                .into_view()
                        })
                        .unwrap_or_default()
                })}
        </Suspense>
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

        let Some(url) = use_context::<crate::auth::AuthSession>()
            .expect("no auth session provided")
            .backend
            .authorize_url(state) else {
                return Err(ServerFnError::ServerError("Authorization URL Error".to_string()))
            };

        return Ok(url);

    }
}

