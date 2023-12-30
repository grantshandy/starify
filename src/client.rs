use leptos::*;
use rspotify::model::PrivateUser;

cfg_if::cfg_if! {   
    if #[cfg(feature = "ssr")] {
        use crate::auth::AuthSession;
        use serde::{de::DeserializeOwned, Serialize};

        lazy_static::lazy_static! {
            pub static ref DATABASE: sled::Db = sled::open(std::env::var("STARIFY_CACHE").unwrap_or("starify_cache".to_string())).expect("create database");
        }

        pub async fn get_from_db<V: DeserializeOwned>(key: &str) -> Result<Option<V>, sled::Error> {
            DATABASE.get(key).map(|out| out.map(|out| bincode::deserialize(&out).expect("parse as bincode")))
        }

        pub async fn put_to_db<V: Serialize>(key: &str, value: V) -> Result<Option<V>, sled::Error> {
            DATABASE.insert(key, bincode::serialize(&value).expect("parse to bincode"))?;

            Ok(Some(value))
        }
    }
}

#[server]
pub async fn get_current_user() -> Result<Option<PrivateUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rspotify::clients::OAuthClient;

        let Some(user) = use_context::<AuthSession>()
            .expect("no auth session provided")
            .user else {
                return Ok(None);
            };

        let userinfo_key = format!("{}_userinfo", user.user_id);

        // get user from cache from user id
        match get_from_db::<PrivateUser>(&userinfo_key).await {
            // if successful & exists, deserialize the result
            Ok(Some(user)) => Ok(Some(user)),
            // if unsuccessful or doesn't exist, fetch from API
            _ => match user.client.current_user().await {
                // if successful, insert that into the database
                Ok(me) => put_to_db::<PrivateUser>(&userinfo_key, me).await.map_err(|err| ServerFnError::ServerError(format!("Error inserting into cache: {err}"))),
                // if API failed, err out to client
                Err(err) => Err(ServerFnError::ServerError(format!("Error fetching from spotify: {err}")))
            }
        }
    }
}

