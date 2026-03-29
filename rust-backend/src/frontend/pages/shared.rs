use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde_json::{Value as JsonValue, json};

use crate::{
    auth::{self, AuthUser},
    db,
    types::AppState,
};

use super::super::render::safe_next_path;

#[derive(Debug, Default, Deserialize)]
pub(super) struct NextQuery {
    pub(super) next: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct SearchPageQuery {
    pub(super) q: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct DetailPageQuery {
    pub(super) id: Option<String>,
    pub(super) source: Option<String>,
    pub(super) title: Option<String>,
    pub(super) premium: Option<String>,
    pub(super) episode: Option<String>,
    #[serde(rename = "groupedSources")]
    pub(super) grouped_sources: Option<String>,
}

pub(super) struct MediaRequest {
    pub(super) video_id: String,
    pub(super) source: String,
    pub(super) title: String,
    pub(super) is_premium: bool,
    pub(super) episode: usize,
    pub(super) grouped_sources: JsonValue,
}

pub(super) async fn load_user_json(state: &AppState, user_id: i64, key: &str) -> JsonValue {
    match db::get_user_data(&state.db, user_id, key).await {
        Ok(raw) => serde_json::from_str::<JsonValue>(&raw).unwrap_or_else(|_| json!({})),
        Err(_) => json!({}),
    }
}

pub(super) fn redirect_to_login(next_path: &str) -> Response {
    let next = safe_next_path(Some(next_path), "/settings");
    Redirect::to(&format!("/login?next={}", urlencoding::encode(&next))).into_response()
}

pub(super) async fn require_page_auth(
    headers: &HeaderMap,
    state: &AppState,
    next_path: &str,
) -> Result<AuthUser, Response> {
    match auth::get_auth_user_optional(headers, state).await {
        Ok(Some(user)) => Ok(user),
        _ => Err(redirect_to_login(next_path)),
    }
}

pub(super) fn require_admin_user(user: &AuthUser) -> Result<(), Response> {
    if user.is_admin {
        Ok(())
    } else {
        Err(Redirect::to("/").into_response())
    }
}

pub(super) fn require_premium_access(user: &AuthUser) -> Result<(), Response> {
    if user.disable_premium {
        Err(Redirect::to("/").into_response())
    } else {
        Ok(())
    }
}

pub(super) fn parse_media_request(query: DetailPageQuery) -> Result<MediaRequest, Response> {
    let Some(video_id) = query
        .id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Err(Redirect::to("/").into_response());
    };
    let Some(source) = query
        .source
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Err(Redirect::to("/").into_response());
    };

    Ok(MediaRequest {
        video_id,
        source,
        title: query.title.unwrap_or_else(|| "未知视频".to_string()),
        is_premium: query.premium.as_deref() == Some("1"),
        episode: query
            .episode
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .unwrap_or(0),
        grouped_sources: query
            .grouped_sources
            .as_deref()
            .and_then(|value| serde_json::from_str::<JsonValue>(value).ok())
            .filter(|value| value.is_array())
            .unwrap_or_else(|| json!([])),
    })
}

pub(super) fn favorites_key(is_premium: bool) -> &'static str {
    if is_premium {
        "premium-favorites"
    } else {
        "favorites"
    }
}

pub(super) fn history_key(is_premium: bool) -> &'static str {
    if is_premium {
        "premium-history"
    } else {
        "history"
    }
}
