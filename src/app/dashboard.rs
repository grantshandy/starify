use leptos::*;
use rspotify::model::PrivateUser;

use crate::client;

#[component]
pub fn Dashboard() -> impl IntoView {
    let client = create_resource(|| (), |_| async move {
        client::get_current_user().await });
    

    view! {
        <div class="grow">
            <Suspense>
                {move || match client.get() {
                    Some(Ok(Some(user))) => Some(view! { <User user=user/> }.into_view()),
                    Some(Ok(None)) => {
                        cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
                            leptos_axum::redirect("/");
                        } else {
                            leptos_router::use_navigate()("/", Default::default());
                        }}

                        None
                    },
                    Some(Err(err)) => Some(view! { <p>"An Error " {err.to_string()}</p> }.into_view()),
                    None => Some(view! { <p>"Loading View"</p> }.into_view()),
                }}

            </Suspense>
        </div>
    }
}

#[component]
fn User(
    user: PrivateUser
) -> impl IntoView {
    let img_url = user.images.expect("no profile pictures").get(0).expect("no profile picture").url.clone();

    view! {
        <div class="card w-96 bg-base-100 shadow-xl image-full">
            <div class="card-body">
                <h2 class="card-title">
                    <div class="avatar">
                        <div class="w-12 mask mask-squircle">
                            <img src={img_url} />
                        </div>
                    </div>
                    {user.display_name.unwrap_or("Unknown Username".to_string())}
                </h2>
                <p>If a dog chews shoes whose shoes does he choose?</p>
                <div class="card-actions justify-end">
                    <button class="btn btn-primary">Buy Now</button>
                </div>
            </div>
        </div>
    }
}

