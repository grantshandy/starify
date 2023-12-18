use leptos::*;
use leptos_router::*;


cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use rspotify::{AuthCodeSpotify, Config, OAuth, scopes, Credentials};
    use http::{header, HeaderValue, StatusCode};
    use time::{Duration, OffsetDateTime};

    const CALLBACK_ENDPOINT: &str = "/authorize";
    const LOGIN_STATE_KEY: &str = "login_state";
}}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <h1>"Login Page"</h1>
        <Await
            future=|| get_login_url()
            let:url_result
        >
            <a class="btn" href=url_result.as_ref().expect("get login URL").to_owned()>"Login"</a>
        </Await>
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
            scopes: scopes!("user-top-read", "user-follow-read"),
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

#[component]
pub fn LoginCallback() -> impl IntoView {
    let error_msg =
        move |msg: &str| view! { <p>"Error logging in: " {msg.to_string()}</p> }.into_view();

    view! {
        <Await
            future=|| validate_login_callback()
            let:res
        >
            {match res.as_ref() {
                Err(err) => error_msg(&err.to_string()),
                Ok(None) => error_msg("Login Link Expired"),
                Ok(Some(code)) => view!{ <p>{code}</p> }.into_view(),
            }}
        </Await>
        <A href="/">"Return to Main Page"</A>
    }
}

/// This is the endpoint spotify redirects back to with the code & previous state value after authentication.
/// From here, we error out or redirect back to the main page, registering the client in the database.
#[server(LoginCallBack)]
async fn validate_login_callback() -> Result<Option<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Query;
        use axum_extra::extract::{
            cookie::{Cookie, SameSite},
            CookieJar,
        };

        #[derive(serde::Deserialize, Debug)]
        struct CallbackQuery {
            pub code: String,
            pub state: i64,
        }

        let res = leptos_axum::extract(|query: Query<CallbackQuery>, jar: CookieJar| async move {
            jar.get(LOGIN_STATE_KEY)
                .map(|s| s.value())
                .map(|s| s.parse::<i64>().ok())
                .flatten()
                .is_some_and(|cookie_state| cookie_state == query.state)
                .then(|| query.code.to_owned())
        })
        .await
        .map_err(|err| ServerFnError::ServerError(format!("Could not extract query: {err:?}")));

        // set the page status code depending on the status of the state validation
        expect_context::<leptos_axum::ResponseOptions>().set_status(match &res {
            Ok(Some(_)) => StatusCode::FOUND,
            Ok(None) => StatusCode::GONE,
            Err(_) => StatusCode::BAD_REQUEST,
        });

        let site_addr = use_context::<LeptosOptions>()
            .expect("no leptos options provided")
            .site_addr;

        // set the LOGIN_STATE_KEY as null and set the expiration date to zero so the browser removes it.
        expect_context::<leptos_axum::ResponseOptions>().insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_KEY, "")
                    .expires(OffsetDateTime::UNIX_EPOCH)
                    .path("/")
                    .same_site(SameSite::None)
                    .domain(site_addr.ip().to_string())
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );

        if let Ok(Some(code)) = &res {
            tracing::info!("got it! {code}");
        }

        return res;
    }
}
