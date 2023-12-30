use async_trait::async_trait;
use thiserror::Error;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_login::{AuthUser, AuthnBackend, UserId};
use http::{header, HeaderValue, StatusCode};
use rspotify::{clients::{OAuthClient, BaseClient}, AuthCodeSpotify, Token};

use crate::{LOGIN_STATE_KEY, client};

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(serde::Deserialize, Debug)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: i64,
}

pub async fn authorize(
    mut auth_session: AuthSession,
    query: Query<CallbackQuery>,
    jar: CookieJar,
) -> impl IntoResponse {
    let state_cookie = jar.get(LOGIN_STATE_KEY);

    if query.code.is_none()
        || state_cookie.is_none()
        || query.state.to_string() != state_cookie.unwrap().value()
    {
        return (
            StatusCode::SEE_OTHER,
            jar.remove(Cookie::named(LOGIN_STATE_KEY)),
            [(header::LOCATION, HeaderValue::from_static("/"))],
        )
            .into_response();
    }

    let user = match auth_session
        .authenticate(Credentials {
            code: query.code.clone().unwrap(),
        })
        .await
    {
        Ok(Some(user)) => user,
        _ => return Redirect::to("/").into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    return (
        StatusCode::SEE_OTHER,
        jar.remove(Cookie::named(LOGIN_STATE_KEY)),
        [(header::LOCATION, HeaderValue::from_static("/dashboard"))],
    )
        .into_response();
}

#[derive(Clone, Debug)]
pub struct User {
    pub client: AuthCodeSpotify,
    pub user_id: String,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.user_id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.user_id.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub code: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Spotify(rspotify::ClientError),

    #[error(transparent)]
    Sled(sled::Error)
}

#[derive(Debug, Clone)]
pub struct Backend {
    client: AuthCodeSpotify,
}

impl Backend {
    pub fn new(client: AuthCodeSpotify) -> Self {
        Self {
            client,
        }
    }

    pub fn authorize_url(&self, state: String) -> Option<String> {
        let mut client = self.client.clone();

        client.oauth.state = state;

        client.get_authorize_url(true).ok()
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let client = self.client.clone();

        client
            .request_token(&creds.code)
            .await
            .map_err(Error::Spotify)?;

        let me = client.current_user().await.map_err(Error::Spotify)?;

        let user = User {
            client,
            user_id: me.id.to_string(),
        };

        let token = user
            .client
            .get_token()
            .lock()
            .await
            .expect("lock client token")
            .clone()
            .expect("get client token");

        client::put_to_db(&user.user_id, token)
            .await
            .map_err(Error::Sled)
            .map(|_| Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let client = self.client.clone();

        let Some(token) = client::get_from_db::<Token>(user_id)
            .await
            .map_err(Error::Sled)? else {
                return Ok(None);
            };
        
        *client.token.lock().await.expect("lock on token") = Some(token);

        let user = User {
            client,
            user_id: user_id.to_string(),
        };

        Ok(Some(user))
    }
}

