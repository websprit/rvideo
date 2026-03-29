use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use serde_json::json;

use crate::types::AppState;

use super::super::super::render::{html_response, render_shell};
use super::super::shared::{
    SearchPageQuery, favorites_key, history_key, load_user_json, require_page_auth,
    require_premium_access,
};
use super::super::view::render_premium_body;

pub(crate) async fn premium_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchPageQuery>,
) -> Response {
    let auth_user = match require_page_auth(&headers, &state, "/premium").await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if let Err(response) = require_premium_access(&auth_user) {
        return response;
    }

    let settings_value = load_user_json(&state, auth_user.id, "settings").await;
    let premium_sources = settings_value
        .get("premiumSources")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let premium_history = load_user_json(&state, auth_user.id, history_key(true)).await;
    let premium_favorites = load_user_json(&state, auth_user.id, favorites_key(true)).await;
    let initial_query = query.q.unwrap_or_default().trim().to_string();

    let body = render_premium_body(
        &premium_sources,
        &initial_query,
        settings_value
            .get("realtimeLatency")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        settings_value
            .get("searchHistory")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        settings_value
            .get("searchDisplayMode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("normal"),
        settings_value
            .get("sortBy")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("default"),
        &premium_history,
        &premium_favorites,
    );

    html_response(render_shell("Premium", Some(&auth_user), "/premium", &body))
}

#[cfg(test)]
mod tests {
    use super::render_premium_body;
    use serde_json::json;

    #[test]
    fn render_premium_body_shows_sources_and_actions() {
        let html = render_premium_body(
            &json!([
                {"id":"premium-a","name":"Premium A","enabled":true}
            ]),
            "庆余年",
            true,
            true,
            "grouped",
            "latency-asc",
            &json!([{ "videoId": "1", "title": "历史 Premium", "source": "premium-a", "premium": true }]),
            &json!([{ "videoId": "2", "title": "收藏 Premium", "source": "premium-a", "premium": true }]),
        );

        assert!(html.contains("刷新内容"));
        assert!(html.contains("Premium 列表"));
        assert!(html.contains("Premium A"));
        assert!(html.contains("premium-state"));
        assert!(html.contains("premium-search-form"));
        assert!(html.contains("premium-search-results"));
        assert!(html.contains("premium-search-display-toggle"));
        assert!(html.contains("premium-search-sort-select"));
        assert!(html.contains("premium-search-filter-toolbar"));
        assert!(html.contains("premium-search-filter-summary"));
        assert!(html.contains("clear-premium-search-filters"));
        assert!(html.contains("premium-search-source-badges"));
        assert!(html.contains("premium-search-type-badges"));
        assert!(html.contains("premium-history-list"));
        assert!(html.contains("premium-search-history-dropdown"));
        assert!(html.contains("\"history\":["));
        assert!(html.contains("\"favorites\":["));
        assert!(html.contains("\"realtimeLatency\":true"));
        assert!(html.contains("\"searchHistory\":true"));
        assert!(html.contains("\"searchDisplayMode\":\"grouped\""));
        assert!(html.contains("\"sortBy\":\"latency-asc\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(html.contains("aria-controls=\"premium-search-history-dropdown\""));
        assert!(html.contains("function replacePremiumSearchUrl(query)"));
        assert!(html.contains("function filterPremiumSearchHistoryItems(query)"));
        assert!(html.contains("function buildPremiumSearchSourceBadges(videos)"));
        assert!(html.contains("function buildPremiumSearchTypeBadges(videos)"));
        assert!(html.contains("function filterPremiumSearchResults(videos)"));
        assert!(html.contains("function premiumSearchUrl(video)"));
        assert!(html.contains("function renderPremiumDisplayToggle()"));
        assert!(html.contains("function sortPremiumVideos(videos, sortBy)"));
        assert!(html.contains("function groupPremiumVideos(videos)"));
        assert!(html.contains("function groupedPremiumDetailUrl(group)"));
        assert!(html.contains("return `/player?${params.toString()}`"));
        assert!(html.contains("params.set('groupedSources'"));
        assert!(html.contains("data-premium-search-source-badge"));
        assert!(html.contains("data-premium-search-type-badge"));
        assert!(html.contains("当前筛选条件下暂无 Premium 结果"));
        assert!(html.contains("const PREMIUM_TAGS_STORAGE_KEY = 'kvideo_premium_custom_tags'"));
        assert!(html.contains("premium-tag-manage-toggle"));
        assert!(html.contains("restore-premium-tags"));
        assert!(html.contains("export-premium-tags"));
        assert!(html.contains("import-premium-tags-trigger"));
        assert!(html.contains("import-premium-tags-merge-trigger"));
        assert!(html.contains("share-premium-tags"));
        assert!(html.contains("share-premium-tags-link"));
        assert!(html.contains("share-premium-tags-link-merge"));
        assert!(html.contains("premium-tags-import-file"));
        assert!(html.contains("function mergePremiumTags(apiTags)"));
        assert!(html.contains("function savePremiumTags(tags)"));
        assert!(html.contains("function normalizePremiumImportedTag(tag)"));
        assert!(html.contains("function buildPremiumTagsExportPayload()"));
        assert!(html.contains("function encodePremiumTagsSharePackage(payload)"));
        assert!(html.contains("function decodePremiumTagsSharePackage(rawValue)"));
        assert!(html.contains("function parsePremiumTagsImportPayload(rawText)"));
        assert!(html.contains("function buildPremiumTagsShareUrl(merge = false)"));
        assert!(html.contains("function applyPremiumTagsImport(rawText, options = {})"));
        assert!(html.contains("async function importPremiumTags(file, options = {})"));
        assert!(html.contains("function exportPremiumTags()"));
        assert!(html.contains("async function copyPremiumTagsSharePackage()"));
        assert!(html.contains("async function copyPremiumTagsShareLink(merge = false)"));
        assert!(html.contains("async function applyPremiumTagShareFromUrl()"));
        assert!(html.contains("function movePremiumTag(tagId, direction)"));
        assert!(html.contains("function removePremiumTag(tagId)"));
        assert!(html.contains("async function restorePremiumTags()"));
        assert!(html.contains("function readStoredPremiumSettings()"));
        assert!(html.contains("function getPremiumSettingsSignature(settings)"));
        assert!(html.contains("function syncPremiumSettings(nextSettings, options = {})"));
        assert!(html.contains("getNormalizedLibrarySort("));
        assert!(html.contains("getLibrarySortLabel("));
        assert!(html.contains("sortLibraryEntries("));
        assert!(html.contains("encodeLibrarySharePackage("));
        assert!(html.contains("decodeLibrarySharePackage("));
        assert!(html.contains("buildLibraryShareUrl("));
        assert!(html.contains("readLibraryShareParams("));
        assert!(html.contains("clearLibraryShareParams("));
        assert!(html.contains("function ensurePremiumLatencyPolling()"));
        assert!(html.contains("function refreshPremiumRealtimeLatencies()"));
        assert!(html.contains("function clearPremiumLoadMoreObserver()"));
        assert!(html.contains("function setupPremiumAutoLoadMore()"));
        assert!(html.contains("function setupPremiumLoadMoreObserver()"));
        assert!(html.contains("IntersectionObserver"));
        assert!(html.contains("/api/ping"));
        assert!(html.contains("premium-search-history-option-"));
        assert!(html.contains("没有匹配的 Premium 搜索历史。"));
        assert!(html.contains("premium-history-clear"));
        assert!(html.contains("open-premium-library-drawer"));
        assert!(html.contains("premium-library-overlay"));
        assert!(html.contains("premium-library-summary"));
        assert!(html.contains("data-premium-library-clear"));
        assert!(html.contains("data-premium-library-selection-toggle"));
        assert!(html.contains("data-premium-library-select-all"));
        assert!(html.contains("data-premium-library-remove-selected"));
        assert!(html.contains("data-premium-library-undo"));
        assert!(html.contains("data-premium-library-copy"));
        assert!(html.contains("data-premium-library-share"));
        assert!(html.contains("data-premium-library-share-link"));
        assert!(html.contains("data-premium-library-share-link-merge"));
        assert!(html.contains("data-premium-library-share-native"));
        assert!(html.contains("data-premium-library-snapshot-save"));
        assert!(html.contains("data-premium-library-snapshot-rename"));
        assert!(html.contains("data-premium-library-snapshot-duplicate"));
        assert!(html.contains("data-premium-library-snapshot-restore"));
        assert!(html.contains("data-premium-library-snapshot-merge"));
        assert!(html.contains("data-premium-library-snapshot-delete"));
        assert!(html.contains("data-premium-library-snapshot-export"));
        assert!(html.contains("data-premium-library-snapshot-share"));
        assert!(html.contains("data-premium-library-snapshot-share-link"));
        assert!(html.contains("data-premium-library-snapshot-share-link-merge"));
        assert!(html.contains("data-premium-library-snapshot-import"));
        assert!(html.contains("data-premium-library-snapshot-import-merge"));
        assert!(html.contains("data-premium-library-export"));
        assert!(html.contains("data-premium-library-import"));
        assert!(html.contains("data-premium-library-import-merge"));
        assert!(html.contains("data-premium-library-import-clipboard"));
        assert!(html.contains("data-premium-library-import-clipboard-merge"));
        assert!(html.contains("data-premium-library-dedupe"));
        assert!(html.contains("data-premium-library-filter"));
        assert!(html.contains("data-premium-library-filter-clear"));
        assert!(html.contains("data-premium-library-sort"));
        assert!(html.contains("premium-library-import-file"));
        assert!(html.contains("premium-library-snapshot-import-file"));
        assert!(html.contains("data-premium-library-main-panel=\"history\""));
        assert!(html.contains("data-premium-library-main-panel=\"favorites\""));
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("aria-modal=\"true\""));
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("aria-controls=\"premium-library-panel-history\""));
        assert!(
            html.contains("function movePremiumLibraryTabFocus(buttons, currentButton, direction)")
        );
        assert!(html.contains("event.key === 'ArrowRight'"));
        assert!(html.contains("event.key === 'Home'"));
        assert!(html.contains("premium-library-toggle"));
        assert!(html.contains("data-open-premium-library"));
        assert!(html.contains("window.addEventListener('kvideo:storage-updated'"));
        assert!(html.contains("data-premium-library-tab=\"favorites\""));
        assert!(html.contains("value=\"庆余年\""));
        assert!(html.contains("Premium 快速访问"));
        assert!(html.contains("历史 Premium"));
        assert!(html.contains("收藏 Premium"));
        assert!(html.contains("data-premium-library-query=\"${escapeHtml(item?.title || '')}\""));
        assert!(html.contains("data-premium-library-remove=\"${kind}:${index}\""));
        assert!(html.contains("data-premium-library-select-item=\"${kind}:${index}\""));
        assert!(html.contains("function persistPremiumLibraryItems(kind)"));
        assert!(html.contains("function getPremiumSelectedSet(kind)"));
        assert!(html.contains("function getSelectedPremiumLibraryEntries(kind)"));
        assert!(html.contains("function getPremiumLibraryActionEntries(kind)"));
        assert!(html.contains(
            "const PREMIUM_LIBRARY_SNAPSHOTS_STORAGE_KEY = 'kvideo-premium-library-snapshots'"
        ));
        assert!(html.contains("function loadPremiumLibrarySnapshots()"));
        assert!(html.contains("function persistPremiumLibrarySnapshots(snapshots)"));
        assert!(html.contains("function getPremiumLibrarySnapshots(kind)"));
        assert!(html.contains(
            "function promptForPremiumSnapshotName(kind, actionLabel, existingSnapshots)"
        ));
        assert!(
            html.contains("function promptSelectPremiumSnapshot(kind, actionLabel, snapshots)")
        );
        assert!(html.contains("async function savePremiumLibrarySnapshot(kind)"));
        assert!(html.contains("function renamePremiumLibrarySnapshot(kind)"));
        assert!(html.contains("function duplicatePremiumLibrarySnapshot(kind)"));
        assert!(html.contains("async function restorePremiumLibrarySnapshot(kind, options = {})"));
        assert!(html.contains("function deletePremiumLibrarySnapshot(kind)"));
        assert!(html.contains("function buildPremiumLibrarySnapshotExportPayload(kind)"));
        assert!(html.contains("function buildPremiumLibrarySnapshotSharePackage(kind)"));
        assert!(html.contains("function exportPremiumLibrarySnapshots(kind)"));
        assert!(html.contains("async function copyPremiumLibrarySnapshotSharePackage(kind)"));
        assert!(
            html.contains(
                "async function copyPremiumLibrarySnapshotShareLink(kind, merge = false)"
            )
        );
        assert!(
            html.contains("async function importPremiumLibrarySnapshots(kind, file, options = {})")
        );
        assert!(html.contains(
            "async function applyPremiumLibrarySnapshotImport(kind, rawText, options = {})"
        ));
        assert!(html.contains("async function applyPremiumLibrarySnapshotShareFromUrl()"));
        assert!(html.contains("function getPremiumLibraryFilter(kind)"));
        assert!(html.contains("function setPremiumLibraryFilter(kind, value)"));
        assert!(html.contains(
            "const PREMIUM_LIBRARY_FILTERS_STORAGE_KEY = 'kvideo-premium-library-filters'"
        ));
        assert!(html.contains("function loadPremiumLibraryFilters()"));
        assert!(html.contains("function persistPremiumLibraryFilters()"));
        assert!(
            html.contains(
                "const PREMIUM_LIBRARY_SORTS_STORAGE_KEY = 'kvideo-premium-library-sorts'"
            )
        );
        assert!(html.contains("function loadPremiumLibrarySorts()"));
        assert!(html.contains("function persistPremiumLibrarySorts()"));
        assert!(html.contains("function updatePremiumLibraryFilterInputs()"));
        assert!(html.contains("function updatePremiumLibrarySortInputs()"));
        assert!(html.contains("function filterPremiumLibraryEntries(kind)"));
        assert!(html.contains("function getSortedPremiumLibraryEntries(kind)"));
        assert!(html.contains("function renderPremiumLibraryCollections()"));
        assert!(html.contains("function setPremiumLibrarySelectionMode(enabled)"));
        assert!(html.contains("function togglePremiumLibraryItemSelection(kind, index)"));
        assert!(html.contains("function selectAllPremiumLibraryItems(kind)"));
        assert!(html.contains("async function removeSelectedPremiumLibraryItems(kind)"));
        assert!(html.contains("async function undoPremiumLibraryRemoval()"));
        assert!(html.contains("function updatePremiumLibraryFilterValue(value)"));
        assert!(html.contains("function updatePremiumLibrarySortValue(value)"));
        assert!(html.contains("function buildPremiumLibraryExportPayload(kind, entries = getPremiumLibraryActionEntries(kind).entries)"));
        assert!(html.contains("function buildPremiumLibraryCopyText(kind, entries = getPremiumLibraryActionEntries(kind).entries)"));
        assert!(html.contains("function buildPremiumLibrarySharePackage(kind)"));
        assert!(html.contains("function decodeLibrarySnapshotSharePackage(rawValue)"));
        assert!(
            html.contains("function buildLibrarySnapshotShareUrl(pathname, kind, merge = false)")
        );
        assert!(html.contains("function readLibrarySnapshotShareParams()"));
        assert!(html.contains("function clearLibrarySnapshotShareParams()"));
        assert!(html.contains("async function copyPremiumLibraryItems(kind)"));
        assert!(html.contains("async function copyPremiumLibrarySharePackage(kind)"));
        assert!(html.contains("async function copyPremiumLibraryShareLink(kind, merge = false)"));
        assert!(html.contains("async function sharePremiumLibraryLink(kind, merge = false)"));
        assert!(html.contains("function exportPremiumLibraryItems(kind)"));
        assert!(html.contains("function normalizeImportedLibrarySnapshot(snapshot)"));
        assert!(html.contains("function parseLibrarySnapshotImportPayload(rawText)"));
        assert!(
            html.contains("async function importPremiumLibraryItems(kind, file, options = {})")
        );
        assert!(
            html.contains("async function applyPremiumLibraryImport(kind, rawText, options = {})")
        );
        assert!(html.contains("async function applyPremiumLibraryShareFromUrl()"));
        assert!(html.contains("async function dedupePremiumLibraryItems(kind)"));
        assert!(html.contains("async function clearPremiumLibraryItems(kind)"));
        assert!(html.contains("async function removePremiumLibraryItem(kind, index)"));
        assert!(html.contains("function triggerPremiumLibrarySearch(query)"));
        assert!(html.contains("function trapPremiumDrawerFocus(event, drawerEl)"));
    }

    #[test]
    fn render_premium_body_embeds_search_history_flag() {
        let html = render_premium_body(
            &json!([]),
            "",
            false,
            false,
            "normal",
            "default",
            &json!([]),
            &json!([]),
        );

        assert!(html.contains("\"searchHistory\":false"));
        assert!(html.contains("function getPremiumSearchHistoryEnabled()"));
        assert!(html.contains("当前已关闭 Premium 搜索历史。"));
    }
}
