use leptos::*;
use leptos_router::*;

pub const CLIENT_ID: &str = env!("SPOTIFY_CLIENT_ID");
pub const SECRET: &str = env!("SPOTIFY_CLIENT_SECRET");

pub const SCOPES: [&str; 2] = ["user-top-read", "user-follow-read"];
pub const REDIRECT_URI: &str = concat!("http://", env!("LEPTOS_SITE_ADDR"), "/callback");

const LOGIN_STATE_COOKIE: &str = "login_state";

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <h1>"Login Page"</h1>
        <Await
            future=|| get_login_url()
            let:url_result
        >
            <a href=url_result.as_ref().unwrap().to_owned()>"Login"</a>
        </Await>
    }
}

#[server(Login)]
async fn get_login_url() -> Result<String, ServerFnError> {
    let now = time::OffsetDateTime::now_utc();
    let state = now.unix_timestamp().to_string();

    #[cfg(feature = "ssr")]
    {
        use http::header::{self, HeaderValue};
        use axum_extra::extract::cookie::Cookie;
        
        let response = expect_context::<leptos_axum::ResponseOptions>();
        
        response.insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_COOKIE, &state)
                    .max_age(time::Duration::minutes(5))
                    // .path("/")
                    // .same_site(axum_extra::extract::cookie::SameSite::Lax)
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );
    }

    tracing::info!("made new URL");

    return Ok(format!("https://accounts.spotify.com/authorize?response_type=code&client_id={CLIENT_ID}&scope={}&redirect_uri={REDIRECT_URI}&state={state}", SCOPES.join("%20")));
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
                Ok(false) => error_msg("Login Link Expired"),
                _ => view! { <p>"You shouldn't see this..."</p> }.into_view()
            }}
        </Await>
        <A href="/">"Return to Main Page"</A>
    }
}

#[server(LoginCallBack)]
async fn validate_login_callback() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        tracing::info!("validating login callback");

        use axum::extract::Query;
        use axum_extra::extract::CookieJar;

        #[derive(serde::Deserialize, Debug)]
        struct CallbackQuery {
            pub code: String,
            pub state: i64,
        }

        let res = leptos_axum::extract(|query: Query<CallbackQuery>, jar: CookieJar| async move {
            tracing::info!("{query:?}");
            tracing::info!("{:?}", jar.get(LOGIN_STATE_COOKIE));

            jar.get(LOGIN_STATE_COOKIE)
                .map(|s| s.value())
                .map(|s| s.parse::<i64>().ok())
                .flatten()
                .is_some_and(|cookie_state| cookie_state == query.state)
                .then(|| query.code.to_owned())
        })
        .await
        .map_err(|err| ServerFnError::ServerError(format!("Could not extract query: {err:?}")));

        tracing::info!("{res:?}");

        if let Ok(Some(code)) = res.as_ref() {
            tracing::info!("got valid login callback");

            // expect_context::<leptos_axum::ResponseOptions>().insert_header(
            //     http::header::SET_COOKIE,
            //     http::header::HeaderValue::from_str(&format!("login_state=foo; Expires=Thu, 31 Oct 2000 00:00:00 GMT;"))
            //         .expect("create cookie HeaderValue"),
            // );

            tracing::info!("{code}");
            tracing::info!("TODO: add user to DB or start session?");
            leptos_axum::redirect("/about");
        }

        return res.map(|code| code.is_some());
    }
}
