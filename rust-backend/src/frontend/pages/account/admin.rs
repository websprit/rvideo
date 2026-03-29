use axum::{extract::State, http::HeaderMap, response::Response};

use crate::types::AppState;

use super::super::super::render::{html_response, render_shell};
use super::super::shared::{require_admin_user, require_page_auth};
use super::super::view::render_admin_body;

pub(crate) async fn admin_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/admin").await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if let Err(response) = require_admin_user(&auth_user) {
        return response;
    }

    let body = render_admin_body(&auth_user);

    html_response(render_shell("管理后台", Some(&auth_user), "/admin", &body))
}

#[cfg(test)]
mod tests {
    use super::render_admin_body;
    use crate::auth::AuthUser;

    #[test]
    fn render_admin_body_shows_management_controls() {
        let user = AuthUser {
            id: 1,
            username: "root".to_string(),
            is_admin: true,
            disable_premium: false,
        };
        let html = render_admin_body(&user);

        assert!(html.contains("create-user-form"));
        assert!(html.contains("admin-password-form"));
        assert!(html.contains("admin-users-count"));
        assert!(html.contains("当前管理员：root"));
    }
}
