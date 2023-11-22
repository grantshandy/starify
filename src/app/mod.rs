use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/musiscope.css" />
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-light">"Home page!"</h1>
    }
}

