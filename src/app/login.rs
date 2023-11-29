use leptos::*;
use leptos_router::*;

cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use http::{header, HeaderValue, StatusCode};
    use time::{Duration, OffsetDateTime};
    use rspotify::Credentials;


    const SCOPES: [&str; 2] = ["user-top-read", "user-follow-read"];
    const LOGIN_STATE_KEY: &str = "login_state";

    fn get_domain() -> String {
        use_context::<crate::Domain>()
            .expect("no domain provided")
            .0
    }
}}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <h1>"Login Page"</h1>
        <Await
            future=|| get_login_url()
            let:url_result
        >
            <a href=url_result.as_ref().expect("get login URL").to_owned()>"Login"</a>
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

        expect_context::<leptos_axum::ResponseOptions>().insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_KEY, &state)
                    .max_age(Duration::hours(1))
                    .path("/")
                    .same_site(SameSite::None)
                    .domain(get_domain())
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );

        let spotify_credentials =
            use_context::<Credentials>().expect("no spotify credentials provided");
        let leptos_options = use_context::<LeptosOptions>().expect("no leptos options provided");

        let redirect_uri = format!("http://{}/callback", leptos_options.site_addr);
        let client_id = spotify_credentials.id;

        return Ok(format!("https://accounts.spotify.com/authorize?response_type=code&client_id={client_id}&scope={}&redirect_uri={redirect_uri}&state={state}", SCOPES.join("%20")));
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
                Ok(Some(code)) => view!{ <p>"yay! "</p><p>{code}</p> }.into_view(),
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

        // set the LOGIN_STATE_KEY as null and set the expiration date to zero so the browser removes it.
        expect_context::<leptos_axum::ResponseOptions>().insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_KEY, "")
                    .expires(OffsetDateTime::UNIX_EPOCH)
                    .path("/")
                    .same_site(SameSite::None)
                    .domain(get_domain())
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );

        if let Ok(Some(code)) = &res {
            tracing::info!("got it! {code}");
            leptos_axum::redirect("/about");
        }

        return res;
    }
}
