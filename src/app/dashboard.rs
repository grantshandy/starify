use leptos::*;
use rspotify::model::PrivateUser;

use crate::client;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="grow p-4">
            <User />
        </div>
    }
}

#[component]
pub fn User() -> impl IntoView {
    let client = create_resource(|| (), |_| async move {
        client::get_current_user().await });

    // "skeleton" <- THIS IS A LOAD BEARING COMMENT. I SHIT YOU NOT.

    let user_widget = |user: Option<PrivateUser>| view! {
        <div class="mx-auto w-32 text-center rounded-xl p-2 shadow-xl bg-base-400">
            <div class="avatar">
                <div class="w-12 mask mask-squircle" class:skeleton=user.is_none()>
                    {user
                        .as_ref()
                        .map(|user| {
                            view! {
                                <img src=user
                                    .images
                                    .clone()
                                    .expect("no profile pictures")
                                    .get(0)
                                    .expect("no profile picture")
                                    .url
                                    .clone()/>
                            }
                        })}
                </div>
            </div>
            <p class="text-xl font-bold" class:skeleton=user.is_none()>
                {user
                    .map(|user| user.display_name)
                    .flatten()
                    .unwrap_or_default()}
            </p>
        </div>
    }.into_view();

    view! {
        <Suspense fallback=move || user_widget(None)>
            {move || {
                client
                    .get()
                    .map(|client| match client {
                        Ok(Some(user)) => Some(user_widget(Some(user))),
                        Ok(None) => {
                            cfg_if::cfg_if! {
                                if #[cfg(feature = "ssr")] { leptos_axum::redirect("/"); } else
                                { leptos_router::use_navigate() ("/", Default::default()); }
                            }
                            None
                        }
                        Err(err) => {
                            Some(view! { <p>"An Error " {err.to_string()}</p> }.into_view())
                        }
                    })
            }}

        </Suspense>
    }
}
