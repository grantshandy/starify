use http::StatusCode;
use leptos::*;
use leptos_router::*;

pub const CLIENT_ID: &str = env!("SPOTIFY_CLIENT_ID");
pub const SECRET: &str = env!("SPOTIFY_CLIENT_SECRET");

pub const SCOPES: [&str; 2] = ["user-top-read", "user-follow-read"];
pub const REDIRECT_URI: &str = concat!("http://", env!("LEPTOS_SITE_ADDR"), "/callback");
const DOMAIN: &str = "127.0.0.1";

const LOGIN_STATE_KEY: &str = "login_state";

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
        use axum_extra::extract::cookie::{self, Cookie};
        
        let response = expect_context::<leptos_axum::ResponseOptions>();
        
        response.insert_header(
            header::SET_COOKIE,
            HeaderValue::from_str(
                &Cookie::build(LOGIN_STATE_KEY, &state)
                    .max_age(time::Duration::minutes(5))
                    .path("/")
                    .same_site(cookie::SameSite::None)
                    .domain(DOMAIN)
                    .finish()
                    .to_string(),
            )
            .expect("create cookie HeaderValue"),
        );
    }

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
                Ok(None) => error_msg("Login Link Expired"),
                Ok(Some(code)) => view!{ <p>"yay! "</p><p>{code}</p> }.into_view(),
            }}
        </Await>
        <A href="/">"Return to Main Page"</A>
    }
}

#[server(LoginCallBack)]
async fn validate_login_callback() -> Result<Option<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Query;
        use axum_extra::extract::CookieJar;

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

        expect_context::<leptos_axum::ResponseOptions>().set_status(match &res {
            Ok(Some(_)) => StatusCode::FOUND,
            Ok(None) => StatusCode::GONE,
            Err(_) => StatusCode::BAD_REQUEST,
        });

        if res.as_ref().is_ok_and(|c| c.is_some()) {
            tracing::info!("got it! you should be put in a database");
        }

        return res;
    }
}
