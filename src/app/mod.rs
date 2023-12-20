use leptos::{html::Select, *};
use leptos_meta::*;
use leptos_router::*;
use rspotify::model::{FullArtist, TimeRange};

mod login;

use login::SpotifyButton;

use crate::{
    errors::{AppError, ErrorTemplate},
    User,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/musiscope.css"/>
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <div data-theme="light" class="min-h-screen flex flex-col">
                <Routes>
                    <Route path="/" view=LoginPage ssr=SsrMode::Async/>
                    <Route path="/about" view=AboutPage/>
                    <Route path="/me" view=Me ssr=SsrMode::Async/>
                </Routes>
                <footer class="footer footer-center p-4 bg-base-300 text-base-content">
                    <aside>
                        <p>"Copyright © 2023 Grant Handy"</p>
                    </aside>
                </footer>
            </div>
        </Router>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="grow hero">
            <div class="hero-content flex-col lg:flex-row-reverse">
                <img
                    src="https://static.observableusercontent.com/thumbnail/58460abd4408b66660e76009e84ac91f2f27bb2ab789c09512cffe13ffe48725.jpg"
                    class="max-w-sm rounded-lg shadow-2xl"
                />
                <div class="space-y-6">
                    <h1 class="text-5xl font-bold">"Musiscope"</h1>
                    <p>"View Artists in Constellations"</p>
                    <div class="flow-root">
                        <div class="float-left">
                            <SpotifyButton/>
                        </div>
                        <A href="/about" class="float-right btn">
                            "About"
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="grow">
            <h1>"About!"</h1>
            <A href="/">"Back to Home"</A>
        </div>
    }
}

#[component]
pub fn Me() -> impl IntoView {
    let user = create_resource(|| (), |_| async move { me().await });

    view! {
        <div class="grow">
            <div class="w-full md:max-w-md mx-auto space-y-2 py-6">
                <div class="hero">
                    <div class="hero-content text-center">
                        <div class="w-full mx-auto space-y-6">
                            <Suspense fallback=move || {
                                view! { <p>"loading"</p> }
                            }>
                                {move || {
                                    user.get()
                                        .map(|me| match me {
                                            Ok(Some(me)) => {
                                                let me = me.clone();
                                                view! {
                                                    <div class="avatar">
                                                        <div class="w-32 rounded-full">
                                                            <img src=&me.images[0].url/>
                                                        </div>
                                                    </div>
                                                    <h1 class="text-4xl font-bold">{me.display_name}</h1>
                                                    <div class="w-full flow-root space-x-4">
                                                        <button
                                                            class="float-left ml-2 btn btn-sm"
                                                            on:click=move |_| {
                                                                spawn_local(async {
                                                                    logout().await.expect("log out");
                                                                    let navigate = leptos_router::use_navigate();
                                                                    navigate("/", Default::default());
                                                                });
                                                            }
                                                        >

                                                            "Log Out"
                                                        </button>
                                                        <p class="float-right">
                                                            <A class="underline italic" href="/">
                                                                "Main Page"
                                                            </A>
                                                        </p>
                                                    </div>
                                                }
                                                    .into_view()
                                            }
                                            Ok(None) => {
                                                view! {
                                                    <h1 class="text-4xl font-bold">Not Logged In</h1>
                                                    <div class="w-full flex items-center">
                                                        <SpotifyButton/>
                                                        <p class="grow text-right">
                                                            <A class="underline italic" href="/">
                                                                "Main Page"
                                                            </A>
                                                        </p>
                                                    </div>
                                                }
                                                    .into_view()
                                            }
                                            Err(err) => {
                                                view! { <p>"Error " {err.to_string()}</p> }.into_view()
                                            }
                                        })
                                }}

                            </Suspense>
                        </div>
                    </div>
                </div>
                <Suspense fallback=move || {
                    view! { <p>"loading"</p> }
                }>
                    {move || match user.get() {
                        Some(Ok(Some(_))) => view! { <TopArtists/> }.into_view(),
                        _ => view! {}.into_view(),
                    }}

                </Suspense>
            </div>
        </div>
    }
}

#[server(GetMe)]
async fn me() -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    return Ok(use_context::<crate::auth::AuthSession>()
        .expect("provide auth session")
        .user);

    #[cfg(not(feature = "ssr"))]
    return Ok(None);
}

#[component]
fn TopArtists() -> impl IntoView {
    let (range, set_range) = create_signal(TimeRange::MediumTerm);
    let select = create_node_ref::<Select>();

    let update_range = move |_| {
        set_range.set(
            match select.get().expect("no select ref").value().as_str() {
                "Short Term" => TimeRange::ShortTerm,
                "Medium Term" => TimeRange::MediumTerm,
                _ => TimeRange::LongTerm,
            },
        )
    };

    let top_artists = create_resource(
        move || range.get(),
        |range| async move { get_top_artists(Some(range)).await },
    );

    view! {
        <h2 class="text-center text-2xl font-semibold">"Top Artists:"</h2>
        <select ref=select class="select max-w-xs" on:change=update_range>
            <option selected=move || range.get() == TimeRange::ShortTerm>"Short Term"</option>
            <option selected=move || range.get() == TimeRange::MediumTerm>"Medium Term"</option>
            <option selected=move || range.get() == TimeRange::LongTerm>"Long Term"</option>
        </select>
        <table class="table w-full">
            <thead>
                <tr>
                    <th>"Name"</th>
                    <th>"Popularity"</th>
                    <th>"Genres"</th>
                </tr>
            </thead>
            <tbody>
                <Suspense fallback=|| {
                    view! {
                        <tr>
                            <td>"Loading..."</td>
                        </tr>
                    }
                }>
                    {move || {
                        top_artists
                            .get()
                            .map(|top_artists| {
                                view! {
                                    <For
                                        each=move || {
                                            top_artists
                                                .as_ref()
                                                .ok()
                                                .cloned()
                                                .flatten()
                                                .unwrap_or_default()
                                        }

                                        key=|artist| artist.id.clone()
                                        children=move |artist| {
                                            view! {
                                                <tr>
                                                    <td class="flex space-x-3 items-center">
                                                        <a
                                                            class="avatar"
                                                            href=artist.external_urls.get("spotify").unwrap()
                                                        >
                                                            <div class="w-12 rounded-xl">
                                                                <img src=artist.images[0].url.clone()/>
                                                            </div>
                                                        </a>
                                                        <span>{artist.name}</span>
                                                    </td>
                                                    <td>{artist.popularity.to_string()}</td>
                                                    <td>{artist.genres.len()}</td>
                                                </tr>
                                            }
                                        }
                                    />
                                }
                            })
                    }}

                </Suspense>
            </tbody>
        </table>
    }
}

#[server(GetTopArtists)]
async fn get_top_artists(
    range: Option<TimeRange>,
) -> Result<Option<Vec<FullArtist>>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use futures::stream::TryStreamExt;
        use futures_util::pin_mut;
        use rspotify::clients::OAuthClient;

        let session = use_context::<crate::auth::AuthSession>().expect("provide auth session");

        let Some(user) = session.user else {
            return Ok(None);
        };

        let client = session.backend.user_client(user).await;

        let mut artists = Vec::new();
        let stream = client.current_user_top_artists(range);

        pin_mut!(stream);
        while let Some(artist) = stream.try_next().await.unwrap() {
            artists.push(artist);
        }

        return Ok((!artists.is_empty()).then_some(artists));
    }
}

#[server(Logout)]
async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use_context::<crate::auth::AuthSession>()
            .expect("provide auth session")
            .logout()
            .expect("log out");
    }

    Ok(())
}

