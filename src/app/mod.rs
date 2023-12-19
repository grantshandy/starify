use leptos::{html::Select, *};
use leptos_meta::*;
use leptos_router::*;
use rspotify::model::{FullArtist, PrivateUser, TimeRange};

mod login;

use crate::errors::{AppError, ErrorTemplate};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/musiscope.css" />
        <Router
            fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors />
            }.into_view()
        }>
            <div data-theme="light" class="min-h-screen flex flex-col">
                <Routes>
                    <Route
                        path="/"
                        view=login::LoginPage
                        ssr=SsrMode::Async
                    />
                    <Route path="/about" view=AboutPage />
                    <Route path="/me" view=Me ssr=SsrMode::Async />
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
    view! {
        <div class="max-w-md mx-auto space-y-2 py-6">
            <div class="hero">
                <div class="hero-content text-center">
                    <Await
                        future=me
                        let:res
                    >
                        {match res.as_ref() {
                            Ok(Some(me)) => {
                                let me = me.clone();

                                view! {
                                    <div class="max-w-md mx-auto">
                                        <div class="avatar">
                                            <div class="w-32 rounded-full">
                                                <img src={&me.images.unwrap()[0].url}></img>
                                            </div>
                                        </div>
                                        <h1 class="text-4xl font-bold">{me.display_name.unwrap_or("no display name".to_string())}</h1>
                                    </div>
                                }.into_view()
                            },
                            Ok(None) => view! { <p>"Not Logged In"</p> }.into_view(),
                            Err(err) => view! { <p>"Error " {err.to_string()}</p> }.into_view()
                        }}
                    </Await>
                </div>
            </div>
            <h2 class="text-center text-2xl font-semibold">"Top Artists:"</h2>
            <TopArtists />
        </div>
    }
}

#[server(GetMe)]
async fn me() -> Result<Option<PrivateUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    return Ok(use_context::<crate::auth::AuthSession>()
        .expect("provide auth session")
        .user
        .map(|user| user.me));

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
        <div class="w-full flow-root">
            <select
                ref=select
                class="float-left select select-primary max-w-xs"
                on:change=update_range
            >
                <option selected=move || range.get() == TimeRange::ShortTerm >"Short Term"</option>
                <option selected=move || range.get() == TimeRange::MediumTerm >"Medium Term"</option>
                <option selected=move || range.get() == TimeRange::LongTerm >"Long Term"</option>
            </select>
            <p><A href="/" class="float-right underline italic">"Back to Home"</A></p>
        </div>
        <Transition fallback=|| "Loading..." >
            <table class="table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Popularity"</th>
                        <th>"Link"</th>
                    </tr>
                </thead>
                <tbody>
                {move || top_artists
                    .get()
                    .map(|top_artists| view! {
                        <For
                            each=move || top_artists.as_ref().ok().cloned().flatten().unwrap_or_default()
                            key=|artist| artist.id.clone()
                            children=move |artist| view! {
                                <tr>
                                    <td class="flex space-x-3 items-center">
                                        <div class="avatar">
                                            <div class="w-12 rounded-xl">
                                                <img src={artist.images[0].url.clone()} />
                                            </div>
                                        </div>
                                        <span>{artist.name}</span>
                                    </td>
                                    <td>{artist.popularity.to_string()}</td>
                                    <td><a href={artist.external_urls.get("spotify").unwrap()}>"Link"</a></td>
                                </tr>
                            }
                        />
                })}
                </tbody>
            </table>
        </Transition>
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

        let Some(client) = use_context::<crate::auth::AuthSession>()
            .expect("provide auth session")
            .user
            .map(|user| user.client)
        else {
            return Ok(None);
        };

        let mut artists = Vec::new();
        let stream = client.current_user_top_artists(range);

        pin_mut!(stream);
        while let Some(artist) = stream.try_next().await.unwrap() {
            artists.push(artist);
        }

        return Ok((!artists.is_empty()).then_some(artists));
    }
}
