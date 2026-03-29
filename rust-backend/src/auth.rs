use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db, db::DbUser, types::AppState};

const COOKIE_NAME: &str = "kvideo_token";
const ACCESS_COOKIE_NAME: &str = "kvideo_access_granted";

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub disable_premium: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub username: String,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(user: &DbUser, secret: &str) -> Result<String, String> {
    let now = Utc::now();
    let claims = Claims {
        user_id: user.id,
        username: user.username.clone(),
        is_admin: user.is_admin == 1,
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(24)).timestamp() as usize,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| error.to_string())
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|error| error.to_string())
}

pub async fn get_auth_user_optional(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<Option<AuthUser>, String> {
    let Some(token) = get_cookie(headers, COOKIE_NAME) else {
        return Ok(None);
    };

    let claims = match verify_token(&token, &state.config.auth_secret) {
        Ok(claims) => claims,
        Err(_) => return Ok(None),
    };

    let Some(user) = db::get_user_by_id(&state.db, claims.user_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };

    Ok(Some(AuthUser {
        id: user.id,
        username: user.username,
        is_admin: user.is_admin == 1,
        disable_premium: user.disable_premium == 1,
    }))
}

pub async fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<AuthUser, Response> {
    match get_auth_user_optional(headers, state).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "未登录" }))).into_response())
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "登录已过期" })),
        )
            .into_response()),
    }
}

pub async fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<AuthUser, Response> {
    let user = require_auth(headers, state).await?;
    if !user.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "error": "无权限" }))).into_response());
    }
    Ok(user)
}

pub fn attach_auth_cookie(response: &mut Response, token: &str, secure: bool) {
    let cookie = format!(
        "{COOKIE_NAME}={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=86400{}",
        if secure { "; Secure" } else { "" }
    );
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
}

pub fn clear_auth_cookie(response: &mut Response) {
    let cookie = format!(
        "{COOKIE_NAME}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
    );
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
}

pub fn attach_access_cookie(response: &mut Response, secure: bool, persist: bool) {
    let cookie = if persist {
        format!(
            "{ACCESS_COOKIE_NAME}=1; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000{}",
            if secure { "; Secure" } else { "" }
        )
    } else {
        format!(
            "{ACCESS_COOKIE_NAME}=1; HttpOnly; Path=/; SameSite=Lax{}",
            if secure { "; Secure" } else { "" }
        )
    };
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

pub fn clear_access_cookie(response: &mut Response, secure: bool) {
    let cookie = format!(
        "{ACCESS_COOKIE_NAME}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT{}",
        if secure { "; Secure" } else { "" }
    );
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

pub fn has_access_cookie(headers: &HeaderMap) -> bool {
    matches!(
        get_cookie(headers, ACCESS_COOKIE_NAME).as_deref(),
        Some("1")
    )
}

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|part| {
        let mut parts = part.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim();
        if key == name {
            Some(value.to_string())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbUser;
    use chrono::{DateTime, Utc};

    fn mock_user() -> DbUser {
        DbUser {
            id: 1,
            username: "admin".to_string(),
            password_hash: "hash".to_string(),
            is_admin: 1,
            disable_premium: 0,
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        }
    }

    #[test]
    fn token_roundtrip_works() {
        let token = create_token(&mock_user(), "secret").unwrap();
        let claims = verify_token(&token, "secret").unwrap();

        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "admin");
        assert!(claims.is_admin);
    }

    #[test]
    fn cookie_parser_extracts_named_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("foo=bar; kvideo_token=abc123; hello=world"),
        );

        assert_eq!(get_cookie(&headers, COOKIE_NAME).as_deref(), Some("abc123"));
    }
}
