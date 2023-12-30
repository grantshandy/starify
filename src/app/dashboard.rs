use leptos::*;
use leptos_router::*;

use crate::{client::SpotifyClient, app::get_client};

#[component]
pub fn Dashboard() -> impl IntoView {
    let client = create_resource(|| (), |_| async move {
        get_client().await });

    view! {
        <Suspense fallback=|| {
            view! { <p>"Loading Client"</p> }
        }>
            {move || {
                client
                    .get()
                    .map(|client| {
                        view! {
                            <Loaded client=client
                                .expect("get client from server fn")
                                .expect("client found for user")
                                .into()/>
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn Loaded(client: SpotifyClient) -> impl IntoView {
    let (client, _) = create_signal(client);

    let user = create_resource(move || client.get(), |client| async move {
        client.current_user().await.map_err(|err| err.to_string())
    });

    view! {
        <Suspense fallback=|| {
            view! { <p>"Loading Profile"</p> }
        }>
            {move || {
                user
                    .get()
                    .map(|user| match user {
                        Ok(user) => view! { <p>"Username: " {user.display_name}</p> }.into_view(),
                        Err(err) => view! { <p>{err}</p> }.into_view(),
                    })
            }}

        </Suspense>
        <p>
            <A href="/">"To Main Page"</A>
        </p>
    }
}


