use axum::response::{Html, IntoResponse};
use evento_store::{PgEngine, EventStore};
use i18n_embed::fluent::FluentLanguageLoader;
use leptos::*;
use unic_langid::LanguageIdentifier;
use serde::Deserialize;

use crate::{config::AppConfig, context::AppContext};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub store: EventStore<PgEngine>,
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

    pub fn language_loader(&self, langs: &[String]) -> FluentLanguageLoader {
        let langs = langs
            .iter()
            .map(|lang| lang.parse().unwrap())
            .collect::<Vec<LanguageIdentifier>>();

        crate::i18n::LANGUAGE_LOADER.select_languages(&langs)
    }
}

#[derive(Deserialize)]
pub(crate) struct JwtClaims {
    pub _sub: String,
}
