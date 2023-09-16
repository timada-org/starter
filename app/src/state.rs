use axum::{
    async_trait,
    extract::FromRequestParts,
    response::{Html, IntoResponse},
    Extension, RequestPartsExt,
};
use evento::PgProducer;
use http::{request::Parts, StatusCode};
use i18n_embed::{fluent::FluentLanguageLoader, LanguageLoader};
use leptos::*;
use serde::Deserialize;
use sqlx::PgPool;
use starter_core::axum_extra::UserLanguage;
use std::sync::Arc;
use timada_starter_feed::{FeedCommand, FeedQuery};
use twa_jwks::axum::JwtPayload;
use ulid::Ulid;
use unic_langid::LanguageIdentifier;

use crate::{
    config::AppConfig,
    i18n::{LANGUAGES, LANGUAGE_LOADER},
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub evento: PgProducer,
    pub db: PgPool,
}

#[derive(Clone)]
pub struct AppContext {
    pub config: AppConfig,
    pub lang: String,
    pub fl_loader: Arc<FluentLanguageLoader>,
    pub jwt_claims: JwtClaims,
    pub feed_cmd: FeedCommand,
    pub feed_query: FeedQuery,
}

impl AppContext {
    pub fn html<F, N>(&self, f: F) -> impl IntoResponse
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
    {
        let ctx = self.clone();
        let html = ssr::render_to_string(move || {
            provide_context(ctx);

            f()
        });

        (StatusCode::OK, Html(html.to_string()))
    }

    pub fn internal_server_error<F, N>(&self, f: F) -> impl IntoResponse
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
    {
        (StatusCode::INTERNAL_SERVER_ERROR, self.html(f))
    }

    pub fn bad_request<F, N>(&self, f: F) -> impl IntoResponse
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
    {
        (StatusCode::BAD_REQUEST, self.html(f))
    }

    pub fn not_found<F, N>(&self, f: F) -> impl IntoResponse
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
    {
        (StatusCode::NOT_FOUND, self.html(f))
    }

    pub fn create_url(&self, uri: impl Into<String>) -> String {
        let uri = uri.into();
        self.config
            .base_url
            .as_ref()
            .map(|base_url| format!("{base_url}{}", uri))
            .unwrap_or(uri)
    }

    pub fn create_static_url(&self, uri: impl Into<String>) -> String {
        self.create_url(format!("/static/{}", uri.into()))
    }

    fn fl_loader(user_lang: UserLanguage) -> FluentLanguageLoader {
        let langs = user_lang
            .preferred_languages()
            .iter()
            .map(|lang| lang.parse().unwrap())
            .collect::<Vec<LanguageIdentifier>>();

        LANGUAGE_LOADER.select_languages(&langs)
    }

    fn lang(loader: &FluentLanguageLoader) -> String {
        loader
            .current_languages()
            .iter()
            .find_map(|language| {
                if LANGUAGES.contains(&language) {
                    Some(language.to_string())
                } else {
                    None
                }
            })
            .unwrap_or(loader.fallback_language().to_string())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AppContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Html<&'static str>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let JwtPayload(jwt_claims) =
            JwtPayload::<JwtClaims>::from_request_parts(parts, state).await?;

        let Ok(user_lang) = UserLanguage::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, Html("Bad Request")));
        };

        let fl_loader = Self::fl_loader(user_lang);
        let lang = Self::lang(&fl_loader);

        let Extension(state) = parts
            .extract::<Extension<AppState>>()
            .await
            .expect("AppState not configured correctly");

        Ok(Self {
            config: state.config,
            fl_loader: Arc::new(fl_loader),
            lang,
            feed_cmd: FeedCommand {
                producer: state.evento,
                user_id: jwt_claims.sub.to_owned(),
                request_id: Ulid::new().to_string(),
            },
            feed_query: FeedQuery {
                user_id: jwt_claims.sub.to_owned(),
                db: state.db,
            },
            jwt_claims,
        })
    }
}

pub fn use_app() -> AppContext {
    use_context().expect("AppContext not configured correctly")
}

#[derive(Deserialize, Debug, Clone)]
pub struct JwtClaims {
    pub sub: String,
}
