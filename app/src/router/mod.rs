mod feed;

use crate::{config::AppConfig, context::AppContext, Page};
use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use leptos::*;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
}

impl AppState {
    pub fn to_context(&self) -> AppContext {
        AppContext {
            config: self.config.clone(),
        }
    }

    pub fn render_to_string<F, N>(&self, f: F) -> impl IntoResponse
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
    {
        let ctx = self.to_context();

        let html = ssr::render_to_string(move || {
            provide_context(ctx);

            f()
        });

        Html(html.to_string())
    }
}

async fn root(State(app): State<AppState>) -> impl IntoResponse {
    app.render_to_string(|| {
        view! { <Page>"Hello"</Page> }
    })
}

pub fn create_router() -> Router<AppState, Body> {
    Router::new()
        .route("/", get(root))
        .nest("/feed", feed::create_router())
}