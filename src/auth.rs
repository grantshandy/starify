use async_trait::async_trait;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};
use thiserror::Error;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_login::{AuthUser, AuthnBackend, UserId};
use http::{header, HeaderValue, StatusCode};
use rspotify::{clients::OAuthClient, model::PrivateUser, AuthCodeSpotify, Config, OAuth};

use crate::{AppState, CALLBACK_ENDPOINT, LOGIN_STATE_KEY, SPOTIFY_SCOPES};

#[derive(serde::Deserialize, Debug)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: i64,
}

pub async fn authorize(
    mut auth_session: AuthSession,
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

    auth_session.authenticate(Credentials {
        code: query.code.clone().unwrap(),
    }).await.expect("authenticate");

    return (
        StatusCode::SEE_OTHER,
        jar.remove(Cookie::named(LOGIN_STATE_KEY)),
        [(header::LOCATION, HeaderValue::from_static("/"))],
    )
        .into_response();
}

#[derive(Debug, Clone)]
pub struct User(pub PrivateUser);

impl AuthUser for User {
    type Id = rspotify::model::idtypes::UserId<'static>;

    fn id(&self) -> Self::Id {
        self.0.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        Borrow::<str>::borrow(&self.0.id).as_bytes()
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
    db: Arc<RwLock<HashMap<UserId<Self>, PrivateUser>>>,
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
        self.client
            .request_token(&creds.code)
            .await
            .map_err(|err| Error::Spotify(err))?;

        let user = User(self
            .client
            .current_user()
            .await
            .map_err(|err| Error::Spotify(err))?);

        let db = &mut self.db.write().unwrap();
        db.insert(user);

        Ok(Some(user))
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        todo!()
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
