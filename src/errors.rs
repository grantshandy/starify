use cfg_if::cfg_if;
use http::status::StatusCode;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use leptos::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,

    #[error("Failure to authenticate: {0}.")]
    Authentication(String)
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Authentication(_) => StatusCode::BAD_REQUEST,
        }
    }
}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => create_rw_signal(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    cfg_if! { if #[cfg(feature="ssr")] {
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }}

    view! {
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md space-y-6">
                    <h1 class="text-5xl font-bold">"Error: " { errors[0].status_code().to_string() }</h1>
                    <p class="bg-base-300 rounded-md p-2">
                        <code>{ errors[0].to_string() }</code>
                    </p>
                    <For
                        each= move || {errors.clone().into_iter().enumerate().skip(1)}
                        key=|(index, _error)| *index
                        children=move |error| view! {
                            <div class="alert alert-error">
                                <span>"Error " {error.1.status_code().to_string()}</span>
                            </div>
                        }
                    />
                    <p>
                        <A href="/">"Return to Main Page?"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}
