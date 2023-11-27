use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod login;

use crate::error_template::{AppError, ErrorTemplate};

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
            <div class="space-y-3 flex-col">
                <main class="p-2 bg-slate-700 rounded-md">
                    <Routes>
                        <Route
                            path=""
                            view=login::LoginPage
                            ssr=SsrMode::Async
                        />
                        <Route
                            path="/callback"
                            view=login::LoginCallback
                            ssr=SsrMode::Async
                        />
                        <Route path="/about" view=AboutPage />
                    </Routes>
                </main>
                <footer class="italic mx-auto">"© 2023 Grant Handy"</footer>
            </div>
        </Router>
    }
}

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <h1>"About!"</h1>
        <A href="/">"Back to Home"</A>
    }
}
