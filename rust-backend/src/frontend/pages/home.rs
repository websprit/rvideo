use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};

use crate::types::AppState;

use super::super::render::{html_response, render_shell};
use super::shared::{SearchPageQuery, load_user_json, require_page_auth};
use super::view::render_index_body;

pub(super) async fn index_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchPageQuery>,
) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/").await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let history_value = load_user_json(&state, auth_user.id, "history").await;
    let favorites_value = load_user_json(&state, auth_user.id, "favorites").await;
    let initial_query = query.q.unwrap_or_default().trim().to_string();

    let body = render_index_body(
        &auth_user,
        &initial_query,
        &settings_value,
        &history_value,
        &favorites_value,
    );

    html_response(render_shell("首页", Some(&auth_user), "/", &body))
}

#[cfg(test)]
mod tests {
    use super::render_index_body;
    use crate::auth::AuthUser;
    use serde_json::json;

    #[test]
    fn render_index_body_shows_counts_and_saved_lists() {
        let user = AuthUser {
            id: 1,
            username: "tester".to_string(),
            is_admin: false,
            disable_premium: false,
        };
        let html = render_index_body(
            &user,
            "仙逆",
            &json!({
                "sources": [
                    {"id":"a","enabled":true},
                    {"id":"b","enabled":false},
                    {"id":"c"}
                ]
            }),
            &json!([
                {"videoId":"1","title":"历史项","source":"source-a"}
            ]),
            &json!([
                {"videoId":"2","title":"收藏项","source":"source-b"}
            ]),
        );

        assert!(html.contains("输入关键词后会直接在下方显示结果。"));
        assert!(html.contains("value=\"仙逆\""));
        assert!(html.contains("历史项"));
        assert!(html.contains("收藏项"));
        assert!(html.contains("快捷入口"));
        assert!(html.contains("常用入口默认折叠，不占首屏主位。"));
        assert!(html.contains("热门功能"));
        assert!(html.contains("data-quick-query=\"仙逆\""));
        assert!(html.contains("搜索历史"));
        assert!(html.contains("更多工具"));
        assert!(html.contains("search-history-list"));
        assert!(html.contains("search-history-dropdown"));
        assert!(html.contains("role=\"listbox\""));
        assert!(html.contains("aria-controls=\"search-history-dropdown\""));
        assert!(html.contains("function replaceSearchUrl(query)"));
        assert!(html.contains("function filterSearchHistoryItems(query)"));
        assert!(html.contains("search-filter-toolbar"));
        assert!(html.contains("search-filter-summary"));
        assert!(html.contains("clear-search-filters"));
        assert!(html.contains("search-source-badges"));
        assert!(html.contains("search-type-badges"));
        assert!(html.contains("function resultPlayerUrl(video)"));
        assert!(html.contains("return `/player?${params.toString()}`"));
        assert!(html.contains("function buildSearchSourceBadges(videos)"));
        assert!(html.contains("function buildSearchTypeBadges(videos)"));
        assert!(html.contains("function filterSearchResults(videos)"));
        assert!(html.contains("function readStoredSearchSettings()"));
        assert!(html.contains("function getSearchSettingsSignature(settings)"));
        assert!(html.contains("function syncSearchSettings(nextSettings, options = {})"));
        assert!(html.contains("window.addEventListener('kvideo:storage-updated'"));
        assert!(html.contains("groupedSources"));
        assert!(html.contains("data-search-source-badge"));
        assert!(html.contains("data-search-type-badge"));
        assert!(html.contains("当前筛选条件下暂无结果"));
        assert!(html.contains("function ensureLatencyPolling()"));
        assert!(html.contains("function refreshRealtimeLatencies()"));
        assert!(html.contains("/api/ping"));
        assert!(html.contains("search-history-option-"));
        assert!(html.contains("没有匹配的搜索历史。"));
        assert!(html.contains("豆瓣推荐"));
        assert!(html.contains("快速访问"));
        assert!(html.contains("open-home-library-drawer"));
        assert!(html.contains("home-library-overlay"));
        assert!(html.contains("home-state"));
        assert!(html.contains("home-library-summary"));
        assert!(html.contains("data-home-library-clear"));
        assert!(html.contains("data-home-library-selection-toggle"));
        assert!(html.contains("data-home-library-select-all"));
        assert!(html.contains("data-home-library-remove-selected"));
        assert!(html.contains("data-home-library-undo"));
        assert!(html.contains("data-home-library-copy"));
        assert!(html.contains("data-home-library-share"));
        assert!(html.contains("data-home-library-share-link"));
        assert!(html.contains("data-home-library-share-link-merge"));
        assert!(html.contains("data-home-library-share-native"));
        assert!(html.contains("data-home-library-snapshot-save"));
        assert!(html.contains("data-home-library-snapshot-rename"));
        assert!(html.contains("data-home-library-snapshot-duplicate"));
        assert!(html.contains("data-home-library-snapshot-restore"));
        assert!(html.contains("data-home-library-snapshot-merge"));
        assert!(html.contains("data-home-library-snapshot-delete"));
        assert!(html.contains("data-home-library-snapshot-export"));
        assert!(html.contains("data-home-library-snapshot-share"));
        assert!(html.contains("data-home-library-snapshot-share-link"));
        assert!(html.contains("data-home-library-snapshot-share-link-merge"));
        assert!(html.contains("data-home-library-snapshot-import"));
        assert!(html.contains("data-home-library-snapshot-import-merge"));
        assert!(html.contains("data-home-library-export"));
        assert!(html.contains("data-home-library-import"));
        assert!(html.contains("data-home-library-import-merge"));
        assert!(html.contains("data-home-library-import-clipboard"));
        assert!(html.contains("data-home-library-import-clipboard-merge"));
        assert!(html.contains("data-home-library-dedupe"));
        assert!(html.contains("data-home-library-filter"));
        assert!(html.contains("data-home-library-filter-clear"));
        assert!(html.contains("data-home-library-sort"));
        assert!(html.contains("home-library-import-file"));
        assert!(html.contains("home-library-snapshot-import-file"));
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("aria-modal=\"true\""));
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("aria-controls=\"home-library-drawer-panel-history\""));
        assert!(
            html.contains("function moveHomeLibraryTabFocus(buttons, currentButton, direction)")
        );
        assert!(html.contains("event.key === 'ArrowRight'"));
        assert!(html.contains("event.key === 'Home'"));
        assert!(html.contains("home-library-drawer-toggle"));
        assert!(html.contains("data-open-home-library"));
        assert!(html.contains("home-library-toggle"));
        assert!(html.contains("data-library-tab=\"favorites\""));
        assert!(html.contains("data-library-drawer-tab=\"favorites\""));
        assert!(html.contains("data-library-query=\"历史项\""));
        assert!(html.contains("data-home-library-remove=\"${kind}:${index}\""));
        assert!(html.contains("data-home-library-select-item=\"${kind}:${index}\""));
        assert!(html.contains("function persistHomeLibraryItems(kind)"));
        assert!(html.contains("function getHomeSelectedSet(kind)"));
        assert!(html.contains("function getSelectedHomeLibraryEntries(kind)"));
        assert!(html.contains("function getHomeLibraryActionEntries(kind)"));
        assert!(html.contains(
            "const HOME_LIBRARY_SNAPSHOTS_STORAGE_KEY = 'kvideo-home-library-snapshots'"
        ));
        assert!(html.contains("function loadHomeLibrarySnapshots()"));
        assert!(html.contains("function persistHomeLibrarySnapshots(snapshots)"));
        assert!(html.contains("function getHomeLibrarySnapshots(kind)"));
        assert!(
            html.contains("function promptForSnapshotName(kind, actionLabel, existingSnapshots)")
        );
        assert!(html.contains("function promptSelectSnapshot(kind, actionLabel, snapshots)"));
        assert!(html.contains("async function saveHomeLibrarySnapshot(kind)"));
        assert!(html.contains("function renameHomeLibrarySnapshot(kind)"));
        assert!(html.contains("function duplicateHomeLibrarySnapshot(kind)"));
        assert!(html.contains("async function restoreHomeLibrarySnapshot(kind, options = {})"));
        assert!(html.contains("function deleteHomeLibrarySnapshot(kind)"));
        assert!(html.contains("function buildHomeLibrarySnapshotExportPayload(kind)"));
        assert!(html.contains("function buildHomeLibrarySnapshotSharePackage(kind)"));
        assert!(html.contains("function exportHomeLibrarySnapshots(kind)"));
        assert!(html.contains("async function copyHomeLibrarySnapshotSharePackage(kind)"));
        assert!(
            html.contains("async function copyHomeLibrarySnapshotShareLink(kind, merge = false)")
        );
        assert!(
            html.contains("async function importHomeLibrarySnapshots(kind, file, options = {})")
        );
        assert!(html.contains(
            "async function applyHomeLibrarySnapshotImport(kind, rawText, options = {})"
        ));
        assert!(html.contains("async function applyHomeLibrarySnapshotShareFromUrl()"));
        assert!(html.contains("function getHomeLibraryFilter(kind)"));
        assert!(html.contains("function setHomeLibraryFilter(kind, value)"));
        assert!(
            html.contains("const HOME_LIBRARY_FILTERS_STORAGE_KEY = 'kvideo-home-library-filters'")
        );
        assert!(html.contains("function loadHomeLibraryFilters()"));
        assert!(html.contains("function persistHomeLibraryFilters()"));
        assert!(
            html.contains("const HOME_LIBRARY_SORTS_STORAGE_KEY = 'kvideo-home-library-sorts'")
        );
        assert!(html.contains("function loadHomeLibrarySorts()"));
        assert!(html.contains("function persistHomeLibrarySorts()"));
        assert!(html.contains("function updateHomeLibraryFilterInputs()"));
        assert!(html.contains("function updateHomeLibrarySortInputs()"));
        assert!(html.contains("function filterHomeLibraryEntries(kind)"));
        assert!(html.contains("function sortLibraryEntries(entries, sortBy)"));
        assert!(html.contains("function getSortedHomeLibraryEntries(kind)"));
        assert!(html.contains("function renderHomeLibraryCollections()"));
        assert!(html.contains("function setHomeLibrarySelectionMode(enabled)"));
        assert!(html.contains("function toggleHomeLibraryItemSelection(kind, index)"));
        assert!(html.contains("function selectAllHomeLibraryItems(kind)"));
        assert!(html.contains("async function removeSelectedHomeLibraryItems(kind)"));
        assert!(html.contains("async function undoHomeLibraryRemoval()"));
        assert!(html.contains("function updateHomeLibraryFilterValue(value)"));
        assert!(html.contains("function updateHomeLibrarySortValue(value)"));
        assert!(html.contains("function buildHomeLibraryExportPayload(kind, entries = getHomeLibraryActionEntries(kind).entries)"));
        assert!(html.contains("function buildHomeLibraryCopyText(kind, entries = getHomeLibraryActionEntries(kind).entries)"));
        assert!(html.contains("function encodeLibrarySharePackage(payload)"));
        assert!(html.contains("function decodeLibrarySharePackage(rawValue)"));
        assert!(html.contains("function decodeLibrarySnapshotSharePackage(rawValue)"));
        assert!(html.contains("function buildHomeLibrarySharePackage(kind)"));
        assert!(html.contains("function buildLibraryShareUrl(pathname, kind, merge = false)"));
        assert!(
            html.contains("function buildLibrarySnapshotShareUrl(pathname, kind, merge = false)")
        );
        assert!(html.contains("function readLibraryShareParams()"));
        assert!(html.contains("function readLibrarySnapshotShareParams()"));
        assert!(html.contains("function clearLibraryShareParams()"));
        assert!(html.contains("function clearLibrarySnapshotShareParams()"));
        assert!(html.contains("async function copyHomeLibraryItems(kind)"));
        assert!(html.contains("async function copyHomeLibrarySharePackage(kind)"));
        assert!(html.contains("async function copyHomeLibraryShareLink(kind, merge = false)"));
        assert!(html.contains("async function shareHomeLibraryLink(kind, merge = false)"));
        assert!(html.contains("function exportHomeLibraryItems(kind)"));
        assert!(html.contains("function parseLibraryImportPayload(rawText)"));
        assert!(html.contains("function normalizeImportedLibrarySnapshot(snapshot)"));
        assert!(html.contains("function parseLibrarySnapshotImportPayload(rawText)"));
        assert!(html.contains("function normalizeImportedLibraryItem(item)"));
        assert!(html.contains("function dedupeLibraryItems(items)"));
        assert!(html.contains("async function importHomeLibraryItems(kind, file, options = {})"));
        assert!(
            html.contains("async function applyHomeLibraryImport(kind, rawText, options = {})")
        );
        assert!(html.contains("async function applyHomeLibraryShareFromUrl()"));
        assert!(html.contains("async function readClipboardImportText(label)"));
        assert!(html.contains("async function dedupeHomeLibraryItems(kind)"));
        assert!(html.contains("async function clearHomeLibraryItems(kind)"));
        assert!(html.contains("async function removeHomeLibraryItem(kind, index)"));
        assert!(html.contains("function trapDrawerFocus(event, drawerEl)"));
    }

    #[test]
    fn render_index_body_hides_premium_entries_for_premium_disabled_user() {
        let user = AuthUser {
            id: 2,
            username: "basic".to_string(),
            is_admin: false,
            disable_premium: true,
        };
        let html = render_index_body(&user, "", &json!({}), &json!([]), &json!([]));

        assert!(html.contains("Premium 已禁用"));
        assert!(!html.contains(r#"href="/premium""#));
    }
}
