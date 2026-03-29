use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

use crate::{auth, types::AppState};

use super::super::super::render::{html_response, render_shell, safe_next_path};
use super::super::shared::NextQuery;
use super::super::view::render_login_body;

pub(crate) async fn login_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NextQuery>,
) -> Response {
    let next = safe_next_path(query.next.as_deref(), "/settings");

    if matches!(
        auth::get_auth_user_optional(&headers, &state).await,
        Ok(Some(_))
    ) {
        return Redirect::to(&next).into_response();
    }

    let body = render_login_body(&next);

    html_response(render_shell("登录", None, "/login", &body))
}

#[cfg(test)]
mod tests {
    use super::render_login_body;

    #[test]
    fn render_login_body_keeps_next_path_payload() {
        let html = render_login_body("/player?id=1&source=test");

        assert!(html.contains("login-form"));
        assert!(html.contains("next-path"));
        assert!(html.contains("使用账号进入当前页面。"));
        assert!(html.contains("/player?id=1&source=test"));
    }
}
