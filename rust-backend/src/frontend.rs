use std::path::PathBuf;

use axum::{Router, routing::get_service};
use tower_http::services::ServeFile;

use crate::types::AppState;

mod assets;
mod pages;
mod render;
mod scripts;

pub fn router(state: AppState) -> Router {
    pages::router(state)
        .route_service(
            "/sw.js",
            get_service(ServeFile::new(public_asset_path("sw.js"))),
        )
        .route_service(
            "/manifest.json",
            get_service(ServeFile::new(public_asset_path("manifest.json"))),
        )
        .route_service(
            "/icon.png",
            get_service(ServeFile::new(public_asset_path("icon.png"))),
        )
        .route_service(
            "/favicon.ico",
            get_service(ServeFile::new(public_asset_path("icon.png"))),
        )
        .route_service(
            "/placeholder-poster.svg",
            get_service(ServeFile::new(public_asset_path("placeholder-poster.svg"))),
        )
}

fn public_asset_path(file_name: &str) -> PathBuf {
    let candidates = [
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("public")
            .join(file_name),
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("..")
            .join("public")
            .join(file_name),
    ];

    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("public")
                .join(file_name)
        })
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use reqwest::Client;
    use serde_json::json;
    use sqlx::mysql::MySqlPoolOptions;
    use tower::util::ServiceExt;

    use super::render::{
        build_rust_detail_url, build_rust_player_url, count_enabled_sources, escape_html,
        escape_script_json, find_source_config, json_array_len, preview_card,
        render_home_library_entries, render_saved_items_list, render_shell, safe_next_path,
    };
    use crate::auth::AuthUser;
    use crate::types::{AppConfig, AppState};

    fn test_state() -> AppState {
        let db = MySqlPoolOptions::new()
            .connect_lazy("mysql://kvideo:kvideo123@localhost:3306/kvideo")
            .expect("lazy mysql pool should initialize");
        let http = Client::builder().build().expect("http client should build");
        AppState {
            db,
            http,
            config: AppConfig {
                auth_secret: "test-secret".to_string(),
                subscription_sources: String::new(),
                ad_keywords: vec!["env-ad".to_string()],
                access_password: None,
                persist_password: true,
                is_production: false,
            },
        }
    }

    async fn get(path: &str) -> axum::response::Response {
        super::router(test_state())
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond")
    }

    async fn body_text(response: axum::response::Response) -> String {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        String::from_utf8(body.to_vec()).expect("body should be valid utf-8")
    }

    fn assert_login_redirect(response: &axum::response::Response, expected_location: &str) {
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some(expected_location)
        );
    }

    #[test]
    fn safe_next_path_accepts_internal_paths() {
        assert_eq!(safe_next_path(Some("/settings"), "/"), "/settings");
    }

    #[test]
    fn safe_next_path_rejects_external_paths() {
        assert_eq!(safe_next_path(Some("https://evil.test"), "/"), "/");
        assert_eq!(safe_next_path(Some("//evil.test"), "/"), "/");
    }

    #[test]
    fn build_rust_player_url_preserves_episode_and_premium_flag() {
        assert_eq!(
            build_rust_player_url("123", "source-a", "测试视频", None, Some(2), true),
            "/player?id=123&source=source-a&title=%E6%B5%8B%E8%AF%95%E8%A7%86%E9%A2%91&episode=2&premium=1"
        );
    }

    #[test]
    fn escape_html_escapes_special_chars() {
        assert_eq!(escape_html("<tag>\"'&"), "&lt;tag&gt;&quot;&#39;&amp;");
    }

    #[test]
    fn render_shell_marks_active_nav_and_shows_admin_link() {
        let user = AuthUser {
            id: 1,
            username: "admin".to_string(),
            is_admin: true,
            disable_premium: false,
        };

        let html = render_shell("管理后台", Some(&user), "/admin", "<p>body</p>");
        assert!(html.contains(r#"<a class="button active" href="/admin">管理后台</a>"#));
        assert!(html.contains(r#"<a class="button" href="/settings">设置</a>"#));
        assert!(html.contains(r#"<span class="chip shell-user">admin</span>"#));
        assert!(html.contains(r#"<a class="shell-brand" href="/">RVIDEO</a>"#));
        assert!(html.contains("shell-topbar"));
        assert!(html.contains("shell-nav-links"));
        assert!(html.contains(r#"id="bootstrap-payload""#));
        assert!(html.contains(r#"id="access-gate-overlay""#));
        assert!(html.contains(r#"id="access-gate-form""#));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#""authenticated":true"#));
        assert!(html.contains(r#""disablePremium":false"#));
        assert!(html.contains("window.__KVIDEO_BOOTSTRAP_PROMISE__"));
        assert!(html.contains("kvideo-history-store"));
        assert!(html.contains("window.__KVIDEO_STORAGE_SYNC_INSTALLED__"));
        assert!(html.contains("function mergeEnvSubscriptions(configData)"));
        assert!(html.contains("async function syncSubscriptionsFromSettings()"));
        assert!(html.contains("fetchSourcesFromSubscriptionUrl(subscription.url)"));
        assert!(html.contains("subscriptionSources"));
        assert!(html.contains("const ACCESS_GATE_SESSION_KEY = 'kvideo-access-unlocked'"));
        assert!(html.contains("function readBootstrapConfig()"));
        assert!(html.contains("function hasEnvAccessGate()"));
        assert!(html.contains("function isEnvAccessGateUnlocked()"));
        assert!(html.contains("function isAccessGateEnabled(settings)"));
        assert!(html.contains("function applyAccessGateState()"));
        assert!(html.contains("function trapAccessGateFocus(event)"));
        assert!(html.contains("/api/auth/access-unlock"));
        assert!(html.contains("hasEnvPassword"));
        assert!(html.contains("envPasswordUnlocked"));
        assert!(html.contains("persistPassword"));
        assert!(html.contains("kvideo:storage-updated"));
        assert!(html.contains("访问密码错误，请重试。"));
        assert!(html.contains("method: 'PUT'"));
        assert!(html.contains("cast_sender.js?loadCastFramework=1"));
        assert!(html.contains("navigator.serviceWorker.register('/sw.js')"));
        assert!(html.contains("scroll-pos:"));
        assert!(html.contains(r#"id="back-to-top""#));
        assert!(html.contains("backToTopButton.classList.toggle('visible'"));
        assert!(html.contains("manifest.json"));
        assert!(html.contains("mergeEnvAdKeywords"));
    }

    #[test]
    fn render_shell_hides_admin_link_for_normal_user() {
        let user = AuthUser {
            id: 2,
            username: "demo".to_string(),
            is_admin: false,
            disable_premium: false,
        };

        let html = render_shell("首页", Some(&user), "/", "<p>body</p>");
        assert!(html.contains(r#"<a class="button active" href="/">搜索</a>"#));
        assert!(!html.contains("管理后台"));
    }

    #[test]
    fn render_shell_hides_premium_nav_for_premium_disabled_user() {
        let user = AuthUser {
            id: 3,
            username: "demo".to_string(),
            is_admin: false,
            disable_premium: true,
        };

        let html = render_shell("设置", Some(&user), "/settings", "<p>body</p>");
        assert!(html.contains(r#"<a class="button active" href="/settings">设置</a>"#));
        assert!(!html.contains(r#"href="/premium""#));
        assert!(!html.contains(r#"href="/premium/settings""#));
    }

    #[test]
    fn render_shell_guest_shows_rust_login_entry() {
        let html = render_shell("登录", None, "/login", "<p>body</p>");
        assert!(!html.contains(r#"<a class="button active" href="/login">登录</a>"#));
        assert!(!html.contains("当前用户："));
        assert!(html.contains("shell-topbar-minimal"));
        assert!(html.contains("login-page-content"));
        assert!(html.contains(r#""authenticated":false"#));
        assert!(html.contains(r#"id="bootstrap-overlay""#));
        assert!(html.contains("输入访问密码"));
        assert!(html.contains("持久 Cookie"));
        assert!(html.contains("返回顶部"));
    }

    #[test]
    fn preview_card_escapes_title_and_description() {
        let html = preview_card("<标签>", "\"描述\" & 内容");
        assert!(html.contains("&lt;标签&gt;"));
        assert!(html.contains("&quot;描述&quot; &amp; 内容"));
    }

    #[test]
    fn render_saved_items_list_uses_rust_player_links() {
        let html = render_saved_items_list(
            &json!([
                {
                    "videoId": "123",
                    "title": "测试视频",
                    "source": "source-a",
                    "sourceName": "线路 A"
                }
            ]),
            "空",
            "/fallback",
        );

        assert!(html.contains(r#"href="/player?id=123&amp;source=source-a&amp;title=%E6%B5%8B%E8%AF%95%E8%A7%86%E9%A2%91""#));
        assert!(html.contains("测试视频"));
        assert!(html.contains("线路 A"));
    }

    #[test]
    fn render_saved_items_list_falls_back_when_item_is_incomplete() {
        let html = render_saved_items_list(
            &json!([
                {
                    "title": "无来源视频"
                }
            ]),
            "空",
            "/fallback",
        );

        assert!(html.contains(r#"href="/fallback""#));
        assert!(html.contains("无来源视频"));
    }

    #[test]
    fn render_saved_items_list_returns_empty_state_for_empty_array() {
        let html = render_saved_items_list(&json!([]), "暂无内容", "/fallback");
        assert!(html.contains("empty-state"));
        assert!(html.contains("暂无内容"));
    }

    #[test]
    fn render_saved_items_list_limits_entries_to_five_items() {
        let html = render_saved_items_list(
            &json!([
                {"videoId":"1","title":"1","source":"a"},
                {"videoId":"2","title":"2","source":"a"},
                {"videoId":"3","title":"3","source":"a"},
                {"videoId":"4","title":"4","source":"a"},
                {"videoId":"5","title":"5","source":"a"},
                {"videoId":"6","title":"6","source":"a"}
            ]),
            "空",
            "/fallback",
        );
        assert_eq!(html.matches(r#"class="saved-item""#).count(), 5);
        assert!(!html.contains(">6<"));
    }

    #[test]
    fn build_rust_detail_url_preserves_premium_flag() {
        assert_eq!(
            build_rust_detail_url("123", "source-a", "测试视频", None, true),
            "/detail?id=123&source=source-a&title=%E6%B5%8B%E8%AF%95%E8%A7%86%E9%A2%91&premium=1"
        );
    }

    #[test]
    fn build_rust_player_url_preserves_grouped_sources() {
        let grouped_sources = json!([
            {"id":"123","source":"source-a"},
            {"id":"456","source":"source-b"}
        ]);
        let url = build_rust_player_url(
            "123",
            "source-a",
            "测试视频",
            Some(&grouped_sources),
            Some(1),
            false,
        );
        assert!(url.contains("groupedSources="));
        assert!(url.contains("%5B%7B%22id%22%3A%22123%22%2C%22source%22%3A%22source-a%22%7D%2C%7B%22id%22%3A%22456%22%2C%22source%22%3A%22source-b%22%7D%5D"));
    }

    #[test]
    fn render_home_library_entries_include_actions_and_links() {
        let html = render_home_library_entries(
            &json!([
                {
                    "videoId": "123",
                    "title": "测试视频",
                    "source": "source-a",
                    "sourceName": "线路 A",
                    "premium": true
                }
            ]),
            "空",
        );

        assert!(html.contains("data-library-query=\"测试视频\""));
        assert!(html.contains(r#"href="/player?id=123&amp;source=source-a&amp;title=%E6%B5%8B%E8%AF%95%E8%A7%86%E9%A2%91&amp;premium=1""#));
        assert!(html.contains(r#"href="/detail?id=123&amp;source=source-a&amp;title=%E6%B5%8B%E8%AF%95%E8%A7%86%E9%A2%91&amp;premium=1""#));
        assert!(html.contains("Premium 内容"));
    }

    #[test]
    fn count_enabled_sources_only_counts_enabled_items_with_ids() {
        let count = count_enabled_sources(&json!({
            "sources": [
                { "id": "a", "enabled": true },
                { "id": "b", "enabled": false },
                { "id": "c" },
                { "enabled": true }
            ]
        }));

        assert_eq!(count, 2);
    }

    #[test]
    fn json_array_len_handles_non_arrays() {
        assert_eq!(json_array_len(&json!([1, 2, 3])), 3);
        assert_eq!(json_array_len(&json!({"a":1})), 0);
    }

    #[test]
    fn find_source_config_checks_sources_then_premium_sources() {
        let settings = json!({
            "sources": [
                {"id": "source-a", "name": "Source A"}
            ],
            "premiumSources": [
                {"id": "premium-a", "name": "Premium A"}
            ]
        });

        assert_eq!(
            find_source_config(&settings, "source-a").and_then(|value| value.get("name").cloned()),
            Some(json!("Source A"))
        );
        assert_eq!(
            find_source_config(&settings, "premium-a").and_then(|value| value.get("name").cloned()),
            Some(json!("Premium A"))
        );
        assert!(find_source_config(&settings, "missing").is_none());
    }

    #[test]
    fn escape_script_json_breaks_closing_script_tag() {
        let escaped = escape_script_json(r#"{"html":"</script><script>alert(1)</script>"}"#);
        assert!(escaped.contains(r#"<\/script>"#));
        assert!(!escaped.contains("</script><script>"));
    }

    #[tokio::test]
    async fn login_page_sanitizes_external_next_path() {
        let response = get("/login?next=https://evil.test").await;

        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains("登录到 RVideo"));
        assert!(html.contains("/settings"));
        assert!(!html.contains("evil.test"));
    }

    #[tokio::test]
    async fn login_page_preserves_internal_next_path() {
        let response = get("/login?next=%2Fpremium").await;

        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains("/premium"));
    }

    #[tokio::test]
    async fn legacy_rust_home_is_not_found() {
        let response = get("/rust").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_premium_is_not_found() {
        let response = get("/rust/premium").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_settings_is_not_found() {
        let response = get("/rust/settings").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_admin_is_not_found() {
        let response = get("/rust/admin").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_premium_settings_is_not_found() {
        let response = get("/rust/premium/settings").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_detail_is_not_found() {
        let response = get("/rust/detail?id=123&source=test-source").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn legacy_rust_player_is_not_found() {
        let response = get("/rust/player?id=123&source=test-source").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn root_alias_redirects_to_login_when_unauthenticated() {
        let response = get("/").await;
        assert_login_redirect(&response, "/login?next=%2F");
    }

    #[tokio::test]
    async fn top_level_login_alias_renders_login_page() {
        let response = get("/login").await;
        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains("登录"));
        assert!(html.contains("login-form"));
    }

    #[tokio::test]
    async fn top_level_settings_alias_redirects_to_login_when_unauthenticated() {
        let response = get("/settings").await;
        assert_login_redirect(&response, "/login?next=%2Fsettings");
    }

    #[tokio::test]
    async fn top_level_player_alias_redirects_to_login_when_unauthenticated() {
        let response = get("/player?id=123&source=test-source").await;
        assert_login_redirect(&response, "/login?next=%2Fplayer");
    }

    #[tokio::test]
    async fn login_page_renders_rust_navigation_shell() {
        let response = get("/login").await;
        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains("登录"));
        assert!(html.contains("RVideo"));
    }

    #[tokio::test]
    async fn public_manifest_is_served() {
        let response = get("/manifest.json").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("RVideo"));
    }

    #[tokio::test]
    async fn favicon_route_is_served() {
        let response = get("/favicon.ico").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn public_service_worker_is_served() {
        let response = get("/sw.js").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("self.addEventListener"));
    }
}
