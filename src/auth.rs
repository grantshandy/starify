use async_trait::async_trait;
use std::{collections::HashSet, sync::Arc};
use thiserror::Error;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_login::{AuthUser, AuthnBackend, UserId};
use http::{header, HeaderValue, StatusCode};
use rspotify::{clients::OAuthClient, AuthCodeSpotify, Config, OAuth};

use crate::{AppState, CALLBACK_ENDPOINT, LOGIN_STATE_KEY, SPOTIFY_SCOPES};

#[derive(serde::Deserialize, Debug)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: i64,
}

pub async fn authorize(
    State(app_state): State<AppState>,
    query: Query<CallbackQuery>,
    jar: CookieJar,
) -> impl IntoResponse {
    let state_cookie = jar.get(LOGIN_STATE_KEY);

    if query.code.is_none()
        || state_cookie.is_none()
        || &query.state.to_string() != state_cookie.unwrap().value()
    {
        return (
            StatusCode::SEE_OTHER,
            jar.remove(Cookie::named(LOGIN_STATE_KEY)),
            [(header::LOCATION, HeaderValue::from_static("/"))],
        )
            .into_response();
    };

    let creds = Credentials {
        code: query.code.clone().unwrap(),
        spotify: AuthCodeSpotify::with_config(
            app_state.spotify_credentials,
            OAuth {
                redirect_uri: format!(
                    "http://{}{CALLBACK_ENDPOINT}",
                    app_state.leptos_options.site_addr
                ),
                scopes: HashSet::from(SPOTIFY_SCOPES.map(|s| s.into())),
                ..Default::default()
            },
            Config {
                token_cached: false,
                ..Default::default()
            },
        ),
    };

    tracing::info!("AUTHENTICATING NOW");

    return (
        StatusCode::SEE_OTHER,
        jar.remove(Cookie::named(LOGIN_STATE_KEY)),
        [(header::LOCATION, HeaderValue::from_static("/"))],
    )
        .into_response();
}

#[derive(Debug, Clone)]
pub struct User {
    id: String,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.to_owned()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub code: String,
    pub spotify: AuthCodeSpotify,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Spotify(rspotify::ClientError),
}

#[derive(Default, Debug, Clone)]
pub struct Backend;

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if let Err(err) = creds.spotify.request_token(&creds.code).await {
            return Err(Error::Spotify(err));
        }

        todo!()
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        todo!()
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
