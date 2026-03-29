use crate::types::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};

use super::super::super::render::{find_source_config, html_response, render_shell};
use super::super::shared::{
    DetailPageQuery, favorites_key, load_user_json, parse_media_request, require_page_auth,
};
use super::super::view::render_detail_body;

pub(crate) async fn detail_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DetailPageQuery>,
) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/detail").await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let media = match parse_media_request(query) {
        Ok(media) => media,
        Err(response) => return response,
    };
    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let source_config = find_source_config(&settings_value, &media.source);
    let favorites_data_key = favorites_key(media.is_premium);
    let favorites_value = load_user_json(&state, auth_user.id, favorites_data_key).await;

    let body = render_detail_body(
        &auth_user,
        &media,
        source_config,
        favorites_data_key,
        &favorites_value,
    );

    html_response(render_shell("详情", Some(&auth_user), "/", &body))
}

#[cfg(test)]
mod tests {
    use super::render_detail_body;
    use crate::auth::AuthUser;
    use crate::frontend::pages::shared::MediaRequest;
    use serde_json::json;

    #[test]
    fn render_detail_body_includes_player_links_and_state_payload() {
        let user = AuthUser {
            id: 1,
            username: "tester".to_string(),
            is_admin: false,
            disable_premium: false,
        };
        let media = MediaRequest {
            video_id: "123".to_string(),
            source: "source-a".to_string(),
            title: "测试详情".to_string(),
            is_premium: true,
            episode: 0,
            grouped_sources: json!([
                {"id":"123","source":"source-a","sourceName":"线路 A"},
                {"id":"456","source":"source-b","sourceName":"线路 B"}
            ]),
        };

        let html = render_detail_body(
            &user,
            &media,
            Some(json!({"id":"source-a"})),
            "premium-favorites",
            &json!([{"videoId":"123"}]),
        );

        assert!(html.contains("打开播放器"));
        assert!(html.contains("详情"));
        assert!(html.contains("模式：Premium"));
        assert!(html.contains("premium-favorites"));
        assert!(html.contains("测试详情"));
        assert!(html.contains("\"groupedSources\":["));
        assert!(html.contains("params.set('groupedSources'"));
    }
}
