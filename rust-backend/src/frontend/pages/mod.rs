use axum::{Router, routing::get};

use crate::types::AppState;

mod account;
mod home;
mod media;
mod shared;
mod view;

use account::{admin_page, login_page, premium_settings_page, settings_page};
use home::index_page;
use media::{detail_page, player_page, premium_page};

pub(super) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/admin", get(admin_page))
        .route("/detail", get(detail_page))
        .route("/player", get(player_page))
        .route("/premium", get(premium_page))
        .route("/login", get(login_page))
        .route("/settings", get(settings_page))
        .route("/premium/settings", get(premium_settings_page))
        .with_state(state)
}
