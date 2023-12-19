use async_trait::async_trait;
use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, RwLock},
};
use thiserror::Error;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_login::{AuthUser, AuthnBackend, UserId};
use http::{header, HeaderValue, StatusCode};
use rspotify::{clients::OAuthClient, AuthCodeSpotify};

use crate::{User, LOGIN_STATE_KEY};

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
    };

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
        [(header::LOCATION, HeaderValue::from_static("/me"))],
    )
        .into_response();
}

impl AuthUser for User {
    type Id = rspotify::model::idtypes::UserId<'static>;

    fn id(&self) -> Self::Id {
        self.me.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        Borrow::<str>::borrow(&self.me.id).as_bytes()
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
}

#[derive(Debug, Clone)]
pub struct Backend {
    client: AuthCodeSpotify,
    db: Arc<RwLock<HashMap<UserId<Self>, User>>>,
}

impl Backend {
    pub fn new(client: AuthCodeSpotify) -> Self {
        Self {
            client,
            db: Arc::new(RwLock::new(HashMap::default())),
        }
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

        let user = User {
            me: client
                .current_user()
                .await
                .map_err(Error::Spotify)?,
            client,
        };

        let db = &mut self.db.write().unwrap();
        db.insert(user.id().clone(), user.clone());

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.db.read().unwrap().get(user_id).cloned())
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
