use leptos_meta::*;
use leptos_router::*;
use leptos::*;

mod login;

use login::SpotifyButton;

use crate::{errors::{AppError, ErrorTemplate}, client::PackedClient};

#[server]
async fn get_client() -> Result<Option<PackedClient>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user = use_context::<crate::auth::AuthSession>()
            .expect("no packed client found")
            .user;

        Ok(match user {
            Some(client) => Some(client.packed().await),
            None => None,
        })
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/starify.css"/>
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <div data-theme="light" class="min-h-screen flex flex-col">
                <main class="grow flex">
                    <Routes>
                        <Route path="/" view=LoginPage ssr=SsrMode::Async/>
                        <Route path="/about" view=AboutPage/>
                        <Route path="/me" view=Me />
                    </Routes>
                </main>
                <footer class="footer footer-center p-4 bg-base-400 text-base-content">
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
                <div class="space-y-6 text-center">
                    <h1 class="text-5xl font-bold">"starify"</h1>
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
        <div>
            <h1>"About!"</h1>
            <A href="/">"Back to Home"</A>
        </div>
    }
}

#[component]
pub fn Me() -> impl IntoView {
    view! {
        <div>
            <p>"Your Info"</p>
        </div>
    }
}

