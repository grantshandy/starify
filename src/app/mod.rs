use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod login;

use crate::{
    User,
    errors::{AppError, ErrorTemplate},
};

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
                    <Route path="/me" view=Me />
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
        <div class="grow">
            <h1>"You:"</h1>
            <Await
                future=|| me()
                let:res
            >
                {match res.as_ref() {
                    Ok(Some(me)) => {
                        let me = me.0.clone();

                        view! {
                            <img src={&me.images.unwrap()[0].url}></img>
                            <p>"Name: " {me.display_name.unwrap_or("no display name".to_string())}</p>
                        }.into_view()
                    },
                    Ok(None) => view! { <p>"Not Logged In"</p> }.into_view(),
                    Err(err) => view! { <p>"Error " {err.to_string()}</p> }.into_view()
                }}
            </Await>
            <A href="/">"Back to Home"</A>
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