mod auth;
mod db;
mod frontend;
mod routes;
mod types;

use reqwest::Client;
use sqlx::mysql::MySqlPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::types::{AppConfig, AppState};

fn build_mysql_url() -> String {
    let host = std::env::var("MYSQL_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    let user = std::env::var("MYSQL_USER").unwrap_or_else(|_| "kvideo".to_string());
    let password = std::env::var("MYSQL_PASSWORD").unwrap_or_else(|_| "kvideo123".to_string());
    let database = std::env::var("MYSQL_DATABASE").unwrap_or_else(|_| "kvideo".to_string());
    format!("mysql://{user}:{password}@{host}:{port}/{database}")
}

fn load_ad_keywords_from_env() -> Vec<String> {
    let mut keywords = Vec::new();

    if let Ok(file_path) = std::env::var("AD_KEYWORDS_FILE") {
        let path = std::path::PathBuf::from(&file_path);
        let resolved = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(path)
        };

        if let Ok(content) = std::fs::read_to_string(&resolved) {
            keywords.extend(
                content
                    .split(['\n', ','])
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(ToOwned::to_owned),
            );
        }
    }

    if keywords.is_empty() {
        if let Ok(raw) =
            std::env::var("AD_KEYWORDS").or_else(|_| std::env::var("NEXT_PUBLIC_AD_KEYWORDS"))
        {
            keywords.extend(
                raw.split(['\n', ','])
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(ToOwned::to_owned),
            );
        }
    }

    let mut deduped = Vec::new();
    for keyword in keywords {
        if !deduped.iter().any(|item| item == &keyword) {
            deduped.push(keyword);
        }
    }
    deduped
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "kvideo_rust_backend=info,tower_http=info".to_string()),
        )
        .init();

    let bind_host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let auth_secret = std::env::var("AUTH_SECRET")
        .unwrap_or_else(|_| "rvideo-default-secret-change-in-production".to_string());
    let access_password = std::env::var("ACCESS_PASSWORD")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let persist_password = std::env::var("PERSIST_PASSWORD")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let subscription_sources = std::env::var("SUBSCRIPTION_SOURCES")
        .or_else(|_| std::env::var("NEXT_PUBLIC_SUBSCRIPTION_SOURCES"))
        .unwrap_or_default();
    let node_env = std::env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string());

    let mysql_url = build_mysql_url();
    let db = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&mysql_url)
        .await?;

    db::init_database(&db).await?;

    let http = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    let state = AppState {
        db,
        http,
        config: AppConfig {
            auth_secret,
            subscription_sources,
            ad_keywords: load_ad_keywords_from_env(),
            access_password,
            persist_password,
            is_production: node_env == "production",
        },
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::router(state.clone())
        .merge(frontend::router(state))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("{bind_host}:{bind_port}")).await?;
    info!(
        "rvideo rust backend listening on http://{}",
        listener.local_addr()?
    );

    axum::serve(listener, app).await?;
    Ok(())
}
