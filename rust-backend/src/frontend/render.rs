use axum::response::{IntoResponse, Response};
use serde_json::Value as JsonValue;

use crate::auth;

use super::assets::{PAGE_STYLE, SHELL_APP_SCRIPT, SHELL_BOOTSTRAP_SCRIPT};

pub(super) fn build_rust_player_url(
    video_id: &str,
    source: &str,
    title: &str,
    grouped_sources: Option<&JsonValue>,
    episode: Option<usize>,
    is_premium: bool,
) -> String {
    let mut params = vec![
        ("id", video_id.to_string()),
        ("source", source.to_string()),
        ("title", title.to_string()),
    ];

    if let Some(episode) = episode {
        params.push(("episode", episode.to_string()));
    }
    if let Some(grouped_sources) = grouped_sources.filter(|value| value.is_array()) {
        let grouped_sources_json = grouped_sources.to_string();
        if grouped_sources_json != "[]" {
            params.push(("groupedSources", grouped_sources_json));
        }
    }
    if is_premium {
        params.push(("premium", "1".to_string()));
    }

    format!(
        "/player?{}",
        params
            .into_iter()
            .map(|(key, value)| format!("{}={}", key, urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&")
    )
}

pub(super) fn build_rust_detail_url(
    video_id: &str,
    source: &str,
    title: &str,
    grouped_sources: Option<&JsonValue>,
    is_premium: bool,
) -> String {
    let mut params = vec![
        ("id", video_id.to_string()),
        ("source", source.to_string()),
        ("title", title.to_string()),
    ];

    if let Some(grouped_sources) = grouped_sources.filter(|value| value.is_array()) {
        let grouped_sources_json = grouped_sources.to_string();
        if grouped_sources_json != "[]" {
            params.push(("groupedSources", grouped_sources_json));
        }
    }
    if is_premium {
        params.push(("premium", "1".to_string()));
    }

    format!(
        "/detail?{}",
        params
            .into_iter()
            .map(|(key, value)| format!("{}={}", key, urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&")
    )
}

pub(super) fn safe_next_path(next: Option<&str>, fallback: &str) -> String {
    let Some(next) = next.map(str::trim) else {
        return fallback.to_string();
    };
    if next.starts_with('/') && !next.starts_with("//") {
        next.to_string()
    } else {
        fallback.to_string()
    }
}

pub(super) fn render_shell(
    title: &str,
    user: Option<&auth::AuthUser>,
    active_path: &str,
    body: &str,
) -> String {
    let is_guest_login = user.is_none() && active_path == "/login";
    let is_compact_shell = matches!(
        active_path,
        "/" | "/player" | "/detail" | "/premium" | "/settings" | "/premium/settings" | "/admin"
    );
    let show_compact_user = matches!(active_path, "/settings" | "/premium/settings" | "/admin");
    let bootstrap_payload = serde_json::json!({
        "authenticated": user.is_some(),
        "user": user.map(|user| serde_json::json!({
            "id": user.id,
            "username": user.username,
            "isAdmin": user.is_admin,
            "disablePremium": user.disable_premium,
        })),
    });
    let brand_actions = if is_guest_login {
        String::new()
    } else if let Some(user) = user {
        let premium_link =
            (!user.disable_premium).then(|| nav_link("Premium", "/premium", active_path));
        let premium_settings_link = (!user.disable_premium)
            .then(|| nav_link("Premium 设置", "/premium/settings", active_path));
        let admin_link = user
            .is_admin
            .then(|| nav_link("管理后台", "/admin", active_path));

        let compact_extra_link = if active_path == "/premium/settings" {
            premium_settings_link.clone().unwrap_or_default()
        } else if active_path == "/admin" {
            admin_link.clone().unwrap_or_default()
        } else {
            String::new()
        };

        let nav_links = if is_compact_shell {
            vec![
                nav_link("搜索", "/", active_path),
                premium_link.unwrap_or_default(),
                nav_link("设置", "/settings", active_path),
                compact_extra_link,
            ]
        } else {
            vec![
                nav_link("搜索", "/", active_path),
                premium_link.unwrap_or_default(),
                nav_link("设置", "/settings", active_path),
                premium_settings_link.unwrap_or_default(),
                admin_link.unwrap_or_default(),
            ]
        };

        format!(
            r#"
<div class="row gap-sm shell-nav-links">
  {}
  {}
  {}
  {}{}
</div>
"#,
            nav_links.first().cloned().unwrap_or_default(),
            nav_links.get(1).cloned().unwrap_or_default(),
            nav_links.get(2).cloned().unwrap_or_default(),
            nav_links.get(3).cloned().unwrap_or_default(),
            if is_compact_shell {
                if show_compact_user {
                    format!(
                        "<span class=\"chip shell-user\">{}</span>",
                        escape_html(&user.username)
                    )
                } else {
                    String::new()
                }
            } else {
                format!(
                    "{}<span class=\"chip shell-user\">{}</span>",
                    nav_links.get(4).cloned().unwrap_or_default(),
                    escape_html(&user.username)
                )
            },
        )
    } else {
        format!(
            r#"
<div class="row gap-sm shell-nav-links">
  {}
</div>
"#,
            nav_link("登录", "/login", active_path),
        )
    };

    let shell_class = if is_guest_login {
        "shell shell-login"
    } else {
        "shell"
    };
    let header_class = if is_guest_login {
        "topbar card shell-topbar shell-topbar-minimal"
    } else if is_compact_shell {
        "topbar card shell-topbar shell-topbar-media"
    } else {
        "topbar card shell-topbar"
    };
    let main_class = if is_guest_login {
        "stack page-content login-page-content"
    } else {
        "stack page-content"
    };

    format!(
        r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="theme-color" content="#000000" />
    <meta name="apple-mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
    <meta name="apple-mobile-web-app-title" content="RVideo" />
    <link rel="manifest" href="/manifest.json" />
    <link rel="apple-touch-icon" href="/icon.png" />
    <title>{}</title>
    <style>{}</style>
    <script src="https://www.gstatic.com/cv/js/sender/v1/cast_sender.js?loadCastFramework=1" async></script>
  </head>
  <body>
    <div class="{}">
      <div id="bootstrap-overlay" class="bootstrap-overlay hidden" aria-hidden="true">
        <div class="bootstrap-card">
          <div class="bootstrap-spinner" aria-hidden="true"></div>
          <strong>账户同步中</strong>
          <span id="bootstrap-message" class="muted">正在初始化页面数据...</span>
        </div>
      </div>
      <div id="access-gate-overlay" class="access-gate-overlay hidden" aria-hidden="true">
        <form id="access-gate-form" class="access-gate-card" autocomplete="off" role="dialog" aria-modal="true" aria-label="访问密码门禁" tabindex="-1">
          <span class="eyebrow">访问控制</span>
          <strong>输入访问密码</strong>
          <p class="muted">当前页面启用了访问密码。解锁后会按当前配置在浏览器会话或持久 Cookie 中保持有效。</p>
          <input
            id="access-gate-password"
            class="input"
            type="password"
            placeholder="输入访问密码"
            autocomplete="current-password"
          />
          <button class="button primary" type="submit">解锁页面</button>
          <p id="access-gate-status" class="status muted">请输入访问密码继续使用。</p>
        </form>
      </div>
      <header class="{}">
        <a class="shell-brand" href="/">RVIDEO</a>
        {}
      </header>
      <script id="bootstrap-payload" type="application/json">{}</script>
      <script>{}</script>
      <script>{}</script>
      <main class="{}">{}</main>
      <button
        id="back-to-top"
        class="back-to-top"
        type="button"
        aria-label="返回顶部"
        title="返回顶部"
      >
        返回顶部
      </button>
    </div>
  </body>
</html>
"##,
        escape_html(title),
        PAGE_STYLE,
        shell_class,
        header_class,
        brand_actions,
        escape_script_json(&json_string(&bootstrap_payload)),
        SHELL_BOOTSTRAP_SCRIPT,
        SHELL_APP_SCRIPT,
        main_class,
        body,
    )
}

fn nav_link(label: &str, href: &str, active_path: &str) -> String {
    let class_name = if href == active_path {
        "button active"
    } else {
        "button"
    };
    format!(
        r#"<a class="{}" href="{}">{}</a>"#,
        class_name,
        href,
        escape_html(label),
    )
}

pub(super) fn preview_card(title: &str, description: &str) -> String {
    format!(
        r#"
<section class="preview-card stack compact-card">
  <span class="eyebrow">{}</span>
  <p class="muted">{}</p>
</section>
"#,
        escape_html(title),
        escape_html(description),
    )
}

pub(super) fn html_response(html: String) -> Response {
    axum::response::Html(html).into_response()
}

pub(super) fn pretty_json(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

pub(super) fn count_enabled_sources(settings_value: &JsonValue) -> usize {
    settings_value
        .get("sources")
        .and_then(JsonValue::as_array)
        .map(|sources| {
            sources
                .iter()
                .filter(|source| {
                    source.get("id").and_then(JsonValue::as_str).is_some()
                        && source
                            .get("enabled")
                            .and_then(JsonValue::as_bool)
                            .unwrap_or(true)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn json_array_len(value: &JsonValue) -> usize {
    value.as_array().map(Vec::len).unwrap_or(0)
}

pub(super) fn render_saved_items_list(
    value: &JsonValue,
    empty_message: &str,
    fallback_href: &str,
) -> String {
    let Some(items) = value.as_array() else {
        return format!(
            r#"<div class="empty-state">{}</div>"#,
            escape_html(empty_message)
        );
    };

    if items.is_empty() {
        return format!(
            r#"<div class="empty-state">{}</div>"#,
            escape_html(empty_message)
        );
    }

    let entries = items
        .iter()
        .take(5)
        .map(|item| {
            let title = item
                .get("title")
                .and_then(JsonValue::as_str)
                .unwrap_or("未命名视频");
            let source = item
                .get("sourceName")
                .or_else(|| item.get("source"))
                .and_then(JsonValue::as_str)
                .unwrap_or("未知来源");
            let href = match (
                item.get("videoId")
                    .or_else(|| item.get("vod_id"))
                    .and_then(|value| match value {
                        JsonValue::Number(number) => Some(number.to_string()),
                        JsonValue::String(text) => Some(text.clone()),
                        _ => None,
                    }),
                item.get("source").and_then(JsonValue::as_str),
            ) {
                (Some(video_id), Some(source_id)) => {
                    build_rust_player_url(&video_id, source_id, title, None, None, false)
                }
                _ => fallback_href.to_string(),
            };

            format!(
                r#"
<a class="saved-item" href="{}">
  <strong>{}</strong>
  <span class="muted">{}</span>
</a>
"#,
                escape_html(&href),
                escape_html(title),
                escape_html(source),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<div class="saved-list">{}</div>"#, entries)
}

pub(super) fn render_home_library_entries(value: &JsonValue, empty_message: &str) -> String {
    let Some(items) = value.as_array() else {
        return format!(
            r#"<div class="empty-state">{}</div>"#,
            escape_html(empty_message)
        );
    };

    if items.is_empty() {
        return format!(
            r#"<div class="empty-state">{}</div>"#,
            escape_html(empty_message)
        );
    }

    let entries = items
        .iter()
        .take(8)
        .map(|item| {
            let title = item
                .get("title")
                .and_then(JsonValue::as_str)
                .unwrap_or("未命名视频");
            let source = item
                .get("sourceName")
                .or_else(|| item.get("source"))
                .and_then(JsonValue::as_str)
                .unwrap_or("未知来源");
            let is_premium = item
                .get("premium")
                .or_else(|| item.get("isPremium"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(false);

            let video_id = item
                .get("videoId")
                .or_else(|| item.get("vod_id"))
                .and_then(|value| match value {
                    JsonValue::Number(number) => Some(number.to_string()),
                    JsonValue::String(text) => Some(text.clone()),
                    _ => None,
                });
            let source_id = item
                .get("source")
                .and_then(JsonValue::as_str)
                .unwrap_or_default();

            let (player_href, detail_href) = match video_id {
                Some(ref id) if !source_id.is_empty() => (
                    build_rust_player_url(id, source_id, title, None, None, is_premium),
                    build_rust_detail_url(id, source_id, title, None, is_premium),
                ),
                _ => ("/player".to_string(), "/detail".to_string()),
            };

            format!(
                r#"
<article class="saved-item library-entry">
  <div class="stack compact-card-body">
    <strong>{}</strong>
    <div class="source-item-meta">
      <span>{}</span>
      <span>{}</span>
    </div>
  </div>
  <div class="row wrap gap-sm library-actions">
    <a class="button button-small" href="{}">播放</a>
    <a class="button button-small" href="{}">详情</a>
    <button class="button button-small" type="button" data-library-query="{}">搜同名</button>
  </div>
</article>
"#,
                escape_html(title),
                escape_html(source),
                if is_premium {
                    "Premium 内容"
                } else {
                    "普通内容"
                },
                escape_html(&player_href),
                escape_html(&detail_href),
                escape_html(title),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<div class="saved-list">{}</div>"#, entries)
}

pub(super) fn find_source_config(settings_value: &JsonValue, source_id: &str) -> Option<JsonValue> {
    for key in ["sources", "premiumSources"] {
        let Some(sources) = settings_value.get(key).and_then(JsonValue::as_array) else {
            continue;
        };

        if let Some(source) = sources.iter().find(|source| {
            source
                .get("id")
                .and_then(JsonValue::as_str)
                .map(|id| id == source_id)
                .unwrap_or(false)
        }) {
            return Some(source.clone());
        }
    }

    None
}

pub(super) fn json_string<T: serde::Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

pub(super) fn escape_script_json(value: &str) -> String {
    value.replace("</script", "<\\/script")
}

pub(super) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
