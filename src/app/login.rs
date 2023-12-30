use leptos::*;
use leptos_router::*;

use serde::{Serialize, Deserialize};

#[cfg(feature = "ssr")]
use {
    crate::{LOGIN_STATE_KEY, client},
    axum_extra::extract::cookie::{Cookie, SameSite},
    http::{header, HeaderValue},
    time::{Duration, OffsetDateTime},
};

#[component]
pub fn SpotifyButtons(
) -> impl IntoView {
    let login_info = create_resource(|| (), |_| async move { get_login_info().await });

    view! {
        <Suspense>
            // match destructuring :)))
            {move || match login_info.get() {
                Some(Ok(LoginInfo { url, user: Some(name) })) => {
                    view! {
                        <A href="/dashboard" class="btn btn-primary">
                            "Continue as "
                            {name}
                        </A>
                        <a href=url class="btn btn-xs">
                            "Continue as Other User"
                        </a>
                    }
                        .into_view()
                }
                Some(Ok(LoginInfo { url, user: None })) => {
                    view! {
                        <a href=url class="btn btn-primary">
                            "Link to Spotify"
                        </a>
                    }
                        .into_view()
                }
                Some(Err(err)) => view! { <p>"Error: " {err.to_string()}</p> }.into_view(),
                None => view! { <span class="btn">"Loading Buttons"</span> }.into_view(),
            }}

        </Suspense>
    }
}

/// Creates a unique spotify login URL and attaches the current unix time
/// to the state-passthrough to the URL & client cookie to validate.
#[server(Login)]
pub async fn get_login_info() -> Result<LoginInfo, ServerFnError> {
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

        let auth_session = use_context::<crate::auth::AuthSession>()
            .expect("no auth session provided");

        let Some(url) = auth_session
            .backend
            .authorize_url(state) else {
                return Err(ServerFnError::ServerError("Authorization URL Error".to_string()))
            };

        return Ok(LoginInfo {
            user: client::get_current_user().await?.map(|user| user.display_name.unwrap_or("Unknown User".to_string())),
            url
        });

    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub user: Option<String>,
    pub url: String,
}
