use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod login;

use crate::{
    errors::{AppError, ErrorTemplate},
    CALLBACK_ENDPOINT,
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
