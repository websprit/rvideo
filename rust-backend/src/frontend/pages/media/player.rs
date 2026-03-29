use crate::types::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};

use super::super::super::render::{find_source_config, html_response, render_shell};
use super::super::shared::{
    DetailPageQuery, favorites_key, history_key, load_user_json, parse_media_request,
    require_page_auth,
};
use super::super::view::render_player_body;

pub(crate) async fn player_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DetailPageQuery>,
) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/player").await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let media = match parse_media_request(query) {
        Ok(media) => media,
        Err(response) => return response,
    };
    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let source_config = find_source_config(&settings_value, &media.source);
    let history_data_key = history_key(media.is_premium);
    let favorites_data_key = favorites_key(media.is_premium);
    let history_value = load_user_json(&state, auth_user.id, history_data_key).await;
    let favorites_value = load_user_json(&state, auth_user.id, favorites_data_key).await;

    let body = render_player_body(
        &auth_user,
        &media,
        source_config,
        &settings_value,
        history_data_key,
        &history_value,
        favorites_data_key,
        &favorites_value,
    );

    html_response(render_shell("播放器", Some(&auth_user), "/player", &body))
}

#[cfg(test)]
mod tests {
    use super::render_player_body;
    use crate::auth::AuthUser;
    use crate::frontend::pages::shared::MediaRequest;
    use serde_json::json;

    #[test]
    fn render_player_body_includes_controls_and_state_payload() {
        let user = AuthUser {
            id: 1,
            username: "tester".to_string(),
            is_admin: false,
            disable_premium: false,
        };
        let media = MediaRequest {
            video_id: "123".to_string(),
            source: "source-a".to_string(),
            title: "测试播放".to_string(),
            is_premium: false,
            episode: 3,
            grouped_sources: json!([
                {"id":"123","source":"source-a","sourceName":"线路 A"},
                {"id":"456","source":"source-b","sourceName":"线路 B","latency":88}
            ]),
        };

        let html = render_player_body(
            &user,
            &media,
            Some(json!({"id":"source-a"})),
            &json!({"proxyMode":"retry"}),
            "history",
            &json!([{"videoId":"123"}]),
            "favorites",
            &json!([{"videoId":"123"}]),
        );

        assert!(html.contains("后退 10 秒"));
        assert!(html.contains("来源切换"));
        assert!(html.contains("播放侧栏"));
        assert!(html.contains("skip-backward-indicator"));
        assert!(html.contains("skip-forward-indicator"));
        assert!(html.contains("双击左右半区可快退/快进"));
        assert!(html.contains("remote-playback"));
        assert!(html.contains("copy-stream-url"));
        assert!(html.contains("toggle-player-preferences"));
        assert!(html.contains("player-preferences-panel"));
        assert!(html.contains("player-pref-fullscreen-type"));
        assert!(html.contains("player-pref-ad-filter-mode"));
        assert!(html.contains("player-pref-show-mode-indicator"));
        assert!(html.contains("player-pref-auto-next-episode"));
        assert!(html.contains("player-pref-auto-switch-source-on-failure"));
        assert!(html.contains("player-pref-auto-skip-intro"));
        assert!(html.contains("player-pref-skip-intro-seconds"));
        assert!(html.contains("player-pref-auto-skip-outro"));
        assert!(html.contains("player-pref-skip-outro-seconds"));
        assert!(html.contains("player-pref-clear-source-preference"));
        assert!(html.contains("player-source-diagnostics"));
        assert!(html.contains("player-pref-clear-failed-trail"));
        assert!(html.contains("player-pref-copy-source-diagnostics"));
        assert!(html.contains("player-retry-diagnostics"));
        assert!(html.contains("player-pref-force-direct"));
        assert!(html.contains("player-pref-force-proxy"));
        assert!(html.contains("player-pref-reset-retry-state"));
        assert!(html.contains("player-pref-copy-retry-diagnostics"));
        assert!(html.contains("player-pref-copy-playback-diagnostics-json"));
        assert!(html.contains("player-pref-export-playback-diagnostics"));
        assert!(html.contains("player-bookmarks-summary"));
        assert!(html.contains("player-bookmarks-list"));
        assert!(html.contains("player-pref-save-bookmark"));
        assert!(html.contains("player-pref-clear-bookmarks"));
        assert!(html.contains("player-pref-copy-page-link"));
        assert!(html.contains("player-pref-copy-original-link"));
        assert!(html.contains("player-pref-copy-proxy-link"));
        assert!(html.contains("player-pref-copy-active-link"));
        assert!(html.contains("C 复制播放地址"));
        assert!(html.contains("Google Cast"));
        assert!(html.contains("检测到播放卡顿，正在等待缓冲"));
        assert!(html.contains("检测到 HEVC/H.265 编码"));
        assert!(html.contains("HLS 网络错误，正在重试"));
        assert!(html.contains("媒体错误，正在恢复"));
        assert!(html.contains("webkit-playsinline=\"true\""));
        assert!(html.contains("x-webkit-airplay=\"allow\""));
        assert!(html.contains("const isiOSDevice = (() =>"));
        assert!(html.contains("function getEffectiveFullscreenType()"));
        assert!(html.contains("function buildSourceDiagnosticsText()"));
        assert!(html.contains("function updateSourceDiagnostics()"));
        assert!(html.contains("async function copySourceDiagnostics()"));
        assert!(html.contains("function buildRetryDiagnosticsText()"));
        assert!(html.contains("function updateRetryDiagnostics()"));
        assert!(html.contains("async function copyRetryDiagnostics()"));
        assert!(html.contains("function buildPlaybackDiagnosticsPayload()"));
        assert!(html.contains("async function copyPlaybackDiagnosticsJson()"));
        assert!(html.contains("function exportPlaybackDiagnostics()"));
        assert!(html.contains("const PLAYER_BOOKMARKS_STORAGE_KEY = 'kvideo-playback-bookmarks'"));
        assert!(html.contains("function getPlaybackBookmarkKey()"));
        assert!(html.contains("function loadAllPlaybackBookmarks()"));
        assert!(html.contains("function persistAllPlaybackBookmarks(bookmarks)"));
        assert!(html.contains("function getCurrentPlaybackBookmarks()"));
        assert!(html.contains("function buildPlaybackBookmarkLink(bookmark)"));
        assert!(html.contains("function renderPlaybackBookmarks()"));
        assert!(html.contains("function saveCurrentPlaybackBookmark()"));
        assert!(html.contains("function removePlaybackBookmark(bookmarkId)"));
        assert!(html.contains("function clearCurrentPlaybackBookmarks()"));
        assert!(html.contains("function jumpToPlaybackBookmark(bookmarkId)"));
        assert!(html.contains("async function copyPlaybackBookmarkLink(bookmarkId)"));
        assert!(html.contains("function getHashResumePosition()"));
        assert!(html.contains("function setForcedPlaybackMode(useProxy)"));
        assert!(html.contains("function resetRetryDiagnosticsState()"));
        assert!(html.contains("function isLandscapeViewport()"));
        assert!(html.contains("function syncWindowFullscreenLayout()"));
        assert!(html.contains("force-landscape"));
        assert!(html.contains("fullscreenType === 'window'"));
        assert!(html.contains("player-web-fullscreen"));
        assert!(html.contains("function shouldFilterAdsForUrl(url)"));
        assert!(html.contains("(!canUseNativeHls() || shouldFilterAds)"));
        assert!(html.contains("async function prepareNativeHlsPlaybackUrl(url, signal)"));
        assert!(html.contains("正在准备原生 HLS 广告过滤回放"));
        assert!(html.contains("正在通过原生 HLS 加载已过滤视频流"));
        assert!(html.contains("原生 HLS 广告过滤准备超时，已回退到当前过滤流"));
        assert!(html.contains("原生 HLS 广告过滤初始化失败，已回退到当前过滤流"));
        assert!(html.contains("detailState.nativeHlsBlobUrls"));
        assert!(html.contains("nativeHlsAbortController"));
        assert!(html.contains("nativeHlsAbortReason"));
        assert!(html.contains("nativeHlsPrepareTimeoutId"));
        assert!(html.contains("function isAbortError(error)"));
        assert!(html.contains("function clearNativeHlsPrepareTimeout()"));
        assert!(html.contains("credentials: 'same-origin', signal"));
        assert!(html.contains("detailState.playbackRequestToken"));
        assert!(html.contains("const playbackRequestToken = detailState.playbackRequestToken"));
        assert!(html.contains("application/vnd.apple.mpegurl"));
        assert!(html.contains("webkitEnterFullscreen"));
        assert!(html.contains("webkitExitFullscreen"));
        assert!(html.contains("webkitfullscreenchange"));
        assert!(html.contains("function canUseAirPlay()"));
        assert!(html.contains("function hasAirPlayApi()"));
        assert!(html.contains("function canUseWebkitPictureInPicture()"));
        assert!(html.contains("function canUsePictureInPicture()"));
        assert!(html.contains("function isPictureInPictureActive()"));
        assert!(html.contains("async function attemptPlayback(context = 'auto')"));
        assert!(html.contains("function startPlaybackPolling()"));
        assert!(html.contains("function clearPlaybackPolling()"));
        assert!(html.contains("function renderSkipIndicator(direction, amount)"));
        assert!(html.contains("function handleSkipGesture(direction)"));
        assert!(html.contains("function resolveTapSide(clientX)"));
        assert!(html.contains("videoEl?.addEventListener('dblclick'"));
        assert!(html.contains("videoEl?.addEventListener('touchend'"));
        assert!(html.contains("detailState.playbackPollingTimer = window.setInterval"));
        assert!(html.contains("浏览器阻止了自动播放，请手动点击播放"));
        assert!(html.contains("开始播放失败，请尝试重新加载或切换线路"));
        assert!(
            html.contains("当前浏览器不支持 HLS 视频播放，请尝试切换线路或使用支持 HLS 的浏览器")
        );
        assert!(html.contains("webkitShowPlaybackTargetPicker"));
        assert!(html.contains("CAST_STATE_CHANGED"));
        assert!(html.contains("SESSION_STATE_CHANGED"));
        assert!(html.contains("detailState.castingActive"));
        assert!(html.contains("\"groupedSources\":["));
        assert!(html.contains("function getGroupedSources()"));
        assert!(html.contains("function getNavigationGroupedSources()"));
        assert!(html.contains("function mergeInitialGroupedSources()"));
        assert!(html.contains("function persistPlayerSettings()"));
        assert!(
            html.contains("const SOURCE_PREFERENCES_STORAGE_KEY = 'kvideo-source-preferences'")
        );
        assert!(html.contains("function loadFailedSourceTrail()"));
        assert!(html.contains("function getSourcePreferenceKey(title = currentDisplayTitle())"));
        assert!(html.contains("function loadSourcePreferences()"));
        assert!(html.contains("function persistSourcePreferences(preferences)"));
        assert!(
            html.contains("function getPreferredSourceForTitle(title = currentDisplayTitle())")
        );
        assert!(html.contains("function setPreferredSourceForTitle(title, source)"));
        assert!(
            html.contains("function clearPreferredSourceForTitle(title = currentDisplayTitle())")
        );
        assert!(html.contains("function updateSourcePreferenceButton()"));
        assert!(html.contains("function sortSourceResultsWithPreference(results)"));
        assert!(html.contains("function syncPlayerPreferenceControls()"));
        assert!(html.contains("function setPlayerPreferencesPanel(open)"));
        assert!(html.contains("function applyPlayerPreferenceChanges()"));
        assert!(html.contains("function clearFailedSourceTrailFromUrl()"));
        assert!(html.contains("function getAutoSwitchCandidates()"));
        assert!(html.contains("function attemptAutoSwitchSource(message)"));
        assert!(html.contains("params.append('failedSource', item)"));
        assert!(html.contains("window.location.replace(buildSourceSwitchUrl(nextSource, { failedSources: nextFailedSources }))"));
        assert!(html.contains("正在自动切换到"));
        assert!(html.contains("当前 / 偏好"));
        assert!(html.contains("偏好来源"));
        assert!(html.contains("async function copyOriginalPlaybackUrl()"));
        assert!(html.contains("async function copyProxyPlaybackUrl()"));
        assert!(html.contains("async function copyActivePlaybackUrl()"));
        assert!(html.contains("async function copyCurrentPageUrl()"));
        assert!(html.contains("原始播放地址已复制"));
        assert!(html.contains("代理播放地址已复制"));
        assert!(html.contains("当前生效播放链接已复制"));
        assert!(html.contains("已更新广告过滤模式，正在重新加载当前视频"));
        assert!(html.contains("已更新当前播放器偏好"));
        assert!(html.contains("params.set('groupedSources'"));
        assert!(html.contains("分组来源"));
        assert!(html.contains("webkitplaybacktargetavailabilitychanged"));
        assert!(html.contains("webkitSupportsPresentationMode('picture-in-picture')"));
        assert!(html.contains("webkitpresentationmodechanged"));
        assert!(html.contains("已打开 AirPlay 设备选择器"));
        assert!(html.contains("history"));
        assert!(html.contains("favorites"));
        assert!(html.contains("测试播放"));
    }
}
