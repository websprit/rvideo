use crate::types::AppState;
use axum::{extract::State, http::HeaderMap, response::Response};

use super::super::super::render::{html_response, render_shell};
use super::super::shared::{load_user_json, require_page_auth};
use super::super::view::render_settings_body;

pub(crate) async fn settings_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/settings").await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let subscription_sources = if state.config.subscription_sources.trim().is_empty() {
        "未配置系统订阅源".to_string()
    } else {
        state.config.subscription_sources.clone()
    };

    let body = render_settings_body(
        &auth_user,
        &subscription_sources,
        &settings_value,
        state.config.access_password.is_some(),
        state.config.persist_password,
    );

    html_response(render_shell("设置", Some(&auth_user), "/settings", &body))
}

#[cfg(test)]
mod tests {
    use super::render_settings_body;
    use crate::auth::AuthUser;
    use serde_json::json;

    #[test]
    fn render_settings_body_shows_user_role_and_subscription_text() {
        let user = AuthUser {
            id: 1,
            username: "admin".to_string(),
            is_admin: true,
            disable_premium: false,
        };
        let html = render_settings_body(
            &user,
            "https://sub.example.com/list.txt",
            &json!({"sources":[{"id":"a"}]}),
            true,
            true,
        );

        assert!(html.contains("用户：admin"));
        assert!(html.contains("角色：管理员"));
        assert!(html.contains("Premium：已启用"));
        assert!(html.contains("设置"));
        assert!(html.contains("快捷入口"));
        assert!(html.contains(
            r#"<pre id="subscription-sources" class="code-block">https://sub.example.com/list.txt</pre>"#
        ));
        assert!(html.contains("href=\"/premium/settings\""));
        assert!(html.contains("href=\"/admin\""));
        assert!(html.contains("refresh-settings"));
        assert!(html.contains("quick-search-display-mode"));
        assert!(html.contains("quick-search-history"));
        assert!(html.contains("quick-watch-history"));
        assert!(html.contains("quick-auto-skip-intro"));
        assert!(html.contains("quick-skip-outro-seconds"));
        assert!(html.contains("quick-ad-filter-mode"));
        assert!(html.contains("quick-ad-keywords"));
        assert!(html.contains("settings-source-list"));
        assert!(html.contains("settings-source-search"));
        assert!(html.contains("toggle-settings-source-limit"));
        assert!(html.contains("settings-source-form"));
        assert!(html.contains("settings-source-submit"));
        assert!(html.contains("新增或编辑线路"));
        assert!(html.contains("inset-card"));
        assert!(html.contains("restore-default-sources"));
        assert!(html.contains("subscription-summary"));
        assert!(html.contains("环境密码：已启用"));
        assert!(html.contains("持久解锁：启用"));
        assert!(html.contains("access-control-enabled"));
        assert!(html.contains("access-password-form"));
        assert!(html.contains("access-password-list"));
        assert!(html.contains("settings-import-payload"));
        assert!(html.contains("collapsible-card"));
        assert!(html.contains("collapsible-summary"));
        assert!(html.contains("code-input-medium"));
        assert!(html.contains("import-subscriptions"));
        assert!(html.contains("sync-system-subscriptions"));
        assert!(html.contains("export-settings"));
        assert!(html.contains("export-backup"));
        assert!(html.contains("import-backup-file-trigger"));
        assert!(html.contains("import-backup-file"));
        assert!(html.contains("code-input-large"));
        assert!(html.contains("backup-include-search-history"));
        assert!(html.contains("backup-include-watch-history"));
        assert!(html.contains("backup-include-premium-data"));
        assert!(html.contains("function getBackupOptions()"));
        assert!(html.contains("async function exportFullBackup()"));
        assert!(html.contains("async function importBackupPayload(rawValue)"));
        assert!(html.contains("format: 'kvideo-rust-backup'"));
        assert!(html.contains("clear-synced-data"));
        assert!(html.contains("reset-all-data"));
        assert!(html.contains("数据管理"));
        assert!(html.contains("logout-button"));
    }

    #[test]
    fn render_settings_body_hides_privileged_links_for_normal_user() {
        let user = AuthUser {
            id: 2,
            username: "demo".to_string(),
            is_admin: false,
            disable_premium: true,
        };
        let html = render_settings_body(&user, "未配置系统订阅源", &json!({}), false, false);

        assert!(html.contains("用户：demo"));
        assert!(html.contains("角色：普通用户"));
        assert!(html.contains("Premium：已禁用"));
        assert!(html.contains("设置"));
        assert!(!html.contains("href=\"/premium/settings\""));
        assert!(!html.contains("href=\"/admin\""));
    }
}
