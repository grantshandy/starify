use std::{collections::HashSet, sync::Arc};

use rspotify::{
    sync::Mutex, AuthCodeSpotify, Config, Credentials, OAuth, Token,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackedAuthorization {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scopes: HashSet<String>,
    token: Token,
}

impl Into<SpotifyClient> for PackedAuthorization {
    fn into(self) -> SpotifyClient {
        let mut client = AuthCodeSpotify::default();

        client.creds = Credentials {
            id: self.client_id,
            secret: Some(self.client_secret),
        };
        client.oauth = OAuth {
            redirect_uri: self.redirect_uri,
            scopes: self.scopes,
            ..Default::default()
        };
        client.token = Arc::new(Mutex::new(Some(self.token)));
        client.config = Config {
            token_cached: false,
            ..Default::default()
        };

        SpotifyClient(client)
    }
}

impl PackedAuthorization {
    pub async fn from_auth(value: AuthCodeSpotify) -> Self {
        #[cfg(feature = "ssr")]
        let token = value.token.lock().await.unwrap();

        #[cfg(feature = "hydrate")]
        let token = value.token.lock().unwrap();

        Self {
            client_id: value.creds.id,
            client_secret: value.creds.secret.unwrap_or_default(),
            redirect_uri: value.oauth.redirect_uri,
            scopes: value.oauth.scopes,
            token: token.clone().unwrap_or_default(),
        }
    }
}

pub struct SpotifyClient(pub AuthCodeSpotify);


