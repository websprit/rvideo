use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, header},
    response::{IntoResponse, Sse, sse::Event},
    routing::{get, post, put},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use rand::prelude::IndexedRandom;
use serde::Deserialize;
use serde_json::{Value as JsonValue, json};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use url::Url;

use crate::{
    auth, db,
    types::{
        ApiDetailResponse, ApiSearchResponse, AppState, Episode, UpstreamCategory, VideoDetail,
        VideoItem, VideoSource,
    },
};

const VALID_USER_DATA_KEYS: &[&str] = &[
    "settings",
    "history",
    "favorites",
    "search-history",
    "premium-search-history",
    "premium-history",
    "premium-favorites",
    "search-cache",
    "premium-tags",
];

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/search-parallel", post(search_parallel))
        .route("/api/detail", get(detail_get).post(detail_post))
        .route("/api/proxy", get(proxy_get).options(proxy_options))
        .route("/api/ping", post(ping))
        .route("/api/config", get(config_get).post(config_post))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/auth/access-unlock", post(access_unlock))
        .route("/api/auth/password", put(change_password))
        .route(
            "/api/admin/users",
            get(admin_users_get).post(admin_users_post),
        )
        .route(
            "/api/admin/users/{id}",
            put(admin_user_put).delete(admin_user_delete),
        )
        .route("/api/user/data", get(user_data_get).put(user_data_put))
        .route(
            "/api/premium/types",
            get(premium_types_get).post(premium_types_post),
        )
        .route(
            "/api/premium/category",
            get(premium_category_get).post(premium_category_post),
        )
        .route("/api/douban/tags", get(douban_tags_get))
        .route("/api/douban/recommend", get(douban_recommend_get))
        .route("/api/douban/image", get(douban_image_get))
        .with_state(state)
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

#[derive(Deserialize)]
struct SearchParallelRequest {
    query: Option<String>,
    sources: Option<Vec<VideoSource>>,
    page: Option<u32>,
}

async fn search_parallel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SearchParallelRequest>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let query = payload.query.unwrap_or_default().trim().to_string();
    let sources = payload.sources.unwrap_or_default();
    let page = payload.page.unwrap_or(1);

    let (tx, rx) = mpsc::channel::<String>(128);

    if query.is_empty() {
        let _ = tx
            .send(json!({ "type": "error", "message": "Invalid query" }).to_string())
            .await;
        drop(tx);
    } else if sources.is_empty() {
        let _ = tx
            .send(json!({ "type": "error", "message": "No valid sources provided" }).to_string())
            .await;
        drop(tx);
    } else {
        let total_sources = sources.len();
        let _ = tx
            .send(json!({ "type": "start", "totalSources": total_sources }).to_string())
            .await;

        let completed_sources = Arc::new(AtomicUsize::new(0));
        let total_videos_found = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(sources.len());

        for source in sources {
            let tx = tx.clone();
            let state = state.clone();
            let query = query.clone();
            let completed_sources = completed_sources.clone();
            let total_videos_found = total_videos_found.clone();

            handles.push(tokio::spawn(async move {
                let started_at = Instant::now();
                match search_videos_by_source(&state, &query, &source, page).await {
                    Ok(videos) => {
                        let latency = started_at.elapsed().as_millis();
                        let found = videos.len();
                        total_videos_found.fetch_add(found, Ordering::SeqCst);
                        let completed = completed_sources.fetch_add(1, Ordering::SeqCst) + 1;
                        let total_found = total_videos_found.load(Ordering::SeqCst);

                        if !videos.is_empty() {
                            let videos = videos
                                .into_iter()
                                .map(|video| {
                                    json!({
                                        "vod_id": video.vod_id,
                                        "vod_name": video.vod_name,
                                        "vod_pic": video.vod_pic,
                                        "type_name": video.type_name,
                                        "vod_remarks": video.vod_remarks,
                                        "vod_year": video.vod_year,
                                        "vod_area": video.vod_area,
                                        "vod_actor": video.vod_actor,
                                        "vod_director": video.vod_director,
                                        "vod_content": video.vod_content,
                                        "source": video.source,
                                        "latency": latency,
                                        "sourceDisplayName": get_source_name(&source.id),
                                    })
                                })
                                .collect::<Vec<_>>();

                            let _ = tx
                                .send(
                                    json!({
                                        "type": "videos",
                                        "videos": videos,
                                        "source": source.id,
                                        "completedSources": completed,
                                        "totalSources": total_sources,
                                        "latency": latency,
                                    })
                                    .to_string(),
                                )
                                .await;
                        }

                        let _ = tx
                            .send(
                                json!({
                                    "type": "progress",
                                    "completedSources": completed,
                                    "totalSources": total_sources,
                                    "totalVideosFound": total_found,
                                })
                                .to_string(),
                            )
                            .await;
                    }
                    Err(error) => {
                        tracing::warn!(source = source.id, error = error, "search source failed");
                        let completed = completed_sources.fetch_add(1, Ordering::SeqCst) + 1;
                        let total_found = total_videos_found.load(Ordering::SeqCst);
                        let _ = tx
                            .send(
                                json!({
                                    "type": "progress",
                                    "completedSources": completed,
                                    "totalSources": total_sources,
                                    "totalVideosFound": total_found,
                                })
                                .to_string(),
                            )
                            .await;
                    }
                }
            }));
        }

        let tx_complete = tx.clone();
        let total_videos_found_complete = total_videos_found.clone();
        tokio::spawn(async move {
            for handle in handles {
                let _ = handle.await;
            }
            let _ = tx_complete
                .send(
                    json!({
                        "type": "complete",
                        "totalVideosFound": total_videos_found_complete.load(Ordering::SeqCst),
                        "totalSources": total_sources,
                    })
                    .to_string(),
                )
                .await;
        });

        drop(tx);
    }

    let stream =
        ReceiverStream::new(rx).map(|data| Ok::<Event, Infallible>(Event::default().data(data)));
    Sse::new(stream).into_response()
}

#[derive(Deserialize)]
struct DetailGetQuery {
    id: Option<String>,
    source: Option<String>,
}

#[derive(Deserialize)]
struct DetailPostBody {
    id: Option<JsonValue>,
    source: Option<JsonValue>,
}

async fn detail_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DetailGetQuery>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    handle_detail_request(
        &state,
        query.id.map(JsonValue::String),
        query.source.map(JsonValue::String),
    )
    .await
}

async fn detail_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<DetailPostBody>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    handle_detail_request(&state, body.id, body.source).await
}

async fn handle_detail_request(
    state: &AppState,
    id: Option<JsonValue>,
    source: Option<JsonValue>,
) -> Response<Body> {
    let Some(id_value) = id else {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "Missing video ID parameter" }),
        );
    };
    let Some(source_value) = source else {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "Missing source parameter" }),
        );
    };

    let Some(source_config) = resolve_source(&source_value) else {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "Invalid source configuration" }),
        );
    };

    match get_video_detail(state, &id_value, &source_config).await {
        Ok(detail) => json_response(StatusCode::OK, json!({ "success": true, "data": detail })),
        Err(error) => {
            tracing::error!(error = error, "detail api error");
            json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "error": error }),
            )
        }
    }
}

#[derive(Deserialize)]
struct ProxyQuery {
    url: Option<String>,
    referer: Option<String>,
    ip: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdFilterMode {
    Off,
    Keyword,
    Heuristic,
    Aggressive,
}

async fn proxy_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProxyQuery>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Some(target_url) = query.url else {
        return text_response(StatusCode::BAD_REQUEST, "Missing URL parameter");
    };

    let (ad_filter_mode, ad_keywords) = load_ad_filter_settings(&state, auth_user.id).await;

    let forwarded_headers = collect_forward_headers(&headers);
    let result = fetch_with_retry(
        &state,
        &target_url,
        &forwarded_headers,
        query.referer.as_deref(),
        query.ip.as_deref(),
    )
    .await;

    let upstream = match result {
        Ok(response) => response,
        Err(error) => {
            return json_with_headers(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Proxy request failed", "message": error, "url": target_url }),
                &[(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")],
            );
        }
    };

    let status = convert_status(upstream.status());
    if !upstream.status().is_success() {
        let content_type = upstream
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("text/plain")
            .to_string();
        let body_text = upstream
            .text()
            .await
            .unwrap_or_else(|_| format!("Upstream error: {}", status));
        return text_with_headers(
            status,
            body_text,
            &[
                (header::CONTENT_TYPE, content_type.as_str()),
                (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            ],
        );
    }

    let content_type = upstream
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    let is_m3u8_by_header = content_type
        .as_deref()
        .map(|value| {
            value.contains("application/vnd.apple.mpegurl")
                || value.contains("application/x-mpegurl")
        })
        .unwrap_or(false);

    if is_m3u8_by_header || is_m3u8_url(&target_url) {
        let text = upstream.text().await.unwrap_or_default();
        let trimmed = text.trim();
        if trimmed.starts_with("#EXTM3U") || trimmed.starts_with("#EXT-X-") {
            let origin = guess_origin_from_headers(&headers).unwrap_or_default();
            let modified =
                process_m3u8_content(&text, &target_url, &origin, ad_filter_mode, &ad_keywords);
            return text_with_headers(
                status,
                modified,
                &[
                    (header::CONTENT_TYPE, "application/vnd.apple.mpegurl"),
                    (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
                    (header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS"),
                    (
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        "Content-Type, Authorization",
                    ),
                ],
            );
        }

        return text_with_headers(
            status,
            text,
            &[
                (
                    header::CONTENT_TYPE,
                    content_type.as_deref().unwrap_or("text/plain"),
                ),
                (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            ],
        );
    }

    let upstream_headers = upstream.headers().clone();
    let upstream_stream = upstream.bytes_stream();
    let mut response = Response::new(Body::from_stream(upstream_stream));
    *response.status_mut() = status;
    copy_upstream_headers(&upstream_headers, response.headers_mut(), true);
    insert_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        "*",
    );
    insert_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, OPTIONS",
    );
    insert_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Content-Type, Authorization",
    );
    insert_header(
        response.headers_mut(),
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate, proxy-revalidate",
    );
    response
}

async fn proxy_options() -> Response<Body> {
    text_with_headers(
        StatusCode::NO_CONTENT,
        String::new(),
        &[
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS"),
            (
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Content-Type, Authorization",
            ),
        ],
    )
}

#[derive(Deserialize)]
struct PingBody {
    url: Option<String>,
}

async fn ping(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PingBody>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let Some(url) = body.url else {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "Invalid URL" }));
    };

    if Url::parse(&url).is_err() {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "Invalid URL format" }),
        );
    }

    let started = Instant::now();
    let head_result = state
        .http
        .head(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    if head_result.is_ok() {
        return json_response(
            StatusCode::OK,
            json!({ "latency": started.elapsed().as_millis(), "success": true }),
        );
    }

    let get_result = state
        .http
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    match get_result {
        Ok(_) => json_response(
            StatusCode::OK,
            json!({ "latency": started.elapsed().as_millis(), "success": true }),
        ),
        Err(_) => json_response(
            StatusCode::OK,
            json!({ "latency": started.elapsed().as_millis(), "success": false, "timeout": true }),
        ),
    }
}

async fn config_get(State(state): State<AppState>, headers: HeaderMap) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let has_env_password = state.config.access_password.is_some();
    let env_password_unlocked = has_env_password && auth::has_access_cookie(&headers);

    json_response(
        StatusCode::OK,
        json!({
            "hasEnvPassword": has_env_password,
            "persistPassword": state.config.persist_password,
            "envPasswordUnlocked": env_password_unlocked,
            "subscriptionSources": state.config.subscription_sources,
            "adKeywords": state.config.ad_keywords,
            "disablePremium": auth_user.disable_premium,
        }),
    )
}

async fn config_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    match serde_json::from_slice::<JsonValue>(&body) {
        Ok(_) => json_response(StatusCode::OK, json!({ "valid": true })),
        Err(_) => json_response(
            StatusCode::BAD_REQUEST,
            json!({ "valid": false, "message": "Invalid request" }),
        ),
    }
}

#[derive(Deserialize)]
struct LoginBody {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Deserialize)]
struct AccessUnlockBody {
    password: Option<String>,
}

async fn access_unlock(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AccessUnlockBody>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let Some(expected_password) = state.config.access_password.as_deref() else {
        let mut response = json_response(
            StatusCode::OK,
            json!({ "success": true, "envPasswordUnlocked": false, "hasEnvPassword": false }),
        );
        auth::clear_access_cookie(&mut response, state.config.is_production);
        return response;
    };

    let submitted_password = payload.password.unwrap_or_default();
    if submitted_password.trim().is_empty() || submitted_password != expected_password {
        let mut response =
            json_response(StatusCode::UNAUTHORIZED, json!({ "error": "访问密码错误" }));
        auth::clear_access_cookie(&mut response, state.config.is_production);
        return response;
    }

    let mut response = json_response(
        StatusCode::OK,
        json!({
            "success": true,
            "envPasswordUnlocked": true,
            "persistPassword": state.config.persist_password,
        }),
    );
    auth::attach_access_cookie(
        &mut response,
        state.config.is_production,
        state.config.persist_password,
    );
    response
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginBody>) -> Response<Body> {
    let username = body.username.unwrap_or_default();
    let password = body.password.unwrap_or_default();

    if username.is_empty() || password.is_empty() {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "请输入用户名和密码" }),
        );
    }

    let user = match db::get_user_by_username(&state.db, &username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return json_response(
                StatusCode::UNAUTHORIZED,
                json!({ "error": "用户名或密码错误" }),
            );
        }
        Err(error) => {
            error!("login get_user_by_username failed: {error}");
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "登录失败" }),
            );
        }
    };

    let password_matches: bool = db::verify_password(&password, &user.password_hash)
        .await
        .unwrap_or_default();

    if !password_matches {
        return json_response(
            StatusCode::UNAUTHORIZED,
            json!({ "error": "用户名或密码错误" }),
        );
    }

    let token = match auth::create_token(&user, &state.config.auth_secret) {
        Ok(token) => token,
        Err(error) => {
            error!("login create_token failed: {error}");
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "登录失败" }),
            );
        }
    };

    let mut response = json_response(
        StatusCode::OK,
        json!({
            "user": {
                "id": user.id,
                "username": user.username,
                "isAdmin": user.is_admin == 1,
                "disablePremium": user.disable_premium == 1,
            }
        }),
    );
    auth::attach_auth_cookie(&mut response, &token, state.config.is_production);
    response
}

async fn logout() -> Response<Body> {
    let mut response = json_response(StatusCode::OK, json!({ "success": true }));
    auth::clear_auth_cookie(&mut response);
    response
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    json_response(
        StatusCode::OK,
        json!({
            "user": {
                "id": auth_user.id,
                "username": auth_user.username,
                "isAdmin": auth_user.is_admin,
                "disablePremium": auth_user.disable_premium,
            }
        }),
    )
}

#[derive(Deserialize)]
struct ChangePasswordBody {
    #[serde(rename = "currentPassword")]
    current_password: Option<String>,
    #[serde(rename = "newPassword")]
    new_password: Option<String>,
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordBody>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let current_password = body.current_password.unwrap_or_default();
    let new_password = body.new_password.unwrap_or_default();

    if current_password.is_empty() || new_password.is_empty() {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "请输入当前密码和新密码" }),
        );
    }
    if new_password.len() < 6 {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "新密码至少6个字符" }),
        );
    }

    let user = match db::get_user_by_id(&state.db, auth_user.id).await {
        Ok(Some(user)) => user,
        _ => return json_response(StatusCode::UNAUTHORIZED, json!({ "error": "当前密码错误" })),
    };

    let valid = db::verify_password(&current_password, &user.password_hash)
        .await
        .unwrap_or(false);
    if !valid {
        return json_response(StatusCode::UNAUTHORIZED, json!({ "error": "当前密码错误" }));
    }

    match db::update_user(&state.db, auth_user.id, None, Some(&new_password), None).await {
        Ok(_) => json_response(StatusCode::OK, json!({ "success": true })),
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "修改密码失败" }),
        ),
    }
}

async fn admin_users_get(State(state): State<AppState>, headers: HeaderMap) -> Response<Body> {
    if let Err(response) = auth::require_admin(&headers, &state).await {
        return response;
    }

    match db::get_all_users(&state.db).await {
        Ok(users) => json_response(
            StatusCode::OK,
            json!({
                "users": users.into_iter().map(public_user_json).collect::<Vec<_>>()
            }),
        ),
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "获取用户失败" }),
        ),
    }
}

#[derive(Deserialize)]
struct AdminCreateUserBody {
    username: Option<String>,
    password: Option<String>,
    #[serde(rename = "disablePremium")]
    disable_premium: Option<bool>,
}

async fn admin_users_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminCreateUserBody>,
) -> Response<Body> {
    if let Err(response) = auth::require_admin(&headers, &state).await {
        return response;
    }

    let username = body.username.unwrap_or_default();
    let password = body.password.unwrap_or_default();

    if username.is_empty() || password.is_empty() {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "用户名和密码不能为空" }),
        );
    }
    if password.len() < 6 {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "密码至少6个字符" }),
        );
    }

    if matches!(
        db::get_user_by_username(&state.db, &username).await,
        Ok(Some(_))
    ) {
        return json_response(StatusCode::CONFLICT, json!({ "error": "用户名已存在" }));
    }

    match db::create_user(
        &state.db,
        &username,
        &password,
        body.disable_premium != Some(false),
    )
    .await
    {
        Ok(user) => json_response(StatusCode::OK, json!({ "user": public_user_json(user) })),
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "创建用户失败" }),
        ),
    }
}

#[derive(Deserialize)]
struct AdminUpdateUserBody {
    username: Option<String>,
    password: Option<String>,
    #[serde(rename = "disablePremium")]
    disable_premium: Option<bool>,
}

async fn admin_user_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AdminUpdateUserBody>,
) -> Response<Body> {
    if let Err(response) = auth::require_admin(&headers, &state).await {
        return response;
    }

    let Ok(user_id) = id.parse::<i64>() else {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的用户ID" }));
    };

    let target_user = match db::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return json_response(StatusCode::NOT_FOUND, json!({ "error": "用户不存在" })),
        Err(_) => {
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "更新用户失败" }),
            );
        }
    };

    if let Some(username) = body.username.as_deref()
        && username != target_user.username
        && matches!(
            db::get_user_by_username(&state.db, username).await,
            Ok(Some(_))
        )
    {
        return json_response(StatusCode::CONFLICT, json!({ "error": "用户名已存在" }));
    }

    match db::update_user(
        &state.db,
        user_id,
        body.username.as_deref(),
        body.password.as_deref(),
        body.disable_premium,
    )
    .await
    {
        Ok(_) => match db::get_user_by_id(&state.db, user_id).await {
            Ok(Some(user)) => {
                json_response(StatusCode::OK, json!({ "user": public_user_json(user) }))
            }
            _ => json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "更新用户失败" }),
            ),
        },
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "更新用户失败" }),
        ),
    }
}

async fn admin_user_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response<Body> {
    let auth_user = match auth::require_admin(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Ok(user_id) = id.parse::<i64>() else {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的用户ID" }));
    };

    if user_id == auth_user.id {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "不能删除自己" }));
    }

    match db::delete_user(&state.db, user_id).await {
        Ok(true) => json_response(StatusCode::OK, json!({ "success": true })),
        Ok(false) => json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "无法删除该用户（可能是管理员）" }),
        ),
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "删除用户失败" }),
        ),
    }
}

#[derive(Deserialize)]
struct UserDataQuery {
    key: Option<String>,
}

async fn user_data_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UserDataQuery>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Some(key) = query.key else {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的数据键" }));
    };
    if !VALID_USER_DATA_KEYS.contains(&key.as_str()) {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的数据键" }));
    }

    match db::get_user_data(&state.db, auth_user.id, &key).await {
        Ok(data) => match serde_json::from_str::<JsonValue>(&data) {
            Ok(value) => json_response(StatusCode::OK, json!({ "data": value })),
            Err(_) => json_response(StatusCode::OK, json!({ "data": {} })),
        },
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "读取失败" }),
        ),
    }
}

#[derive(Deserialize)]
struct UserDataPutBody {
    key: Option<String>,
    value: Option<JsonValue>,
}

async fn user_data_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UserDataPutBody>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Some(key) = body.key else {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的数据键" }));
    };
    if !VALID_USER_DATA_KEYS.contains(&key.as_str()) {
        return json_response(StatusCode::BAD_REQUEST, json!({ "error": "无效的数据键" }));
    }

    match db::set_user_data(
        &state.db,
        auth_user.id,
        &key,
        &serde_json::to_string(&body.value.unwrap_or(JsonValue::Null))
            .unwrap_or_else(|_| "null".to_string()),
    )
    .await
    {
        Ok(_) => json_response(StatusCode::OK, json!({ "success": true })),
        Err(_) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "保存失败" }),
        ),
    }
}

#[derive(Deserialize)]
struct PremiumTypesBody {
    sources: Option<Vec<VideoSource>>,
}

async fn premium_types_get(State(state): State<AppState>, headers: HeaderMap) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if auth_user.disable_premium {
        return json_response(StatusCode::OK, json!({ "tags": [] }));
    }
    handle_types_request(&state, vec![]).await
}

async fn premium_types_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PremiumTypesBody>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if auth_user.disable_premium {
        return json_response(StatusCode::OK, json!({ "tags": [] }));
    }
    handle_types_request(&state, body.sources.unwrap_or_default()).await
}

#[derive(Deserialize)]
struct PremiumCategoryQuery {
    category: Option<String>,
    page: Option<String>,
    limit: Option<String>,
}

#[derive(Deserialize)]
struct PremiumCategoryBody {
    sources: Option<Vec<VideoSource>>,
    category: Option<String>,
    page: Option<JsonValue>,
    limit: Option<JsonValue>,
}

async fn premium_category_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PremiumCategoryQuery>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if auth_user.disable_premium {
        return json_response(StatusCode::OK, json!({ "videos": [] }));
    }
    handle_category_request(
        &state,
        vec![],
        query.category.unwrap_or_default(),
        query
            .page
            .as_deref()
            .unwrap_or("1")
            .parse::<u32>()
            .unwrap_or(1),
        query
            .limit
            .as_deref()
            .unwrap_or("20")
            .parse::<usize>()
            .unwrap_or(20),
    )
    .await
}

async fn premium_category_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PremiumCategoryBody>,
) -> Response<Body> {
    let auth_user = match auth::require_auth(&headers, &state).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if auth_user.disable_premium {
        return json_response(StatusCode::OK, json!({ "videos": [] }));
    }
    handle_category_request(
        &state,
        body.sources.unwrap_or_default(),
        body.category.unwrap_or_default(),
        json_value_to_u32(body.page.as_ref()).unwrap_or(1),
        json_value_to_usize(body.limit.as_ref()).unwrap_or(20),
    )
    .await
}

#[derive(Deserialize)]
struct DoubanTagsQuery {
    #[serde(rename = "type")]
    content_type: Option<String>,
}

async fn douban_tags_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DoubanTagsQuery>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let content_type = query.content_type.unwrap_or_else(|| "movie".to_string());
    let url = format!("https://movie.douban.com/j/search_tags?type={content_type}&source=index");

    match state
        .http
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .header(reqwest::header::REFERER, "https://movie.douban.com/")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            match response.json::<JsonValue>().await {
                Ok(data) => json_response(StatusCode::OK, data),
                Err(_) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "tags": [], "error": "Failed to fetch tags" }),
                ),
            }
        }
        _ => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "tags": [], "error": "Failed to fetch tags" }),
        ),
    }
}

#[derive(Deserialize)]
struct DoubanRecommendQuery {
    tag: Option<String>,
    page_limit: Option<String>,
    page_start: Option<String>,
    #[serde(rename = "type")]
    content_type: Option<String>,
}

async fn douban_recommend_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DoubanRecommendQuery>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let tag = query.tag.unwrap_or_else(|| "热门".to_string());
    let page_limit = query.page_limit.unwrap_or_else(|| "20".to_string());
    let page_start = query.page_start.unwrap_or_else(|| "0".to_string());
    let content_type = query.content_type.unwrap_or_else(|| "movie".to_string());
    let url = format!(
        "https://movie.douban.com/j/search_subjects?type={}&tag={}&sort=recommend&page_limit={}&page_start={}",
        content_type,
        urlencoding::encode(&tag),
        page_limit,
        page_start,
    );

    let result = state
        .http
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .header(reqwest::header::REFERER, "https://movie.douban.com/")
        .send()
        .await;

    let Ok(response) = result else {
        return json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "subjects": [], "error": "Failed to fetch recommendations" }),
        );
    };

    if !response.status().is_success() {
        return json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "subjects": [], "error": "Failed to fetch recommendations" }),
        );
    }

    let Ok(mut data) = response.json::<JsonValue>().await else {
        return json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "subjects": [], "error": "Failed to fetch recommendations" }),
        );
    };

    if let Some(subjects) = data.get_mut("subjects").and_then(JsonValue::as_array_mut) {
        for item in subjects.iter_mut() {
            if let Some(cover) = item.get("cover").and_then(JsonValue::as_str) {
                let cover = cover.to_string();
                *item = merge_json_object(
                    item.take(),
                    json!({ "cover": format!("/api/douban/image?url={}", urlencoding::encode(&cover)) }),
                );
            }
        }
    }

    json_response(StatusCode::OK, data)
}

#[derive(Deserialize)]
struct DoubanImageQuery {
    url: Option<String>,
}

async fn douban_image_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DoubanImageQuery>,
) -> Response<Body> {
    if let Err(response) = auth::require_auth(&headers, &state).await {
        return response;
    }

    let Some(image_url) = query.url else {
        return json_response(
            StatusCode::BAD_REQUEST,
            json!({ "error": "Missing image URL" }),
        );
    };

    let result = state
        .http
        .get(image_url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        )
        .header(reqwest::header::ACCEPT, "image/jpeg,image/png,image/gif,*/*;q=0.8")
        .header(reqwest::header::REFERER, "https://movie.douban.com/")
        .send()
        .await;

    let Ok(response) = result else {
        return json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "Error fetching image" }),
        );
    };

    if !response.status().is_success() {
        return json_response(
            convert_status(response.status()),
            json!({ "error": response.status().canonical_reason().unwrap_or("Error") }),
        );
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = response.bytes().await.unwrap_or_default();

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    insert_header(response.headers_mut(), header::CONTENT_TYPE, &content_type);
    insert_header(
        response.headers_mut(),
        header::CACHE_CONTROL,
        "public, max-age=15720000, s-maxage=15720000",
    );
    response
}

async fn handle_types_request(state: &AppState, source_list: Vec<VideoSource>) -> Response<Body> {
    let enabled_sources = source_list
        .into_iter()
        .filter(|source| source.enabled.unwrap_or(false))
        .collect::<Vec<_>>();

    let futures = enabled_sources.into_iter().map(|source| {
        let state = state.clone();
        async move {
            let mut url = match Url::parse(&source.base_url) {
                Ok(url) => url,
                Err(_) => return None,
            };
            url.query_pairs_mut().append_pair("ac", "list");

            let response = state
                .http
                .get(url.as_str())
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
                )
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
                .ok()?;

            if !response.status().is_success() {
                return None;
            }

            let data = response.json::<JsonValue>().await.ok()?;
            let categories = serde_json::from_value::<Vec<UpstreamCategory>>(
                data.get("class")
                    .cloned()
                    .unwrap_or_else(|| JsonValue::Array(vec![])),
            )
            .unwrap_or_default();

            Some((source.id, categories))
        }
    });

    let results = futures_util::future::join_all(futures).await;
    let mut all_tags = vec![json!({ "id": "recommend", "label": "今日推荐", "value": "" })];
    let mut merged_categories: Vec<(String, Vec<String>)> = Vec::new();

    for result in results.into_iter().flatten() {
        let (source_id, categories) = result;
        for category in categories {
            let type_name = category.type_name.trim().to_string();
            if type_name.is_empty() {
                continue;
            }

            let value = format!(
                "{}:{}",
                source_id,
                json_value_to_compact_string(&category.type_id)
            );
            let mut matched = false;

            for existing in &mut merged_categories {
                if is_fuzzy_match(&existing.0, &type_name) {
                    existing.1.push(value.clone());
                    matched = true;
                    break;
                }
            }

            if !matched {
                merged_categories.push((type_name, vec![value]));
            }
        }
    }

    for (label, values) in merged_categories {
        if values.is_empty() {
            continue;
        }
        all_tags.push(json!({
            "id": STANDARD.encode(label.as_bytes()),
            "label": label,
            "value": values.join(","),
        }));
    }

    json_response(StatusCode::OK, json!({ "tags": all_tags }))
}

async fn handle_category_request(
    state: &AppState,
    source_list: Vec<VideoSource>,
    category_param: String,
    page: u32,
    _limit: usize,
) -> Response<Body> {
    let mut source_map = HashMap::<String, String>::new();
    if !category_param.is_empty() {
        for part in category_param.split(',') {
            if let Some((source_id, type_id)) = part.split_once(':') {
                source_map.insert(source_id.to_string(), type_id.to_string());
            } else if let Some(first_source) = source_list
                .iter()
                .find(|source| source.enabled.unwrap_or(false))
            {
                source_map.insert(first_source.id.clone(), part.to_string());
            }
        }
    }

    let target_sources = if source_map.is_empty() {
        source_list
            .into_iter()
            .filter(|source| source.enabled.unwrap_or(false))
            .collect::<Vec<_>>()
    } else {
        source_list
            .into_iter()
            .filter(|source| source.enabled.unwrap_or(false) && source_map.contains_key(&source.id))
            .collect::<Vec<_>>()
    };

    if target_sources.is_empty() {
        return json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "videos": [], "error": "No enabled sources provided or found" }),
        );
    }

    let futures = target_sources.into_iter().map(|source| {
        let state = state.clone();
        let type_id = source_map.get(&source.id).cloned();
        async move {
            let mut url = match Url::parse(&source.base_url) {
                Ok(url) => url,
                Err(_) => return Vec::<JsonValue>::new(),
            };
            {
                let mut query = url.query_pairs_mut();
                query.append_pair("ac", "detail");
                query.append_pair("pg", &page.to_string());
                if let Some(type_id) = type_id.as_deref() {
                    query.append_pair("t", type_id);
                }
            }

            let response = state
                .http
                .get(url.as_str())
                .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(8))
                .send()
                .await;

            let Ok(response) = response else {
                return Vec::<JsonValue>::new();
            };
            if !response.status().is_success() {
                return Vec::<JsonValue>::new();
            }

            let data = response.json::<JsonValue>().await.unwrap_or_else(|_| json!({}));
            data.get("list")
                .and_then(JsonValue::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|item| {
                    json!({
                        "vod_id": item.get("vod_id").cloned().unwrap_or(JsonValue::Null),
                        "vod_name": item.get("vod_name").cloned().unwrap_or_else(|| JsonValue::String(String::new())),
                        "vod_pic": item.get("vod_pic").cloned().unwrap_or(JsonValue::Null),
                        "vod_remarks": item.get("vod_remarks").cloned().unwrap_or(JsonValue::Null),
                        "type_name": item.get("type_name").cloned().unwrap_or(JsonValue::Null),
                        "source": source.id,
                    })
                })
                .collect::<Vec<_>>()
        }
    });

    let results = futures_util::future::join_all(futures).await;
    let max_len = results.iter().map(Vec::len).max().unwrap_or(0);
    let mut interleaved = Vec::new();
    for index in 0..max_len {
        for result in &results {
            if let Some(item) = result.get(index) {
                interleaved.push(item.clone());
            }
        }
    }

    json_response(StatusCode::OK, json!({ "videos": interleaved }))
}

async fn search_videos_by_source(
    state: &AppState,
    query: &str,
    source: &VideoSource,
    page: u32,
) -> Result<Vec<VideoItem>, String> {
    let mut url = Url::parse(&format!("{}{}", source.base_url, source.search_path))
        .map_err(|error| error.to_string())?;
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("ac", "detail");
        params.append_pair("wd", query);
        params.append_pair("pg", &page.to_string());
    }

    let response = get_with_retry(
        state,
        url.as_str(),
        source.headers.as_ref(),
        std::time::Duration::from_secs(15),
        3,
    )
    .await?;

    let data = response
        .json::<ApiSearchResponse>()
        .await
        .map_err(|error| error.to_string())?;
    if data.code != 0 && data.code != 1 {
        return Err(data
            .msg
            .unwrap_or_else(|| "Invalid API response".to_string()));
    }

    Ok(data
        .list
        .into_iter()
        .map(|mut item| {
            item.source = source.id.clone();
            item
        })
        .collect())
}

async fn get_video_detail(
    state: &AppState,
    id: &JsonValue,
    source: &VideoSource,
) -> Result<VideoDetail, String> {
    let mut url = Url::parse(&format!("{}{}", source.base_url, source.detail_path))
        .map_err(|error| error.to_string())?;
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("ac", "detail");
        params.append_pair("ids", &json_value_to_compact_string(id));
    }

    let response = get_with_retry(
        state,
        url.as_str(),
        source.headers.as_ref(),
        std::time::Duration::from_secs(15),
        3,
    )
    .await?;

    let data = response
        .json::<ApiDetailResponse>()
        .await
        .map_err(|error| error.to_string())?;
    if data.code != 0 && data.code != 1 {
        return Err(data
            .msg
            .unwrap_or_else(|| "Invalid API response".to_string()));
    }
    let Some(video) = data.list.into_iter().next() else {
        return Err("Video not found".to_string());
    };

    let play_from = video
        .vod_play_from
        .unwrap_or_default()
        .split("$$$")
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let play_urls = video
        .vod_play_url
        .unwrap_or_default()
        .split("$$$")
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    let mut selected_index = 0usize;
    if let Some(index) = play_from
        .iter()
        .position(|code| code.to_lowercase().contains("m3u8"))
        && index < play_urls.len()
    {
        selected_index = index;
    }

    Ok(VideoDetail {
        vod_id: video.vod_id,
        vod_name: video.vod_name,
        vod_pic: video.vod_pic,
        vod_remarks: video.vod_remarks,
        vod_year: video.vod_year,
        vod_area: video.vod_area,
        vod_actor: video.vod_actor,
        vod_director: video.vod_director,
        vod_content: video.vod_content,
        type_name: video.type_name,
        episodes: parse_episodes(
            play_urls
                .get(selected_index)
                .map(String::as_str)
                .unwrap_or_default(),
        ),
        source: source.id.clone(),
        source_code: play_from.get(selected_index).cloned().unwrap_or_default(),
    })
}

async fn get_with_retry(
    state: &AppState,
    url: &str,
    extra_headers: Option<&HashMap<String, String>>,
    timeout: std::time::Duration,
    retries: usize,
) -> Result<reqwest::Response, String> {
    let mut last_error = None;
    for attempt in 0..=retries {
        let mut request = state
            .http
            .get(url)
            .timeout(timeout)
            .header(reqwest::header::USER_AGENT, "Mozilla/5.0");
        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response) => {
                last_error = Some(format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Error")
                ))
            }
            Err(error) => last_error = Some(error.to_string()),
        }

        if attempt < retries {
            tokio::time::sleep(std::time::Duration::from_millis(200 * (attempt as u64 + 1))).await;
        }
    }

    Err(last_error.unwrap_or_else(|| "Unknown error".to_string()))
}

async fn fetch_with_retry(
    state: &AppState,
    url: &str,
    forwarded_headers: &HashMap<String, String>,
    referer: Option<&str>,
    ip: Option<&str>,
) -> Result<reqwest::Response, String> {
    let user_agents = [
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0",
    ];

    let random_ua = {
        let mut rng = rand::rng();
        user_agents
            .choose(&mut rng)
            .copied()
            .unwrap_or(user_agents[0])
    };
    let video_url = Url::parse(url).map_err(|error| error.to_string())?;
    let referer = referer.map(ToOwned::to_owned).unwrap_or_else(|| {
        format!(
            "{}://{}",
            video_url.scheme(),
            video_url.host_str().unwrap_or_default()
        )
    });
    let forwarded_ip = ip.unwrap_or("202.108.22.5");

    let max_retries = 5usize;
    let mut last_error = None;
    let mut response = None;

    for attempt in 1..=max_retries {
        if attempt > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(
                (1u64 << (attempt - 2)) * 100,
            ))
            .await;
        }

        let mut request = state
            .http
            .get(url)
            .timeout(std::time::Duration::from_secs(30));
        for (key, value) in forwarded_headers {
            request = request.header(key, value);
        }

        request = request
            .header(reqwest::header::USER_AGENT, random_ua)
            .header(reqwest::header::ACCEPT, "*/*")
            .header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
            .header(reqwest::header::ACCEPT_ENCODING, "gzip, deflate, br")
            .header(reqwest::header::CONNECTION, "keep-alive")
            .header("X-Forwarded-For", forwarded_ip)
            .header("Client-IP", forwarded_ip)
            .header(reqwest::header::REFERER, &referer)
            .header(
                reqwest::header::ORIGIN,
                format!(
                    "{}://{}",
                    video_url.scheme(),
                    video_url.host_str().unwrap_or_default()
                ),
            )
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "cross-site");

        match request.send().await {
            Ok(result) => {
                if result.status().is_success() {
                    return Ok(result);
                }
                if result.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE
                    && attempt < max_retries
                {
                    last_error = Some(format!("503 on attempt {}", attempt));
                    response = Some(result);
                    continue;
                }
                response = Some(result);
                break;
            }
            Err(error) => {
                last_error = Some(error.to_string());
                if attempt == max_retries {
                    return Err(last_error.unwrap_or_else(|| "Unknown error".to_string()));
                }
            }
        }
    }

    response.ok_or_else(|| {
        format!(
            "Failed after {} attempts: {}",
            max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )
    })
}

fn parse_episodes(play_url: &str) -> Vec<Episode> {
    if play_url.is_empty() {
        return Vec::new();
    }

    play_url
        .split('#')
        .filter(|episode| !episode.is_empty())
        .enumerate()
        .map(|(index, episode)| {
            let mut parts = episode.splitn(2, '$');
            let first = parts.next().unwrap_or_default();
            let second = parts.next();
            let (name, url) = match second {
                Some(url) => (first.to_string(), url.to_string()),
                None => (format!("第 {} 集", index + 1), first.to_string()),
            };

            Episode {
                name: if name.is_empty() {
                    format!("第 {} 集", index + 1)
                } else {
                    name
                },
                url: cleanup_url(&url),
                index,
            }
        })
        .collect()
}

fn cleanup_url(url: &str) -> String {
    let chars = url.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(url.len());
    let mut index = 0usize;

    while index < chars.len() {
        if index + 2 < chars.len()
            && chars[index] != ':'
            && chars[index + 1] == '/'
            && chars[index + 2] == '/'
        {
            output.push(chars[index]);
            output.push('/');
            index += 3;
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

async fn load_ad_filter_settings(state: &AppState, user_id: i64) -> (AdFilterMode, Vec<String>) {
    let settings = match db::get_user_data(&state.db, user_id, "settings").await {
        Ok(raw) => serde_json::from_str::<JsonValue>(&raw).unwrap_or_else(|_| json!({})),
        Err(_) => json!({}),
    };

    let ad_filter_mode = match settings
        .get("adFilterMode")
        .and_then(JsonValue::as_str)
        .unwrap_or_else(|| {
            if settings
                .get("adFilter")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false)
            {
                "heuristic"
            } else {
                "off"
            }
        }) {
        "keyword" => AdFilterMode::Keyword,
        "heuristic" => AdFilterMode::Heuristic,
        "aggressive" => AdFilterMode::Aggressive,
        _ => AdFilterMode::Off,
    };

    let user_keywords = settings
        .get("adKeywords")
        .and_then(JsonValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(JsonValue::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    (
        ad_filter_mode,
        merge_ad_keywords(&state.config.ad_keywords, &user_keywords),
    )
}

fn merge_ad_keywords(env_keywords: &[String], user_keywords: &[String]) -> Vec<String> {
    let mut merged = Vec::new();

    for keyword in env_keywords.iter().chain(user_keywords.iter()) {
        let trimmed = keyword.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !merged.iter().any(|item| item == trimmed) {
            merged.push(trimmed.to_string());
        }
    }

    merged
}

fn is_m3u8_url(target_url: &str) -> bool {
    let has_playlist_suffix = |path: &str| {
        let normalized = path.trim().to_ascii_lowercase();
        normalized.ends_with(".m3u8") || normalized.ends_with(".m3u")
    };

    Url::parse(target_url)
        .ok()
        .map(|url| has_playlist_suffix(url.path()))
        .unwrap_or_else(|| has_playlist_suffix(target_url))
}

#[derive(Clone, Debug)]
struct M3u8SegmentBlock {
    segments: Vec<M3u8Segment>,
    has_cue_tag: bool,
}

#[derive(Clone, Debug)]
struct M3u8Segment {
    url: String,
    duration: f64,
    line_index: usize,
}

#[derive(Clone, Debug)]
struct MainPattern {
    common_prefix: String,
    avg_duration: f64,
    path_prefix: String,
}

const AD_PATH_KEYWORDS: &[&str] = &[
    "advert",
    "preroll",
    "midroll",
    "postroll",
    "dai",
    "vast",
    "ima",
    "adjump",
    "commercial",
    "sponsor",
];

fn parse_m3u8_blocks(lines: &[&str]) -> Vec<M3u8SegmentBlock> {
    let mut blocks = Vec::new();
    let mut current = M3u8SegmentBlock {
        segments: Vec::new(),
        has_cue_tag: false,
    };

    for (index, raw_line) in lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.starts_with("#EXT-X-CUE-OUT") || line.starts_with("#EXT-X-CUE-IN") {
            current.has_cue_tag = true;
        }

        if line == "#EXT-X-DISCONTINUITY" {
            if !current.segments.is_empty() {
                blocks.push(current);
            }
            current = M3u8SegmentBlock {
                segments: Vec::new(),
                has_cue_tag: false,
            };
            continue;
        }

        if let Some(duration_text) = line.strip_prefix("#EXTINF:") {
            let duration = duration_text
                .split(',')
                .next()
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0);
            if let Some(next_line) = lines.get(index + 1) {
                let url = next_line.trim();
                if !url.is_empty() && !url.starts_with('#') {
                    current.segments.push(M3u8Segment {
                        url: url.to_string(),
                        duration,
                        line_index: index + 1,
                    });
                }
            }
        }
    }

    if !current.segments.is_empty() {
        blocks.push(current);
    }

    blocks
}

fn extract_filename(url: &str) -> String {
    url.rsplit('/').next().unwrap_or_default().to_string()
}

fn extract_path_prefix(url: &str) -> String {
    match url.rfind('/') {
        Some(index) => url[..=index].to_string(),
        None => String::new(),
    }
}

fn common_prefix(values: &[String]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };

    let mut prefix = String::new();
    for (index, ch) in first.chars().enumerate() {
        if values
            .iter()
            .all(|value| value.chars().nth(index).is_some_and(|other| other == ch))
        {
            prefix.push(ch);
        } else {
            break;
        }
    }
    prefix
}

fn learn_main_pattern(blocks: &[M3u8SegmentBlock]) -> MainPattern {
    let Some(main_block) = blocks.iter().max_by_key(|block| block.segments.len()) else {
        return MainPattern {
            common_prefix: String::new(),
            avg_duration: 0.0,
            path_prefix: String::new(),
        };
    };

    if main_block.segments.is_empty() {
        return MainPattern {
            common_prefix: String::new(),
            avg_duration: 0.0,
            path_prefix: String::new(),
        };
    }

    let filenames = main_block
        .segments
        .iter()
        .map(|segment| extract_filename(&segment.url))
        .collect::<Vec<_>>();
    let avg_duration = main_block
        .segments
        .iter()
        .map(|segment| segment.duration)
        .sum::<f64>()
        / main_block.segments.len() as f64;

    MainPattern {
        common_prefix: common_prefix(&filenames),
        avg_duration,
        path_prefix: extract_path_prefix(&main_block.segments[0].url),
    }
}

fn score_block(
    block: &M3u8SegmentBlock,
    main_pattern: &MainPattern,
    extra_keywords: &[String],
) -> f64 {
    if block.has_cue_tag {
        return 10.0;
    }

    let mut score = 0.0;

    let safe_keywords = extra_keywords
        .iter()
        .map(|keyword| keyword.trim().to_lowercase())
        .filter(|keyword| keyword.len() > 2)
        .collect::<Vec<_>>();

    for segment in &block.segments {
        let url_lower = segment.url.to_lowercase();
        if AD_PATH_KEYWORDS
            .iter()
            .any(|keyword| url_lower.contains(keyword))
            || safe_keywords
                .iter()
                .any(|keyword| url_lower.contains(keyword))
        {
            score += 2.5;
        }
    }

    if !main_pattern.common_prefix.is_empty()
        && block
            .segments
            .iter()
            .all(|segment| !extract_filename(&segment.url).starts_with(&main_pattern.common_prefix))
    {
        score += 1.5;
    }

    if !main_pattern.path_prefix.is_empty()
        && block
            .segments
            .iter()
            .all(|segment| extract_path_prefix(&segment.url) != main_pattern.path_prefix)
    {
        score += 5.0;
    }

    if main_pattern.avg_duration > 0.0
        && !block.segments.is_empty()
        && block
            .segments
            .iter()
            .all(|segment| segment.duration < main_pattern.avg_duration * 0.6)
    {
        score += 1.0;
    }

    score
}

fn process_m3u8_content(
    content: &str,
    base_url: &str,
    origin: &str,
    ad_filter_mode: AdFilterMode,
    ad_keywords: &[String],
) -> String {
    let Ok(base) = Url::parse(base_url) else {
        return content.to_string();
    };

    let lines = content.lines().collect::<Vec<_>>();
    let mut ad_line_indices = HashSet::new();

    if matches!(
        ad_filter_mode,
        AdFilterMode::Heuristic | AdFilterMode::Aggressive
    ) {
        let blocks = parse_m3u8_blocks(&lines);
        if blocks.len() > 1 {
            let main_pattern = learn_main_pattern(&blocks);
            let threshold = if ad_filter_mode == AdFilterMode::Aggressive {
                3.0
            } else {
                5.0
            };

            for block in &blocks {
                if score_block(block, &main_pattern, ad_keywords) >= threshold {
                    for segment in &block.segments {
                        ad_line_indices.insert(segment.line_index);
                        if segment.line_index > 0 {
                            ad_line_indices.insert(segment.line_index - 1);
                        }
                    }
                }
            }
        }
    }

    let has_keyword_match = !matches!(ad_filter_mode, AdFilterMode::Off)
        && !ad_keywords.is_empty()
        && ad_keywords.iter().any(|keyword| content.contains(keyword));
    let mut processed_lines = Vec::new();
    let mut inside_cue_ad_block = false;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if ad_line_indices.contains(&index) {
            continue;
        }

        if !matches!(ad_filter_mode, AdFilterMode::Off) && trimmed.starts_with("#EXT-X-CUE-OUT") {
            inside_cue_ad_block = true;
            if processed_lines
                .last()
                .is_some_and(|previous: &String| previous.trim() == "#EXT-X-DISCONTINUITY")
            {
                processed_lines.pop();
            }
            continue;
        }

        if !matches!(ad_filter_mode, AdFilterMode::Off) && trimmed.starts_with("#EXT-X-CUE-IN") {
            inside_cue_ad_block = false;
            continue;
        }

        if inside_cue_ad_block {
            continue;
        }

        if has_keyword_match
            && !matches!(ad_filter_mode, AdFilterMode::Off)
            && ad_keywords.iter().any(|keyword| trimmed.contains(keyword))
        {
            while let Some(last) = processed_lines.last() {
                let trimmed_last = last.trim();
                if trimmed_last.starts_with("#EXTINF:") || trimmed_last == "#EXT-X-DISCONTINUITY" {
                    processed_lines.pop();
                } else {
                    break;
                }
            }
            continue;
        }

        if trimmed.starts_with("#EXT-X-KEY:")
            || trimmed.starts_with("#EXT-X-MAP:")
            || trimmed.starts_with("#EXT-X-MEDIA:")
        {
            processed_lines.push(proxy_uri_in_tag(trimmed, &base, origin));
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.contains("/api/proxy") {
            processed_lines.push((*line).to_string());
            continue;
        }

        match base.join(trimmed) {
            Ok(absolute) => processed_lines.push(format!(
                "{origin}/api/proxy?url={}",
                urlencoding::encode(absolute.as_str())
            )),
            Err(_) => processed_lines.push((*line).to_string()),
        }
    }

    processed_lines.join("\n")
}

fn proxy_uri_in_tag(line: &str, base: &Url, origin: &str) -> String {
    let Some(start) = line.find("URI=\"") else {
        return line.to_string();
    };
    let rest = &line[start + 5..];
    let Some(end) = rest.find('"') else {
        return line.to_string();
    };
    let uri = &rest[..end];
    if uri.contains("/api/proxy") {
        return line.to_string();
    }
    match base.join(uri) {
        Ok(absolute) => line.replace(
            &format!("URI=\"{}\"", uri),
            &format!(
                "URI=\"{origin}/api/proxy?url={}\"",
                urlencoding::encode(absolute.as_str())
            ),
        ),
        Err(_) => line.to_string(),
    }
}

fn resolve_source(value: &JsonValue) -> Option<VideoSource> {
    match value {
        JsonValue::Object(_) => serde_json::from_value(value.clone()).ok(),
        JsonValue::String(_) => None,
        _ => None,
    }
}

fn public_user_json(user: db::DbUser) -> JsonValue {
    json!({
        "id": user.id,
        "username": user.username,
        "isAdmin": user.is_admin == 1,
        "disablePremium": user.disable_premium == 1,
        "createdAt": user.created_at.to_string(),
    })
}

fn json_response(status: StatusCode, value: JsonValue) -> Response<Body> {
    let mut response = Json(value).into_response();
    *response.status_mut() = status;
    response
}

fn json_with_headers(
    status: StatusCode,
    value: JsonValue,
    headers: &[(HeaderName, &str)],
) -> Response<Body> {
    let mut response = json_response(status, value);
    for (name, value) in headers {
        insert_header(response.headers_mut(), name.clone(), value);
    }
    response
}

fn text_response(status: StatusCode, body: impl Into<String>) -> Response<Body> {
    text_with_headers(status, body.into(), &[])
}

fn text_with_headers(
    status: StatusCode,
    body: impl Into<String>,
    headers: &[(HeaderName, &str)],
) -> Response<Body> {
    let mut response = Response::new(Body::from(body.into()));
    *response.status_mut() = status;
    for (name, value) in headers {
        insert_header(response.headers_mut(), name.clone(), value);
    }
    response
}

fn insert_header(headers: &mut HeaderMap, name: HeaderName, value: &str) {
    if let Ok(parsed) = HeaderValue::from_str(value) {
        headers.insert(name, parsed);
    }
}

fn copy_upstream_headers(
    source: &reqwest::header::HeaderMap,
    target: &mut HeaderMap,
    strip_stream_headers: bool,
) {
    for (name, value) in source {
        let lower = name.as_str().to_ascii_lowercase();
        if strip_stream_headers
            && ["content-encoding", "content-length", "transfer-encoding"].contains(&lower.as_str())
        {
            continue;
        }
        if let Ok(header_name) = HeaderName::from_bytes(name.as_str().as_bytes())
            && let Ok(header_value) = HeaderValue::from_bytes(value.as_bytes())
        {
            target.insert(header_name, header_value);
        }
    }
}

fn collect_forward_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut output = HashMap::new();
    for key in [header::COOKIE, header::RANGE] {
        if let Some(value) = headers.get(&key).and_then(|value| value.to_str().ok()) {
            output.insert(key.as_str().to_string(), value.to_string());
        }
    }
    output
}

fn convert_status(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

fn guess_origin_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    {
        return Some(origin.to_string());
    }
    if let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    {
        return Some(format!("http://{host}"));
    }
    None
}

fn get_source_name(source_id: &str) -> &str {
    match source_id {
        "zuida" => "最大资源",
        "wujin" => "无尽资源",
        "tianya" => "天涯资源",
        "baofeng" => "暴风资源",
        "jisu" => "极速资源",
        "baiduyun" => "百度云资源",
        "ruyi" => "如意资源",
        "wolong" => "卧龙资源",
        "dytt" => "电影天堂",
        "modu" => "魔都资源",
        "wangwang" => "旺旺资源",
        "hongniu" => "红牛资源",
        "guangsu" => "光速资源",
        "jiangyu" => "鲸鱼资源",
        "sanliuling" => "360资源",
        "haihua" => "海豚资源",
        "wujin2" => "无尽ME",
        "tianyazy" => "天涯海角",
        "guangsu2" => "光速HTTP",
        "youku" => "优酷资源",
        "yilingba" => "1080资源",
        "huya" => "虎牙资源",
        "xinlang" => "新浪资源",
        "ikun" => "iKun资源",
        "lezi" => "乐子资源",
        "xinlang2" => "新浪HTTPS",
        "yilingba2" => "1080JSON",
        "baofeng2" => "暴风APP",
        "wolong2" => "卧龙采集",
        "lezi2" => "乐子HTTP",
        "feifan" => "非凡资源",
        "aidan" => "爱蛋资源",
        "feifanapi" => "非凡API",
        "feifancj" => "非凡采集",
        "feifancj2" => "非凡采集HTTPS",
        "feifan1" => "非凡线路1",
        "moduzy" => "魔都影视",
        "leba" => "乐播资源",
        _ => source_id,
    }
}

fn clean_label(label: &str) -> String {
    label
        .chars()
        .filter(|ch| !matches!(ch, '视' | '频' | '片' | '区' | '专'))
        .collect()
}

fn is_fuzzy_match(left: &str, right: &str) -> bool {
    let left = clean_label(left);
    let right = clean_label(right);
    if left.chars().count() < 4 || right.chars().count() < 4 {
        return left == right;
    }
    let set = left.chars().collect::<HashSet<_>>();
    let overlap = right.chars().filter(|ch| set.contains(ch)).count();
    overlap >= 4
}

fn json_value_to_compact_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(text) => text.clone(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::Bool(boolean) => boolean.to_string(),
        JsonValue::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn json_value_to_u32(value: Option<&JsonValue>) -> Option<u32> {
    value.and_then(|value| match value {
        JsonValue::String(text) => text.parse::<u32>().ok(),
        JsonValue::Number(number) => number.as_u64().map(|value| value as u32),
        _ => None,
    })
}

fn json_value_to_usize(value: Option<&JsonValue>) -> Option<usize> {
    value.and_then(|value| match value {
        JsonValue::String(text) => text.parse::<usize>().ok(),
        JsonValue::Number(number) => number.as_u64().map(|value| value as usize),
        _ => None,
    })
}

fn merge_json_object(left: JsonValue, right: JsonValue) -> JsonValue {
    match (left, right) {
        (JsonValue::Object(mut left), JsonValue::Object(right)) => {
            for (key, value) in right {
                left.insert(key, value);
            }
            JsonValue::Object(left)
        }
        (_, right) => right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_episodes_supports_named_and_unnamed_entries() {
        let episodes = parse_episodes("第1集$https://a.com/1.m3u8#https://a.com/2.m3u8");

        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].name, "第1集");
        assert_eq!(episodes[0].url, "https://a.com/1.m3u8");
        assert_eq!(episodes[1].name, "第 2 集");
        assert_eq!(episodes[1].url, "https://a.com/2.m3u8");
    }

    #[test]
    fn cleanup_url_collapses_duplicate_slashes_without_breaking_protocol() {
        assert_eq!(
            cleanup_url("https://a.com//foo///bar.m3u8"),
            "https://a.com/foo//bar.m3u8"
        );
    }

    #[test]
    fn process_m3u8_content_rewrites_segments_and_keys() {
        let input = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.key\"\nseg-1.ts\n";
        let output = process_m3u8_content(
            input,
            "https://video.example.com/path/index.m3u8",
            "http://localhost:8787",
            AdFilterMode::Off,
            &[],
        );

        assert!(output.contains("/api/proxy?url="));
        assert!(output.contains("key.key"));
        assert!(output.contains("seg-1.ts"));
    }

    #[test]
    fn process_m3u8_content_filters_keyword_matched_segments() {
        let input =
            "#EXTM3U\n#EXTINF:10,\nmain-1.ts\n#EXTINF:5,\nadvert-1.ts\n#EXTINF:10,\nmain-2.ts\n";
        let output = process_m3u8_content(
            input,
            "https://video.example.com/path/index.m3u8",
            "http://localhost:8787",
            AdFilterMode::Keyword,
            &[String::from("advert")],
        );

        assert!(output.contains("main-1.ts"));
        assert!(output.contains("main-2.ts"));
        assert!(!output.contains("advert-1.ts"));
    }

    #[test]
    fn m3u8_url_detection_only_matches_playlist_path_suffix() {
        assert!(is_m3u8_url(
            "https://video.example.com/path/index.m3u8?token=abc"
        ));
        assert!(is_m3u8_url("https://video.example.com/path/live/main.m3u"));
        assert!(!is_m3u8_url(
            "https://video.example.com/path/index.m3u8/0000000.ts"
        ));
        assert!(!is_m3u8_url(
            "https://video.example.com/path/folder.m3u8/segment.ts?foo=bar"
        ));
    }

    #[test]
    fn fuzzy_match_merges_similar_labels() {
        assert!(is_fuzzy_match("动作片", "动作视频"));
        assert!(!is_fuzzy_match("动漫", "综艺"));
    }

    #[test]
    fn source_string_cannot_be_resolved_without_object_payload() {
        assert!(resolve_source(&JsonValue::String("zuida".to_string())).is_none());
    }

    #[test]
    fn merge_ad_keywords_combines_env_and_user_without_duplicates() {
        let merged = merge_ad_keywords(
            &["env-ad".to_string(), "shared".to_string()],
            &["user-ad".to_string(), "shared".to_string(), "".to_string()],
        );

        assert_eq!(merged, vec!["env-ad", "shared", "user-ad"]);
    }

    #[test]
    fn valid_user_data_keys_include_premium_search_history() {
        assert!(VALID_USER_DATA_KEYS.contains(&"premium-search-history"));
    }
}
