use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::json;

use crate::types::AppState;

use super::super::super::render::{html_response, render_shell};
use super::super::shared::{load_user_json, require_page_auth, require_premium_access};
use super::super::view::render_premium_settings_body;

pub(crate) async fn premium_settings_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/premium/settings").await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_premium_access(&auth_user) {
        return response;
    }

    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let premium_sources_value = settings_value
        .get("premiumSources")
        .cloned()
        .unwrap_or_else(|| json!([]));

    let body = render_premium_settings_body(&auth_user, &premium_sources_value);

    html_response(render_shell(
        "Premium 设置",
        Some(&auth_user),
        "/premium/settings",
        &body,
    ))
}

#[cfg(test)]
mod tests {
    use super::render_premium_settings_body;
    use crate::auth::AuthUser;
    use serde_json::json;

    #[test]
    fn render_premium_settings_body_shows_editor_and_status() {
        let user = AuthUser {
            id: 1,
            username: "vip".to_string(),
            is_admin: false,
            disable_premium: false,
        };
        let html = render_premium_settings_body(&user, &json!([{"id":"px","enabled":true}]));

        assert!(html.contains("Premium 设置"));
        assert!(html.contains("premium-json"));
        assert!(html.contains("premium-source-list"));
        assert!(html.contains("premium-source-search"));
        assert!(html.contains("toggle-premium-source-limit"));
        assert!(html.contains("premium-enable-all"));
        assert!(html.contains("premium-source-form"));
        assert!(html.contains("premium-source-submit"));
        assert!(html.contains("新增或编辑高级源"));
        assert!(html.contains("inset-card"));
        assert!(html.contains("restore-default-premium-sources"));
        assert!(html.contains("format-premium"));
        assert!(html.contains("save-premium"));
        assert!(html.contains("collapsible-card"));
        assert!(html.contains("collapsible-summary"));
        assert!(html.contains("code-input-large"));
        assert!(html.contains("同步"));
    }
}
