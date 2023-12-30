use std::{collections::HashSet, sync::Arc};

use cfg_if::cfg_if;
use rspotify::{
    sync::Mutex, AuthCodeSpotify, Config, Credentials, OAuth, Token, ClientError, clients::OAuthClient, model::PrivateUser,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackedClient {
    client_id: String,
    client_secret: String,
    pub user_id: String,
    redirect_uri: String,
    scopes: HashSet<String>,
    token: Token,
}

impl Into<SpotifyClient> for PackedClient {
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

        SpotifyClient {
            client,
            user_id: self.user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpotifyClient {
    pub client: AuthCodeSpotify,
    pub user_id: String,
}

impl SpotifyClient {
    pub async fn packed(self) -> PackedClient {
        #[cfg(feature = "ssr")]
        let token = self.client.token.lock().await.unwrap();

        #[cfg(feature = "hydrate")]
        let token = self.client.token.lock().unwrap();

        PackedClient {
            user_id: self.user_id,
            client_id: self.client.creds.id,
            client_secret: self.client.creds.secret.unwrap_or_default(),
            redirect_uri: self.client.oauth.redirect_uri,
            scopes: self.client.oauth.scopes,
            token: token.clone().unwrap_or_default(),
        }
    } 

    pub async fn current_user(&self) -> Result<PrivateUser, ClientError> {
        cfg_if! {
            if #[cfg(feature = "hydrate")] {
                self.client.current_user()
            } else {
                self.client.current_user().await
            }
        }
    }
}


