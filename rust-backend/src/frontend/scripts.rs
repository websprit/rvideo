pub(super) const SEARCH_SCRIPT: &str = r#"
const initialQuery = JSON.parse(document.getElementById('initial-query').textContent || '""');
const initialSettings = JSON.parse(document.getElementById('initial-settings').textContent || '{}');
const homeState = JSON.parse(document.getElementById('home-state')?.textContent || '{}');
const searchForm = document.getElementById('search-form');
const searchInput = document.getElementById('search-query');
const clearButton = document.getElementById('clear-search');
const statusEl = document.getElementById('search-status');
const progressEl = document.getElementById('search-progress');
const totalEl = document.getElementById('search-total');
const resultsEl = document.getElementById('search-results');
const historyListEl = document.getElementById('search-history-list');
const historyDropdownEl = document.getElementById('search-history-dropdown');
const historyCountEl = document.getElementById('search-history-count');
const clearHistoryButton = document.getElementById('clear-search-history');
const displayToggleEl = document.getElementById('search-display-toggle');
const sortSelectEl = document.getElementById('search-sort-select');
const searchFilterToolbarEl = document.getElementById('search-filter-toolbar');
const searchFilterSummaryEl = document.getElementById('search-filter-summary');
const clearSearchFiltersButton = document.getElementById('clear-search-filters');
const searchSourceBadgesEl = document.getElementById('search-source-badges');
const searchTypeBadgesEl = document.getElementById('search-type-badges');
const homeLibraryToggleEl = document.getElementById('home-library-toggle');
const homeLibraryStatusEls = document.querySelectorAll('[data-home-library-status]');
const homeLibraryPanelEls = document.querySelectorAll('[data-library-panel]');
const homeLibraryDrawerToggleEl = document.getElementById('home-library-drawer-toggle');
const homeLibraryDrawerPanelEls = document.querySelectorAll('[data-library-drawer-panel]');
const homeLibraryDrawerOverlayEl = document.getElementById('home-library-overlay');
const homeLibraryDrawerEl = document.getElementById('home-library-drawer');
const openHomeLibraryDrawerButtons = document.querySelectorAll('[data-open-home-library]');
const closeHomeLibraryDrawerButton = document.getElementById('close-home-library-drawer');
const homeLibrarySummaryEl = document.getElementById('home-library-summary');
const homeLibraryFilterInputs = document.querySelectorAll('[data-home-library-filter]');
const clearHomeLibraryFilterButtons = document.querySelectorAll('[data-home-library-filter-clear]');
const homeLibrarySortInputs = document.querySelectorAll('[data-home-library-sort]');
const clearHomeLibraryButtons = document.querySelectorAll('[data-home-library-clear]');
const toggleHomeLibrarySelectionButtons = document.querySelectorAll('[data-home-library-selection-toggle]');
const selectAllHomeLibraryButtons = document.querySelectorAll('[data-home-library-select-all]');
const removeSelectedHomeLibraryButtons = document.querySelectorAll('[data-home-library-remove-selected]');
const undoHomeLibraryButtons = document.querySelectorAll('[data-home-library-undo]');
const copyHomeLibraryButtons = document.querySelectorAll('[data-home-library-copy]');
const shareHomeLibraryButtons = document.querySelectorAll('[data-home-library-share]');
const shareLinkHomeLibraryButtons = document.querySelectorAll('[data-home-library-share-link]');
const shareLinkMergeHomeLibraryButtons = document.querySelectorAll('[data-home-library-share-link-merge]');
const nativeShareHomeLibraryButtons = document.querySelectorAll('[data-home-library-share-native]');
const saveHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-save]');
const renameHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-rename]');
const duplicateHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-duplicate]');
const restoreHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-restore]');
const mergeHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-merge]');
const deleteHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-delete]');
const exportHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-export]');
const shareHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-share]');
const shareLinkHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-share-link]');
const shareLinkMergeHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-share-link-merge]');
const importHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-import]');
const mergeImportHomeLibrarySnapshotButtons = document.querySelectorAll('[data-home-library-snapshot-import-merge]');
const exportHomeLibraryButtons = document.querySelectorAll('[data-home-library-export]');
const importHomeLibraryButtons = document.querySelectorAll('[data-home-library-import]');
const mergeImportHomeLibraryButtons = document.querySelectorAll('[data-home-library-import-merge]');
const clipboardImportHomeLibraryButtons = document.querySelectorAll('[data-home-library-import-clipboard]');
const clipboardMergeImportHomeLibraryButtons = document.querySelectorAll('[data-home-library-import-clipboard-merge]');
const dedupeHomeLibraryButtons = document.querySelectorAll('[data-home-library-dedupe]');
const homeLibraryImportFileInput = document.getElementById('home-library-import-file');
const homeLibrarySnapshotImportFileInput = document.getElementById('home-library-snapshot-import-file');
const SEARCH_HISTORY_KEY = 'kvideo-search-history';
const SEARCH_LATENCY_INTERVAL_MS = 5000;
const HOME_LIBRARY_FILTERS_STORAGE_KEY = 'kvideo-home-library-filters';
const HOME_LIBRARY_SORTS_STORAGE_KEY = 'kvideo-home-library-sorts';
const HOME_LIBRARY_SNAPSHOTS_STORAGE_KEY = 'kvideo-home-library-snapshots';

const searchState = {
  rawResults: [],
  displayMode: initialSettings.searchDisplayMode === 'grouped' ? 'grouped' : 'normal',
  sortBy: typeof initialSettings.sortBy === 'string' ? initialSettings.sortBy : 'default',
  settingsSignature: '',
  libraryTab: 'history',
  librarySelectionMode: false,
  historyItems: Array.isArray(homeState.history) ? homeState.history : [],
  favoriteItems: Array.isArray(homeState.favorites) ? homeState.favorites : [],
  selectedLibraryItems: {
    history: new Set(),
    favorites: new Set(),
  },
  libraryFilters: loadHomeLibraryFilters(),
  librarySorts: loadHomeLibrarySorts(),
  libraryUndo: null,
  historyDropdownOpen: false,
  historyHighlightedIndex: -1,
  historyBlurTimer: 0,
  historySubmitLockUntil: 0,
  drawerLastFocused: null,
  drawerTrapHandler: null,
  liveLatencies: {},
  sourceBaseUrls: {},
  latencyTimer: 0,
  latencyRequestId: 0,
  selectedSources: new Set(),
  selectedTypes: new Set(),
};

function setSearchStatus(kind, message) {
  statusEl.textContent = message;
  statusEl.className = `status ${kind}`;
}

function getEnabledSources(settings) {
  if (!settings || typeof settings !== 'object' || !Array.isArray(settings.sources)) {
    return [];
  }
  return settings.sources.filter((source) => source && typeof source.id === 'string' && source.enabled !== false);
}

function readStoredSearchSettings() {
  if (typeof window === 'undefined') {
    return {};
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem('kvideo-settings') || '{}');
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
  } catch (_) {
    return {};
  }
}

function getSearchSettingsSignature(settings) {
  const enabledSources = getEnabledSources(settings).map((source) => ({
    id: String(source.id || ''),
    enabled: source.enabled !== false,
    baseUrl: String(source.baseUrl || ''),
  }));
  return JSON.stringify({
    sources: enabledSources,
    searchHistory: settings?.searchHistory !== false,
    realtimeLatency: settings?.realtimeLatency === true,
    searchDisplayMode: settings?.searchDisplayMode === 'grouped' ? 'grouped' : 'normal',
    sortBy: typeof settings?.sortBy === 'string' ? settings.sortBy : 'default',
  });
}

function syncSearchSettings(nextSettings, options = {}) {
  const normalized = nextSettings && typeof nextSettings === 'object' && !Array.isArray(nextSettings)
    ? nextSettings
    : {};
  const previousSignature = searchState.settingsSignature;
  Object.keys(initialSettings).forEach((key) => delete initialSettings[key]);
  Object.assign(initialSettings, normalized);
  searchState.settingsSignature = getSearchSettingsSignature(initialSettings);
  searchState.displayMode = initialSettings.searchDisplayMode === 'grouped' ? 'grouped' : 'normal';
  searchState.sortBy = typeof initialSettings.sortBy === 'string' ? initialSettings.sortBy : 'default';
  renderSearchHistory();
  if (sortSelectEl instanceof HTMLSelectElement) {
    sortSelectEl.value = searchState.sortBy;
  }
  applySearchPresentation();

  const shouldRerun = options.rerun !== false
    && previousSignature
    && previousSignature !== searchState.settingsSignature
    && String(searchInput?.value || '').trim();
  if (shouldRerun) {
    runSearch(searchInput.value).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
      progressEl.textContent = '搜索失败';
    });
    return;
  }

  if (!String(searchInput?.value || '').trim()) {
    setSearchStatus('muted', '输入关键词后会直接在下方显示结果。');
  }
}

function buildHomeLibraryPlayerUrl(item) {
  const videoId = item?.videoId ?? item?.vod_id;
  const source = item?.source;
  const title = item?.title || '未知视频';
  if (!videoId || !source) {
    return '/player';
  }
  const params = new URLSearchParams({
    id: String(videoId),
    source: String(source),
    title: String(title),
  });
  const episodeIndex = Number(item?.episodeIndex);
  if (Number.isInteger(episodeIndex) && episodeIndex >= 0) {
    params.set('episode', String(episodeIndex));
  }
  return `/player?${params.toString()}`;
}

function buildHomeLibraryDetailUrl(item) {
  const videoId = item?.videoId ?? item?.vod_id;
  const source = item?.source;
  const title = item?.title || '未知视频';
  if (!videoId || !source) {
    return '/detail';
  }
  const params = new URLSearchParams({
    id: String(videoId),
    source: String(source),
    title: String(title),
  });
  return `/detail?${params.toString()}`;
}

function getHomeLibraryItems(kind) {
  return kind === 'favorites' ? searchState.favoriteItems : searchState.historyItems;
}

function setHomeLibraryItems(kind, items) {
  if (kind === 'favorites') {
    searchState.favoriteItems = items;
  } else {
    searchState.historyItems = items;
  }
  getHomeSelectedSet(kind).clear();
}

function getHomeSelectedSet(kind) {
  return kind === 'favorites'
    ? searchState.selectedLibraryItems.favorites
    : searchState.selectedLibraryItems.history;
}

function getHomeLibraryStorageKey(kind) {
  return kind === 'favorites' ? 'kvideo-favorites-store' : 'kvideo-history-store';
}

function getHomeLibraryDataKey(kind) {
  return kind === 'favorites' ? 'favorites' : 'history';
}

function getHomeLibraryLabel(kind) {
  return kind === 'favorites' ? '收藏' : '历史';
}

function getHomeLibraryFilter(kind) {
  return kind === 'favorites'
    ? searchState.libraryFilters.favorites
    : searchState.libraryFilters.history;
}

function setHomeLibraryFilter(kind, value) {
  if (kind === 'favorites') {
    searchState.libraryFilters.favorites = value;
  } else {
    searchState.libraryFilters.history = value;
  }
}

function getNormalizedLibrarySort(value) {
  switch (String(value || '')) {
    case 'recent-asc':
    case 'title-asc':
    case 'title-desc':
    case 'source-asc':
    case 'source-desc':
      return String(value);
    case 'recent-desc':
    default:
      return 'recent-desc';
  }
}

function getLibrarySortLabel(value) {
  switch (getNormalizedLibrarySort(value)) {
    case 'recent-asc':
      return '最早添加';
    case 'title-asc':
      return '标题 A-Z';
    case 'title-desc':
      return '标题 Z-A';
    case 'source-asc':
      return '来源 A-Z';
    case 'source-desc':
      return '来源 Z-A';
    case 'recent-desc':
    default:
      return '最近添加';
  }
}

function getHomeLibrarySort(kind) {
  return kind === 'favorites'
    ? searchState.librarySorts.favorites
    : searchState.librarySorts.history;
}

function setHomeLibrarySort(kind, value) {
  const nextSort = getNormalizedLibrarySort(value);
  if (kind === 'favorites') {
    searchState.librarySorts.favorites = nextSort;
  } else {
    searchState.librarySorts.history = nextSort;
  }
}

function loadHomeLibraryFilters() {
  if (typeof window === 'undefined') {
    return { history: '', favorites: '' };
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(HOME_LIBRARY_FILTERS_STORAGE_KEY) || '{}');
    return {
      history: typeof parsed?.history === 'string' ? parsed.history : '',
      favorites: typeof parsed?.favorites === 'string' ? parsed.favorites : '',
    };
  } catch (_) {
    return { history: '', favorites: '' };
  }
}

function persistHomeLibraryFilters() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(HOME_LIBRARY_FILTERS_STORAGE_KEY, JSON.stringify({
    history: searchState.libraryFilters.history,
    favorites: searchState.libraryFilters.favorites,
  }));
}

function loadHomeLibrarySorts() {
  if (typeof window === 'undefined') {
    return { history: 'recent-desc', favorites: 'recent-desc' };
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(HOME_LIBRARY_SORTS_STORAGE_KEY) || '{}');
    return {
      history: getNormalizedLibrarySort(parsed?.history),
      favorites: getNormalizedLibrarySort(parsed?.favorites),
    };
  } catch (_) {
    return { history: 'recent-desc', favorites: 'recent-desc' };
  }
}

function persistHomeLibrarySorts() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(HOME_LIBRARY_SORTS_STORAGE_KEY, JSON.stringify({
    history: getHomeLibrarySort('history'),
    favorites: getHomeLibrarySort('favorites'),
  }));
}

function updateHomeLibraryFilterInputs() {
  const value = getHomeLibraryFilter(searchState.libraryTab);
  homeLibraryFilterInputs.forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.value = value;
    }
  });
  clearHomeLibraryFilterButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = !value;
    }
  });
}

function updateHomeLibrarySortInputs() {
  const value = getHomeLibrarySort(searchState.libraryTab);
  homeLibrarySortInputs.forEach((input) => {
    if (input instanceof HTMLSelectElement) {
      input.value = value;
    }
  });
}

function filterHomeLibraryEntries(kind) {
  const filterText = String(getHomeLibraryFilter(kind) || '').trim().toLowerCase();
  const items = getHomeLibraryItems(kind).map((item, index) => ({ item, index }));
  if (!filterText) {
    return items;
  }
  return items.filter(({ item }) => {
    const title = String(item?.title || '').toLowerCase();
    const source = String(item?.sourceName || item?.source || '').toLowerCase();
    return title.includes(filterText) || source.includes(filterText);
  });
}

function sortLibraryEntries(entries, sortBy) {
  const normalizedSort = getNormalizedLibrarySort(sortBy);
  return [...entries].sort((left, right) => {
    const leftTitle = String(left?.item?.title || '').trim();
    const rightTitle = String(right?.item?.title || '').trim();
    const leftSource = String(left?.item?.sourceName || left?.item?.source || '').trim();
    const rightSource = String(right?.item?.sourceName || right?.item?.source || '').trim();

    switch (normalizedSort) {
      case 'recent-asc':
        return left.index - right.index;
      case 'title-asc':
        return leftTitle.localeCompare(rightTitle, 'zh-CN') || left.index - right.index;
      case 'title-desc':
        return rightTitle.localeCompare(leftTitle, 'zh-CN') || left.index - right.index;
      case 'source-asc':
        return leftSource.localeCompare(rightSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'source-desc':
        return rightSource.localeCompare(leftSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'recent-desc':
      default:
        return right.index - left.index;
    }
  });
}

function getSortedHomeLibraryEntries(kind) {
  return sortLibraryEntries(filterHomeLibraryEntries(kind), getHomeLibrarySort(kind));
}

function getSelectedHomeLibraryEntries(kind) {
  const selectedItems = getHomeSelectedSet(kind);
  if (!selectedItems.size) {
    return [];
  }
  return getHomeLibraryItems(kind)
    .map((item, index) => ({ item, index }))
    .filter(({ index }) => selectedItems.has(index));
}

function getHomeLibraryActionEntries(kind) {
  const selectedEntries = getSelectedHomeLibraryEntries(kind);
  if (selectedEntries.length) {
    return {
      entries: selectedEntries,
      usingSelection: true,
      selectedCount: selectedEntries.length,
    };
  }
  return {
    entries: getHomeLibraryItems(kind).map((item, index) => ({ item, index })),
    usingSelection: false,
    selectedCount: 0,
  };
}

function loadHomeLibrarySnapshots() {
  if (typeof window === 'undefined') {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(HOME_LIBRARY_SNAPSHOTS_STORAGE_KEY) || '[]');
    return Array.isArray(parsed) ? parsed.filter((snapshot) =>
      snapshot
      && typeof snapshot === 'object'
      && typeof snapshot.name === 'string'
      && (snapshot.kind === 'history' || snapshot.kind === 'favorites')
      && Array.isArray(snapshot.items)
    ) : [];
  } catch (_) {
    return [];
  }
}

function persistHomeLibrarySnapshots(snapshots) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(HOME_LIBRARY_SNAPSHOTS_STORAGE_KEY, JSON.stringify(snapshots));
}

function getHomeLibrarySnapshots(kind) {
  return loadHomeLibrarySnapshots().filter((snapshot) => snapshot.kind === kind);
}

function promptForSnapshotName(kind, actionLabel, existingSnapshots) {
  const existingNames = existingSnapshots.map((snapshot) => snapshot.name).join('、');
  const message = existingNames
    ? `${actionLabel}${getHomeLibraryLabel(kind)}快照。\n已有快照：${existingNames}\n请输入快照名称`
    : `${actionLabel}${getHomeLibraryLabel(kind)}快照。\n请输入快照名称`;
  return String(window.prompt(message, '') || '').trim();
}

function promptSelectSnapshot(kind, actionLabel, snapshots) {
  if (!snapshots.length) {
    throw new Error(`当前还没有保存的${getHomeLibraryLabel(kind)}快照`);
  }
  const names = snapshots.map((snapshot, index) => `${index + 1}. ${snapshot.name}`).join('\n');
  const rawInput = String(window.prompt(`${actionLabel}${getHomeLibraryLabel(kind)}快照：\n${names}\n请输入快照名称或序号`, '') || '').trim();
  if (!rawInput) {
    throw new Error('未选择快照');
  }
  const index = Number.parseInt(rawInput, 10);
  if (Number.isInteger(index) && index >= 1 && index <= snapshots.length) {
    return snapshots[index - 1];
  }
  const normalized = rawInput.toLowerCase();
  const matched = snapshots.find((snapshot) => snapshot.name.toLowerCase() === normalized || snapshot.name === rawInput);
  if (!matched) {
    throw new Error('没有匹配的快照');
  }
  return matched;
}

async function saveHomeLibrarySnapshot(kind) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可保存的${getHomeLibraryLabel(kind)}内容`);
  }
  const snapshots = loadHomeLibrarySnapshots();
  const name = promptForSnapshotName(kind, action.usingSelection ? '保存已选' : '保存当前', snapshots.filter((snapshot) => snapshot.kind === kind));
  if (!name) {
    throw new Error('未输入快照名称');
  }
  const nextSnapshot = {
    name,
    kind,
    savedAt: new Date().toISOString(),
    items: action.entries.map(({ item }) => item),
  };
  const nextSnapshots = snapshots.filter((snapshot) => !(snapshot.kind === kind && snapshot.name === name));
  nextSnapshots.unshift(nextSnapshot);
  persistHomeLibrarySnapshots(nextSnapshots.slice(0, 20));
  renderHomeLibraryCollections();
}

function renameHomeLibrarySnapshot(kind) {
  const snapshots = getHomeLibrarySnapshots(kind);
  const snapshot = promptSelectSnapshot(kind, '重命名', snapshots);
  const name = promptForSnapshotName(
    kind,
    '重命名',
    snapshots.filter((item) => item.name !== snapshot.name)
  );
  if (!name) {
    throw new Error('未输入新的快照名称');
  }
  const nextSnapshots = loadHomeLibrarySnapshots().map((item) => (
    item.kind === kind && item.name === snapshot.name
      ? { ...item, name }
      : item
  ));
  persistHomeLibrarySnapshots(nextSnapshots);
  renderHomeLibraryCollections();
  return { previousName: snapshot.name, name };
}

function duplicateHomeLibrarySnapshot(kind) {
  const snapshots = getHomeLibrarySnapshots(kind);
  const snapshot = promptSelectSnapshot(kind, '克隆', snapshots);
  const name = promptForSnapshotName(kind, '克隆', snapshots);
  if (!name) {
    throw new Error('未输入新的快照名称');
  }
  const nextSnapshot = {
    ...snapshot,
    name,
    savedAt: new Date().toISOString(),
    items: snapshot.items.map((item) => ({ ...item })),
  };
  const nextSnapshots = loadHomeLibrarySnapshots().filter((item) => !(item.kind === kind && item.name === name));
  nextSnapshots.unshift(nextSnapshot);
  persistHomeLibrarySnapshots(nextSnapshots.slice(0, 20));
  renderHomeLibraryCollections();
  return nextSnapshot;
}

async function restoreHomeLibrarySnapshot(kind, options = {}) {
  const snapshots = getHomeLibrarySnapshots(kind);
  const snapshot = promptSelectSnapshot(kind, options.merge ? '合并' : '恢复', snapshots);
  const previousItems = getHomeLibraryItems(kind).slice();
  const nextItems = options.merge
    ? dedupeLibraryItems([...previousItems, ...snapshot.items])
    : snapshot.items.slice();
  setHomeLibraryItems(kind, nextItems);
  searchState.libraryUndo = { kind, items: previousItems };
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
  return snapshot;
}

function deleteHomeLibrarySnapshot(kind) {
  const snapshots = getHomeLibrarySnapshots(kind);
  const snapshot = promptSelectSnapshot(kind, '删除', snapshots);
  if (!window.confirm(`确定删除快照「${snapshot.name}」？`)) {
    return null;
  }
  const nextSnapshots = loadHomeLibrarySnapshots().filter((item) => !(item.kind === kind && item.name === snapshot.name));
  persistHomeLibrarySnapshots(nextSnapshots);
  renderHomeLibraryCollections();
  return snapshot;
}

function buildHomeLibrarySnapshotExportPayload(kind) {
  return {
    format: 'kvideo-library-snapshots-export',
    version: 1,
    exportedAt: new Date().toISOString(),
    page: 'home',
    kind,
    snapshots: getHomeLibrarySnapshots(kind),
  };
}

function buildHomeLibrarySnapshotSharePackage(kind) {
  const payload = buildHomeLibrarySnapshotExportPayload(kind);
  return `kvideo://library-snapshots/${encodeLibrarySharePackage(payload)}`;
}

function exportHomeLibrarySnapshots(kind) {
  const snapshots = getHomeLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可导出的${getHomeLibraryLabel(kind)}快照`);
  }
  const payload = buildHomeLibrarySnapshotExportPayload(kind);
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-home-${kind}-snapshots-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

async function copyHomeLibrarySnapshotSharePackage(kind) {
  const snapshots = getHomeLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可分享的${getHomeLibraryLabel(kind)}快照`);
  }
  const text = buildHomeLibrarySnapshotSharePackage(kind);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function copyHomeLibrarySnapshotShareLink(kind, merge = false) {
  const snapshots = getHomeLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可分享的${getHomeLibraryLabel(kind)}快照`);
  }
  const text = buildLibrarySnapshotShareUrl('/', kind, merge);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function importHomeLibrarySnapshots(kind, file, options = {}) {
  const rawText = await file.text();
  return applyHomeLibrarySnapshotImport(kind, rawText, options);
}

async function applyHomeLibrarySnapshotImport(kind, rawText, options = {}) {
  const parsed = parseLibrarySnapshotImportPayload(rawText);
  const snapshots = parsed.snapshots.filter((snapshot) => snapshot.kind === kind);
  if (!snapshots.length) {
    throw new Error(`导入文件中没有${getHomeLibraryLabel(kind)}快照`);
  }
  const existingSnapshots = loadHomeLibrarySnapshots();
  const retainedSnapshots = existingSnapshots.filter((snapshot) => snapshot.kind !== kind);
  const nextKindSnapshots = options.merge
    ? [
        ...snapshots,
        ...existingSnapshots.filter((snapshot) =>
          snapshot.kind === kind && !snapshots.some((item) => item.name === snapshot.name)
        ),
      ]
    : snapshots;
  persistHomeLibrarySnapshots([...retainedSnapshots, ...nextKindSnapshots].slice(0, 40));
  renderHomeLibraryCollections();
}

async function applyHomeLibrarySnapshotShareFromUrl() {
  const shareParams = readLibrarySnapshotShareParams();
  if (!shareParams) {
    return;
  }
  const parsed = parseLibrarySnapshotImportPayload(shareParams.rawValue);
  const targetKind = parsed.snapshots[0]?.kind === 'favorites' ? 'favorites' : 'history';
  await applyHomeLibrarySnapshotImport(targetKind, shareParams.rawValue, {
    merge: shareParams.merge,
  });
  clearLibrarySnapshotShareParams();
  setSearchStatus('success', shareParams.merge
    ? `已从分享链接合并导入${getHomeLibraryLabel(targetKind)}快照`
    : `已从分享链接导入${getHomeLibraryLabel(targetKind)}快照`);
}

async function persistHomeLibraryItems(kind) {
  const items = getHomeLibraryItems(kind);
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(getHomeLibraryStorageKey(kind), JSON.stringify(items));
  }
  const response = await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: getHomeLibraryDataKey(kind), value: items }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error || `保存${kind === 'favorites' ? '收藏' : '历史'}失败`);
  }
}

function buildHomeLibraryExportPayload(kind, entries = getHomeLibraryActionEntries(kind).entries) {
  return {
    format: 'kvideo-library-export',
    version: 1,
    exportedAt: new Date().toISOString(),
    page: 'home',
    kind,
    items: entries.map(({ item }) => ({
      raw: item,
      videoId: item?.videoId ?? item?.vod_id ?? '',
      title: item?.title || '未知视频',
      source: item?.source || '',
      sourceName: item?.sourceName || item?.source || '',
      premium: item?.premium === true,
      episodeIndex: Number.isInteger(Number(item?.episodeIndex)) ? Number(item.episodeIndex) : null,
      playerUrl: buildHomeLibraryPlayerUrl(item),
      detailUrl: buildHomeLibraryDetailUrl(item),
    })),
  };
}

function buildHomeLibraryCopyText(kind, entries = getHomeLibraryActionEntries(kind).entries) {
  const label = getHomeLibraryLabel(kind);
  const lines = [`# 首页${label}清单`, ''];
  entries.forEach(({ item }, index) => {
    const title = item?.title || '未知视频';
    const source = item?.sourceName || item?.source || '未知来源';
    lines.push(`${index + 1}. ${title} [${source}]`);
    lines.push(`   播放: ${window.location.origin}${buildHomeLibraryPlayerUrl(item)}`);
    lines.push(`   详情: ${window.location.origin}${buildHomeLibraryDetailUrl(item)}`);
  });
  return lines.join('\n');
}

function encodeLibrarySharePackage(payload) {
  const json = JSON.stringify(payload);
  const utf8 = encodeURIComponent(json).replace(/%([0-9A-F]{2})/g, (_, hex) =>
    String.fromCharCode(Number.parseInt(hex, 16))
  );
  return btoa(utf8).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/g, '');
}

function decodeLibrarySharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://library/',
    'kvideo://library-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  const json = decodeURIComponent(percentEncoded);
  return JSON.parse(json);
}

function decodeLibrarySnapshotSharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://library-snapshots/',
    'kvideo://library-snapshots-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('快照分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  const json = decodeURIComponent(percentEncoded);
  return JSON.parse(json);
}

function buildHomeLibrarySharePackage(kind) {
  const payload = buildHomeLibraryExportPayload(kind);
  return `kvideo://library/${encodeLibrarySharePackage(payload)}`;
}

function buildLibraryShareUrl(pathname, kind, merge = false) {
  const sharePackage = pathname === '/premium'
    ? buildPremiumLibrarySharePackage(kind)
    : buildHomeLibrarySharePackage(kind);
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  return `${window.location.origin}${pathname}?${params.toString()}`;
}

function readLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  const libraryShare = String(params.get('libraryShare') || '').trim();
  if (!libraryShare) {
    return null;
  }
  return {
    rawValue: libraryShare,
    merge: params.get('libraryShareMode') === 'merge',
  };
}

function buildLibrarySnapshotShareUrl(pathname, kind, merge = false) {
  const sharePackage = pathname === '/premium'
    ? buildPremiumLibrarySnapshotSharePackage(kind)
    : buildHomeLibrarySnapshotSharePackage(kind);
  const params = new URLSearchParams();
  params.set('librarySnapshotsShare', sharePackage);
  if (merge) {
    params.set('librarySnapshotsShareMode', 'merge');
  }
  return `${window.location.origin}${pathname}?${params.toString()}`;
}

function readLibrarySnapshotShareParams() {
  const params = new URLSearchParams(window.location.search);
  const rawValue = String(params.get('librarySnapshotsShare') || '').trim();
  if (!rawValue) {
    return null;
  }
  return {
    rawValue,
    merge: params.get('librarySnapshotsShareMode') === 'merge',
  };
}

function clearLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('libraryShare');
  params.delete('libraryShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function clearLibrarySnapshotShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('librarySnapshotsShare');
  params.delete('librarySnapshotsShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

async function copyHomeLibraryItems(kind) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可复制的${getHomeLibraryLabel(kind)}内容`);
  }
  const text = buildHomeLibraryCopyText(kind, action.entries);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function copyHomeLibrarySharePackage(kind) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getHomeLibraryLabel(kind)}内容`);
  }
  const text = `kvideo://library/${encodeLibrarySharePackage(buildHomeLibraryExportPayload(kind, action.entries))}`;
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function copyHomeLibraryShareLink(kind, merge = false) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getHomeLibraryLabel(kind)}内容`);
  }
  const sharePackage = `kvideo://library/${encodeLibrarySharePackage(buildHomeLibraryExportPayload(kind, action.entries))}`;
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  const text = `${window.location.origin}/?${params.toString()}`;
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function shareHomeLibraryLink(kind, merge = false) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getHomeLibraryLabel(kind)}内容`);
  }
  const sharePackage = `kvideo://library/${encodeLibrarySharePackage(buildHomeLibraryExportPayload(kind, action.entries))}`;
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  const url = `${window.location.origin}/?${params.toString()}`;
  if (navigator.share) {
    await navigator.share({
      title: `${getHomeLibraryLabel(kind)}分享`,
      text: merge
        ? `分享当前${getHomeLibraryLabel(kind)}列表，打开后会合并导入`
        : `分享当前${getHomeLibraryLabel(kind)}列表`,
      url,
    });
    return;
  }
  await copyHomeLibraryShareLink(kind, merge);
}

function exportHomeLibraryItems(kind) {
  const action = getHomeLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可导出的${getHomeLibraryLabel(kind)}内容`);
  }
  const payload = buildHomeLibraryExportPayload(kind, action.entries);
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-home-${kind}-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function normalizeImportedLibraryItem(item) {
  if (!item || typeof item !== 'object') {
    return null;
  }
  if (item.raw && typeof item.raw === 'object') {
    return item.raw;
  }
  const videoId = item.videoId ?? item.vod_id ?? '';
  const normalized = {
    videoId: videoId ? String(videoId) : '',
    title: String(item.title || '未知视频'),
    source: String(item.source || ''),
    sourceName: String(item.sourceName || item.source || ''),
  };
  if (item.premium === true) {
    normalized.premium = true;
  }
  const episodeIndex = Number(item.episodeIndex);
  if (Number.isInteger(episodeIndex) && episodeIndex >= 0) {
    normalized.episodeIndex = episodeIndex;
  }
  return normalized;
}

function getLibraryItemIdentity(item) {
  const source = String(item?.source || '').trim();
  const videoId = String(item?.videoId ?? item?.vod_id ?? '').trim();
  const title = String(item?.title || '').trim().toLowerCase();
  const episodeIndex = Number.isInteger(Number(item?.episodeIndex)) ? Number(item.episodeIndex) : -1;
  return [source, videoId || title, episodeIndex].join('::');
}

function dedupeLibraryItems(items) {
  const seen = new Set();
  return items.filter((item) => {
    const identity = getLibraryItemIdentity(item);
    if (!identity || seen.has(identity)) {
      return false;
    }
    seen.add(identity);
    return true;
  });
}

function parseLibraryImportPayload(rawText) {
  const sharedPayload = decodeLibrarySharePackage(rawText);
  let parsed;
  if (sharedPayload) {
    parsed = sharedPayload;
  } else {
    try {
      parsed = JSON.parse(rawText);
    } catch (_) {
      throw new Error('导入内容不是有效的 JSON 或分享包');
    }
  }
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.items)) {
    throw new Error('导入文件缺少 items 列表');
  }
  const items = parsed.items
    .map((item) => normalizeImportedLibraryItem(item))
    .filter((item) => item && item.title && item.source);
  if (!items.length) {
    throw new Error('导入文件中没有可用条目');
  }
  return { kind: parsed.kind, items };
}

function normalizeImportedLibrarySnapshot(snapshot) {
  if (!snapshot || typeof snapshot !== 'object') {
    return null;
  }
  const kind = snapshot.kind === 'favorites' ? 'favorites' : snapshot.kind === 'history' ? 'history' : null;
  const name = String(snapshot.name || '').trim();
  if (!kind || !name || !Array.isArray(snapshot.items)) {
    return null;
  }
  const items = snapshot.items
    .map((item) => normalizeImportedLibraryItem(item))
    .filter((item) => item && item.title && item.source);
  if (!items.length) {
    return null;
  }
  return {
    name,
    kind,
    savedAt: typeof snapshot.savedAt === 'string' ? snapshot.savedAt : new Date().toISOString(),
    items,
  };
}

function parseLibrarySnapshotImportPayload(rawText) {
  const sharedPayload = decodeLibrarySnapshotSharePackage(rawText);
  let parsed;
  if (sharedPayload) {
    parsed = sharedPayload;
  } else {
    try {
      parsed = JSON.parse(rawText);
    } catch (_) {
      throw new Error('快照导入内容不是有效的 JSON');
    }
  }
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.snapshots)) {
    throw new Error('快照导入文件缺少 snapshots 列表');
  }
  const snapshots = parsed.snapshots
    .map((snapshot) => normalizeImportedLibrarySnapshot(snapshot))
    .filter(Boolean);
  if (!snapshots.length) {
    throw new Error('快照导入文件中没有可用快照');
  }
  return {
    page: typeof parsed.page === 'string' ? parsed.page : '',
    snapshots,
  };
}

async function importHomeLibraryItems(kind, file, options = {}) {
  const rawText = await file.text();
  return applyHomeLibraryImport(kind, rawText, options);
}

async function applyHomeLibraryImport(kind, rawText, options = {}) {
  const parsed = parseLibraryImportPayload(rawText);
  const targetKind = options.targetKind || (parsed.kind === 'favorites' ? 'favorites' : kind);
  const { items } = parsed;
  const previousItems = getHomeLibraryItems(targetKind).slice();
  const nextItems = options.merge
    ? dedupeLibraryItems([...previousItems, ...items])
    : items;
  setHomeLibraryItems(targetKind, nextItems);
  searchState.libraryUndo = { kind: targetKind, items: previousItems };
  await persistHomeLibraryItems(targetKind);
  renderHomeLibraryCollections();
}

async function applyHomeLibraryShareFromUrl() {
  const shareParams = readLibraryShareParams();
  if (!shareParams) {
    return;
  }
  const parsed = parseLibraryImportPayload(shareParams.rawValue);
  const targetKind = parsed.kind === 'favorites' ? 'favorites' : 'history';
  await applyHomeLibraryImport(targetKind, shareParams.rawValue, {
    merge: shareParams.merge,
    targetKind,
  });
  clearLibraryShareParams();
  setSearchStatus('success', shareParams.merge
    ? `已从分享链接合并导入${getHomeLibraryLabel(targetKind)}列表，可撤销`
    : `已从分享链接导入${getHomeLibraryLabel(targetKind)}列表，可撤销`);
}

async function readClipboardImportText(label) {
  if (navigator.clipboard?.readText) {
    const text = await navigator.clipboard.readText();
    if (String(text || '').trim()) {
      return text;
    }
  }
  const fallback = window.prompt(`请粘贴要导入的 ${label} JSON 内容`);
  if (!String(fallback || '').trim()) {
    throw new Error('没有可导入的文本内容');
  }
  return fallback;
}

async function dedupeHomeLibraryItems(kind) {
  const previousItems = getHomeLibraryItems(kind).slice();
  const nextItems = dedupeLibraryItems(previousItems);
  if (nextItems.length === previousItems.length) {
    throw new Error(`当前${getHomeLibraryLabel(kind)}列表没有重复项`);
  }
  setHomeLibraryItems(kind, nextItems);
  searchState.libraryUndo = { kind, items: previousItems };
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
}

function renderHomeLibrarySummary() {
  if (homeLibrarySummaryEl instanceof HTMLElement) {
    const selectedCount = getHomeSelectedSet(searchState.libraryTab).size;
    const visibleCount = getSortedHomeLibraryEntries(searchState.libraryTab).length;
    const totalCount = getHomeLibraryItems(searchState.libraryTab).length;
    const snapshotCount = getHomeLibrarySnapshots(searchState.libraryTab).length;
    const parts = [`历史 ${searchState.historyItems.length}`, `收藏 ${searchState.favoriteItems.length}`];
    if (visibleCount !== totalCount) {
      parts.push(`当前显示 ${visibleCount}/${totalCount}`);
    }
    parts.push(`排序 ${getLibrarySortLabel(getHomeLibrarySort(searchState.libraryTab))}`);
    parts.push(`快照 ${snapshotCount}`);
    if (selectedCount > 0) {
      parts.push(`已选 ${selectedCount}`);
    }
    if (searchState.libraryUndo && Array.isArray(searchState.libraryUndo.items)) {
      parts.push(`可撤销 ${searchState.libraryUndo.items.length}`);
    }
    homeLibrarySummaryEl.textContent = parts.join(' / ');
  }
}

function renderHomeLibraryCollection(kind) {
  const items = getSortedHomeLibraryEntries(kind);
  const selectedItems = getHomeSelectedSet(kind);
  const panels = [
    ...Array.from(homeLibraryPanelEls),
    ...Array.from(homeLibraryDrawerPanelEls),
  ].filter((panel) => panel instanceof HTMLElement && (
    panel.dataset.libraryPanel === kind || panel.dataset.libraryDrawerPanel === kind
  ));

  const emptyMessage = kind === 'favorites'
    ? '当前没有可快速访问的收藏内容。'
    : '当前没有可快速访问的历史记录。';

  panels.forEach((panel) => {
    if (!(panel instanceof HTMLElement)) {
      return;
    }
    if (!items.length) {
      panel.classList.add('empty-state');
      panel.innerHTML = getHomeLibraryFilter(kind)
        ? `没有匹配“${escapeHtml(getHomeLibraryFilter(kind))}”的${kind === 'favorites' ? '收藏' : '历史'}。`
        : emptyMessage;
      return;
    }

    panel.classList.remove('empty-state');
    panel.innerHTML = items.slice(0, 12).map(({ item, index }) => `
      <article class="saved-item library-entry ${searchState.librarySelectionMode && selectedItems.has(index) ? 'selected' : ''}">
        <div class="stack compact-card-body">
          <strong>${escapeHtml(item?.title || '未知视频')}</strong>
          <div class="source-item-meta">
            <span>${escapeHtml(item?.sourceName || item?.source || '未知来源')}</span>
            <span>${kind === 'favorites'
              ? escapeHtml(item?.type || item?.remarks || '收藏内容')
              : escapeHtml(item?.episodes?.[item?.episodeIndex]?.name || `第${Number(item?.episodeIndex ?? 0) + 1}集`)}</span>
          </div>
        </div>
        <div class="row wrap gap-sm library-actions">
          ${searchState.librarySelectionMode ? `
            <button
              class="button button-small ${selectedItems.has(index) ? 'primary' : ''}"
              type="button"
              data-home-library-select-item="${kind}:${index}"
              aria-pressed="${selectedItems.has(index) ? 'true' : 'false'}"
            >${selectedItems.has(index) ? '取消选择' : '选择'}</button>
          ` : ''}
          <a class="button button-small" href="${escapeHtml(buildHomeLibraryPlayerUrl(item))}">播放</a>
          <a class="button button-small" href="${escapeHtml(buildHomeLibraryDetailUrl(item))}">详情</a>
          <button class="button button-small" type="button" data-library-query="${escapeHtml(item?.title || '')}">搜同名</button>
          <button class="button button-small danger" type="button" data-home-library-remove="${kind}:${index}">移除</button>
        </div>
      </article>
    `).join('');
  });
}

function renderHomeLibraryCollections() {
  renderHomeLibrarySummary();
  renderHomeLibraryCollection('history');
  renderHomeLibraryCollection('favorites');
  updateHomeLibraryFilterInputs();
  updateHomeLibrarySortInputs();
  const currentItems = getHomeLibraryItems(searchState.libraryTab);
  const selectedCount = getHomeSelectedSet(searchState.libraryTab).size;
  clearHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = searchState.libraryTab === 'favorites' ? '清空收藏' : '清空历史';
    }
  });
  toggleHomeLibrarySelectionButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = searchState.librarySelectionMode ? '取消批量' : '批量选择';
      button.setAttribute('aria-pressed', searchState.librarySelectionMode ? 'true' : 'false');
    }
  });
  selectAllHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0 || !searchState.librarySelectionMode;
      button.textContent = selectedCount === currentItems.length && currentItems.length > 0
        ? '已全选'
        : '全选当前列表';
    }
  });
  removeSelectedHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = selectedCount === 0 || !searchState.librarySelectionMode;
      button.textContent = selectedCount > 0 ? `移除已选 (${selectedCount})` : '移除已选';
    }
  });
  undoHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      const undoCount = Array.isArray(searchState.libraryUndo?.items) ? searchState.libraryUndo.items.length : 0;
      button.disabled = undoCount === 0;
      button.textContent = undoCount > 0 ? `撤销上次移除 (${undoCount})` : '撤销上次移除';
    }
  });
  copyHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选 (${selectedCount})`
        : `复制当前${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
  exportHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `导出已选 (${selectedCount})`
        : `导出当前${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
  shareHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选分享包 (${selectedCount})`
        : `复制${getHomeLibraryLabel(searchState.libraryTab)}分享包`;
    }
  });
  shareLinkHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选分享链接 (${selectedCount})`
        : `复制${getHomeLibraryLabel(searchState.libraryTab)}分享链接`;
    }
  });
  shareLinkMergeHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选合并分享链接 (${selectedCount})`
        : `复制${getHomeLibraryLabel(searchState.libraryTab)}合并分享链接`;
    }
  });
  nativeShareHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = navigator.share
        ? (selectedCount > 0
          ? `分享已选 (${selectedCount})`
          : `分享当前${getHomeLibraryLabel(searchState.libraryTab)}`)
        : (selectedCount > 0
          ? `复制已选分享链接 (${selectedCount})`
          : `复制${getHomeLibraryLabel(searchState.libraryTab)}分享链接`);
    }
  });
  saveHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `保存已选快照 (${selectedCount})`
        : `保存${getHomeLibraryLabel(searchState.libraryTab)}快照`;
    }
  });
  const snapshotCount = getHomeLibrarySnapshots(searchState.libraryTab).length;
  renameHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `重命名快照 (${snapshotCount})`
        : '重命名快照';
    }
  });
  duplicateHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `克隆快照 (${snapshotCount})`
        : '克隆快照';
    }
  });
  restoreHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `恢复快照 (${snapshotCount})`
        : '恢复快照';
    }
  });
  mergeHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `合并快照 (${snapshotCount})`
        : '合并快照';
    }
  });
  deleteHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `删除快照 (${snapshotCount})`
        : '删除快照';
    }
  });
  exportHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `导出快照 (${snapshotCount})`
        : '导出快照';
    }
  });
  shareHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照分享包 (${snapshotCount})`
        : '复制快照分享包';
    }
  });
  shareLinkHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照分享链接 (${snapshotCount})`
        : '复制快照分享链接';
    }
  });
  shareLinkMergeHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照合并链接 (${snapshotCount})`
        : '复制快照合并链接';
    }
  });
  importHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = '导入快照';
    }
  });
  mergeImportHomeLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = '合并导入快照';
    }
  });
  mergeImportHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `合并导入${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
  clipboardImportHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `剪贴板导入${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
  clipboardMergeImportHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `剪贴板合并${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
  dedupeHomeLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length < 2;
      button.textContent = `去重当前${getHomeLibraryLabel(searchState.libraryTab)}`;
    }
  });
}

async function clearHomeLibraryItems(kind) {
  const previousItems = getHomeLibraryItems(kind).slice();
  if (!window.confirm(`确定清空当前${kind === 'favorites' ? '收藏' : '历史'}列表？`)) {
    return;
  }
  setHomeLibraryItems(kind, []);
  searchState.libraryUndo = { kind, items: previousItems };
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
  setSearchStatus('success', `已清空${kind === 'favorites' ? '收藏' : '历史'}，可撤销`);
}

function setHomeLibrarySelectionMode(enabled) {
  searchState.librarySelectionMode = Boolean(enabled);
  if (!searchState.librarySelectionMode) {
    getHomeSelectedSet('history').clear();
    getHomeSelectedSet('favorites').clear();
  }
  renderHomeLibraryCollections();
}

function updateHomeLibraryFilterValue(value) {
  setHomeLibraryFilter(searchState.libraryTab, String(value || '').trim());
  persistHomeLibraryFilters();
  renderHomeLibraryCollections();
}

function updateHomeLibrarySortValue(value) {
  setHomeLibrarySort(searchState.libraryTab, value);
  persistHomeLibrarySorts();
  renderHomeLibraryCollections();
}

function toggleHomeLibraryItemSelection(kind, index) {
  const items = getHomeLibraryItems(kind);
  if (!items[index]) {
    return;
  }
  const selectedItems = getHomeSelectedSet(kind);
  if (selectedItems.has(index)) {
    selectedItems.delete(index);
  } else {
    selectedItems.add(index);
  }
  renderHomeLibraryCollections();
}

function selectAllHomeLibraryItems(kind) {
  const items = getHomeLibraryItems(kind);
  const selectedItems = getHomeSelectedSet(kind);
  selectedItems.clear();
  items.forEach((_, index) => selectedItems.add(index));
  renderHomeLibraryCollections();
}

async function removeSelectedHomeLibraryItems(kind) {
  const selectedItems = getHomeSelectedSet(kind);
  if (!selectedItems.size) {
    return;
  }
  const selectedCount = selectedItems.size;
  const previousItems = getHomeLibraryItems(kind).slice();
  if (!window.confirm(`确定移除当前${kind === 'favorites' ? '收藏' : '历史'}列表中的 ${selectedItems.size} 项？`)) {
    return;
  }
  const items = getHomeLibraryItems(kind).filter((_, index) => !selectedItems.has(index));
  setHomeLibraryItems(kind, items);
  searchState.libraryUndo = { kind, items: previousItems };
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
  setSearchStatus('success', `已批量移除 ${selectedCount} 项，可撤销`);
}

async function removeHomeLibraryItem(kind, index) {
  const items = getHomeLibraryItems(kind).slice();
  const item = items[index];
  if (!item) {
    return;
  }
  if (!window.confirm(`确定移除「${item.title || '该条目'}」？`)) {
    return;
  }
  const previousItems = items.slice();
  items.splice(index, 1);
  setHomeLibraryItems(kind, items);
  searchState.libraryUndo = { kind, items: previousItems };
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
  setSearchStatus('success', `已移除「${item.title || '该条目'}」，可撤销`);
}

async function undoHomeLibraryRemoval() {
  const undoState = searchState.libraryUndo;
  if (!undoState || !Array.isArray(undoState.items)) {
    return;
  }
  const kind = undoState.kind === 'favorites' ? 'favorites' : 'history';
  setHomeLibraryItems(kind, undoState.items.slice());
  searchState.libraryUndo = null;
  await persistHomeLibraryItems(kind);
  renderHomeLibraryCollections();
  setSearchStatus('success', '已撤销上次移除');
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function resultPlayerUrl(video) {
  const params = new URLSearchParams({
    id: String(video.vod_id ?? ''),
    source: String(video.source ?? ''),
    title: String(video.vod_name ?? '未知视频'),
  });
  return `/player?${params.toString()}`;
}

function replaceSearchUrl(query) {
  const params = new URLSearchParams(window.location.search);
  if (query) {
    params.set('q', query);
  } else {
    params.delete('q');
  }
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function getSearchHistoryEnabled() {
  return initialSettings.searchHistory !== false;
}

function loadSearchHistory() {
  if (typeof window === 'undefined' || !getSearchHistoryEnabled()) {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(SEARCH_HISTORY_KEY) || '[]');
    return Array.isArray(parsed) ? parsed.filter((item) =>
      item
      && typeof item === 'object'
      && typeof item.query === 'string'
      && item.query.trim()
    ) : [];
  } catch (_) {
    return [];
  }
}

function saveSearchHistory(items) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(items.slice(0, 20)));
}

function filterSearchHistoryItems(query) {
  const normalized = String(query || '').trim().toLowerCase();
  const items = loadSearchHistory();
  if (!normalized) {
    return items.slice(0, 10);
  }
  return items
    .filter((item) => String(item.query || '').trim().toLowerCase().includes(normalized))
    .slice(0, 10);
}

function updateSearchHistory(query, resultCount) {
  if (!getSearchHistoryEnabled()) {
    renderSearchHistory();
    return;
  }

  const trimmedQuery = String(query || '').trim();
  if (!trimmedQuery) {
    return;
  }

  const normalized = trimmedQuery.toLowerCase();
  const nextItem = {
    query: trimmedQuery,
    timestamp: Date.now(),
    resultCount: Number.isFinite(resultCount) ? resultCount : undefined,
  };
  const history = loadSearchHistory();
  const deduped = history.filter((item) => String(item.query || '').trim().toLowerCase() !== normalized);
  deduped.unshift(nextItem);
  saveSearchHistory(deduped);
  renderSearchHistory();
}

function removeSearchHistoryItem(query) {
  const normalized = String(query || '').trim().toLowerCase();
  const nextHistory = loadSearchHistory().filter((item) =>
    String(item.query || '').trim().toLowerCase() !== normalized
  );
  saveSearchHistory(nextHistory);
  renderSearchHistory();
}

function clearSearchHistory() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.removeItem(SEARCH_HISTORY_KEY);
  renderSearchHistory();
}

function renderSearchHistory() {
  if (!historyListEl || !historyCountEl) {
    return;
  }

  if (!getSearchHistoryEnabled()) {
    historyCountEl.textContent = '历史关闭';
    historyListEl.className = 'saved-list empty-state';
    historyListEl.textContent = '当前已关闭搜索历史，可在设置页重新启用。';
    if (clearHistoryButton instanceof HTMLButtonElement) {
      clearHistoryButton.disabled = true;
    }
    return;
  }

  const history = loadSearchHistory();
  historyCountEl.textContent = `历史 ${history.length}`;
  if (clearHistoryButton instanceof HTMLButtonElement) {
    clearHistoryButton.disabled = history.length === 0;
  }

  if (!history.length) {
    historyListEl.className = 'saved-list empty-state';
    historyListEl.textContent = '当前还没有搜索历史。';
    return;
  }

  historyListEl.className = 'saved-list';
  historyListEl.innerHTML = history.slice(0, 10).map((item, index) => `
    <article class="saved-item library-item">
      <div class="row space-between wrap gap-sm">
        <button class="button result-card-button" type="button" data-search-history-query="${escapeHtml(item.query)}">
          <div class="library-item-main">
            <strong>${escapeHtml(item.query)}</strong>
            <span class="muted tiny">
              ${typeof item.resultCount === 'number' ? `上次结果 ${item.resultCount} 条` : '点击再次搜索'}
              · 第 ${index + 1} 条
            </span>
          </div>
        </button>
        <button class="button danger button-small" type="button" data-search-history-remove="${escapeHtml(item.query)}">移除</button>
      </div>
    </article>
  `).join('');

  renderSearchHistoryDropdown();
}

function setSearchHistoryDropdown(open) {
  searchState.historyDropdownOpen = open;
  if (searchInput instanceof HTMLInputElement) {
    searchInput.setAttribute('aria-expanded', open ? 'true' : 'false');
    if (!open) {
      searchInput.removeAttribute('aria-activedescendant');
    }
  }
  if (!(historyDropdownEl instanceof HTMLElement)) {
    return;
  }
  historyDropdownEl.classList.toggle('hidden', !open);
  historyDropdownEl.hidden = !open;
  historyDropdownEl.setAttribute('aria-hidden', open ? 'false' : 'true');
}

function renderSearchHistoryDropdown() {
  if (!(historyDropdownEl instanceof HTMLElement)) {
    return;
  }

  const allHistory = loadSearchHistory().slice(0, 10);
  const history = filterSearchHistoryItems(searchInput?.value || '');
  if (!searchState.historyDropdownOpen) {
    historyDropdownEl.classList.add('hidden');
    historyDropdownEl.hidden = true;
    return;
  }

  historyDropdownEl.classList.remove('hidden');
  historyDropdownEl.hidden = false;

  if (!history.length) {
    if (searchInput instanceof HTMLInputElement) {
      searchInput.removeAttribute('aria-activedescendant');
    }
    historyDropdownEl.innerHTML = `
      <div class="search-history-dropdown-header">
        <strong>搜索历史</strong>
      </div>
      <div class="empty-state">${allHistory.length ? '没有匹配的搜索历史。' : '当前还没有搜索历史。'}</div>
    `;
    return;
  }

  historyDropdownEl.innerHTML = `
    <div class="search-history-dropdown-header">
      <strong>搜索历史</strong>
      <button class="button button-small" type="button" data-search-history-clear-all>清空</button>
    </div>
    <div class="search-history-dropdown-list">
      ${history.map((item, index) => `
        <div id="search-history-option-${index}" class="search-history-dropdown-item ${index === searchState.historyHighlightedIndex ? 'active' : ''}" role="option" aria-selected="${index === searchState.historyHighlightedIndex ? 'true' : 'false'}">
          <button class="button result-card-button" type="button" data-search-history-query="${escapeHtml(item.query)}">
            <div class="search-history-dropdown-main">
              <strong>${escapeHtml(item.query)}</strong>
              <span class="muted tiny">${typeof item.resultCount === 'number' ? `上次结果 ${item.resultCount} 条` : '点击再次搜索'}</span>
            </div>
          </button>
          <button class="button danger button-small" type="button" data-search-history-remove="${escapeHtml(item.query)}">移除</button>
        </div>
      `).join('')}
    </div>
  `;
  if (searchInput instanceof HTMLInputElement) {
    if (searchState.historyHighlightedIndex >= 0 && history[searchState.historyHighlightedIndex]) {
      searchInput.setAttribute('aria-activedescendant', `search-history-option-${searchState.historyHighlightedIndex}`);
    } else {
      searchInput.removeAttribute('aria-activedescendant');
    }
  }
}

function openSearchHistoryDropdown() {
  if (searchState.historySubmitLockUntil > Date.now()) {
    return;
  }
  if (searchState.historyBlurTimer) {
    window.clearTimeout(searchState.historyBlurTimer);
    searchState.historyBlurTimer = 0;
  }
  searchState.historyHighlightedIndex = -1;
  setSearchHistoryDropdown(true);
  renderSearchHistoryDropdown();
}

function closeSearchHistoryDropdown() {
  if (searchState.historyBlurTimer) {
    window.clearTimeout(searchState.historyBlurTimer);
  }
  searchState.historyBlurTimer = window.setTimeout(() => {
    searchState.historyHighlightedIndex = -1;
    setSearchHistoryDropdown(false);
  }, 120);
}

function getFocusableElements(container) {
  if (!(container instanceof HTMLElement)) {
    return [];
  }
  return Array.from(container.querySelectorAll(
    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
  )).filter((element) =>
    element instanceof HTMLElement
    && !element.hasAttribute('disabled')
    && element.getAttribute('aria-hidden') !== 'true'
    && !element.classList.contains('hidden')
  );
}

function trapDrawerFocus(event, drawerEl) {
  if (event.key !== 'Tab' || !(drawerEl instanceof HTMLElement)) {
    return;
  }

  const focusableElements = getFocusableElements(drawerEl);
  if (!focusableElements.length) {
    event.preventDefault();
    drawerEl.focus();
    return;
  }

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];
  const activeElement = document.activeElement;
  const isInsideDrawer = activeElement instanceof HTMLElement && drawerEl.contains(activeElement);

  if (event.shiftKey) {
    if (activeElement === firstElement || activeElement === drawerEl || !isInsideDrawer) {
      event.preventDefault();
      lastElement.focus();
    }
    return;
  }

  if (activeElement === lastElement || !isInsideDrawer) {
    event.preventDefault();
    firstElement.focus();
  }
}

function moveHomeLibraryTabFocus(buttons, currentButton, direction) {
  const tabButtons = Array.from(buttons).filter((button) => button instanceof HTMLButtonElement);
  if (!tabButtons.length || !(currentButton instanceof HTMLButtonElement)) {
    return;
  }
  const currentIndex = tabButtons.indexOf(currentButton);
  const nextIndex = direction === 'next'
    ? (currentIndex + 1 + tabButtons.length) % tabButtons.length
    : (currentIndex - 1 + tabButtons.length) % tabButtons.length;
  tabButtons[nextIndex].focus();
  const nextTab = tabButtons[nextIndex].dataset.libraryTab || tabButtons[nextIndex].dataset.libraryDrawerTab || 'history';
  setHomeLibraryTab(nextTab);
}

function setHomeLibraryTab(tab) {
  const nextTab = tab === 'favorites' ? 'favorites' : 'history';
  searchState.libraryTab = nextTab;
  const buttons = homeLibraryToggleEl?.querySelectorAll('[data-library-tab]') || [];
  buttons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.libraryTab === nextTab);
    button.setAttribute('aria-selected', button.dataset.libraryTab === nextTab ? 'true' : 'false');
    button.tabIndex = button.dataset.libraryTab === nextTab ? 0 : -1;
  });

  const drawerButtons = homeLibraryDrawerToggleEl?.querySelectorAll('[data-library-drawer-tab]') || [];
  drawerButtons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.libraryDrawerTab === nextTab);
    button.setAttribute('aria-selected', button.dataset.libraryDrawerTab === nextTab ? 'true' : 'false');
    button.tabIndex = button.dataset.libraryDrawerTab === nextTab ? 0 : -1;
  });

  homeLibraryPanelEls.forEach((panel) => {
    if (!(panel instanceof HTMLElement)) {
      return;
    }
    const panelTab = panel.dataset.libraryPanel === 'favorites' ? 'favorites' : 'history';
    const hidden = panelTab !== nextTab;
    panel.classList.toggle('hidden', hidden);
    panel.hidden = hidden;
  });

  homeLibraryDrawerPanelEls.forEach((panel) => {
    if (!(panel instanceof HTMLElement)) {
      return;
    }
    const panelTab = panel.dataset.libraryDrawerPanel === 'favorites' ? 'favorites' : 'history';
    const hidden = panelTab !== nextTab;
    panel.classList.toggle('hidden', hidden);
    panel.hidden = hidden;
  });

  homeLibraryStatusEls.forEach((element) => {
    if (!(element instanceof HTMLElement)) {
      return;
    }
    element.textContent = nextTab === 'favorites' ? '收藏面板' : '历史面板';
  });
  renderHomeLibraryCollections();
}

function toggleHomeLibraryDrawer(open) {
  if (!(homeLibraryDrawerOverlayEl instanceof HTMLElement) || !(homeLibraryDrawerEl instanceof HTMLElement)) {
    return;
  }
  homeLibraryDrawerOverlayEl.classList.toggle('hidden', !open);
  homeLibraryDrawerOverlayEl.setAttribute('aria-hidden', open ? 'false' : 'true');
  openHomeLibraryDrawerButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.setAttribute('aria-expanded', open ? 'true' : 'false');
    }
  });
  document.body.classList.toggle('drawer-open', open);

  if (open) {
    searchState.drawerLastFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (searchState.drawerTrapHandler) {
      homeLibraryDrawerEl.removeEventListener('keydown', searchState.drawerTrapHandler);
    }
    searchState.drawerTrapHandler = (event) => trapDrawerFocus(event, homeLibraryDrawerEl);
    homeLibraryDrawerEl.addEventListener('keydown', searchState.drawerTrapHandler);
    window.requestAnimationFrame(() => {
      const focusableElements = getFocusableElements(homeLibraryDrawerEl);
      const target = focusableElements[0] || homeLibraryDrawerEl;
      target.focus();
    });
    return;
  }

  if (searchState.drawerTrapHandler) {
    homeLibraryDrawerEl.removeEventListener('keydown', searchState.drawerTrapHandler);
    searchState.drawerTrapHandler = null;
  }
  const lastFocused = searchState.drawerLastFocused;
  searchState.drawerLastFocused = null;
  if (lastFocused instanceof HTMLElement) {
    window.requestAnimationFrame(() => {
      lastFocused.focus();
    });
  }
}

function normalizeTitle(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/[\s\-_:：·•,，.。!！?？"'`~《》“”‘’()\[\]【】]/g, '')
    .trim();
}

function getLatencyValue(video) {
  const liveLatency = searchState.liveLatencies[String(video?.source || '')];
  if (typeof liveLatency === 'number') {
    return liveLatency;
  }
  return typeof video?.latency === 'number' ? video.latency : 99999;
}

function isRealtimeLatencyEnabled() {
  return initialSettings.realtimeLatency === true;
}

function stopLatencyPolling() {
  if (searchState.latencyTimer) {
    window.clearInterval(searchState.latencyTimer);
    searchState.latencyTimer = 0;
  }
  searchState.latencyRequestId += 1;
}

async function pingSourceLatency(baseUrl) {
  const response = await fetch('/api/ping', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url: baseUrl }),
  });

  if (!response.ok) {
    throw new Error(`测速失败: ${response.status}`);
  }

  const data = await response.json().catch(() => ({}));
  return typeof data.latency === 'number' ? data.latency : null;
}

async function refreshRealtimeLatencies() {
  if (!isRealtimeLatencyEnabled() || !searchState.rawResults.length) {
    return;
  }

  const requestId = ++searchState.latencyRequestId;
  const sourceEntries = Array.from(new Set(
    searchState.rawResults
      .map((video) => String(video?.source || '').trim())
      .filter(Boolean)
  ))
    .map((sourceId) => [sourceId, searchState.sourceBaseUrls[sourceId]])
    .filter(([, baseUrl]) => typeof baseUrl === 'string' && baseUrl.trim());

  if (!sourceEntries.length) {
    return;
  }

  const results = await Promise.all(
    sourceEntries.map(async ([sourceId, baseUrl]) => {
      try {
        const latency = await pingSourceLatency(baseUrl);
        return { sourceId, latency };
      } catch (_) {
        return { sourceId, latency: null };
      }
    })
  );

  if (requestId !== searchState.latencyRequestId) {
    return;
  }

  let changed = false;
  results.forEach(({ sourceId, latency }) => {
    if (typeof latency !== 'number') {
      return;
    }
    if (searchState.liveLatencies[sourceId] !== latency) {
      searchState.liveLatencies[sourceId] = latency;
      changed = true;
    }
  });

  if (changed) {
    applySearchPresentation();
  }
}

function ensureLatencyPolling() {
  if (!isRealtimeLatencyEnabled() || !searchState.rawResults.length) {
    stopLatencyPolling();
    return;
  }

  if (!searchState.latencyTimer) {
    void refreshRealtimeLatencies();
    searchState.latencyTimer = window.setInterval(() => {
      void refreshRealtimeLatencies();
    }, SEARCH_LATENCY_INTERVAL_MS);
  }
}

function sortVideos(videos, sortBy) {
  const sorted = [...videos];

  switch (sortBy) {
    case 'relevance':
      return sorted.sort((a, b) => Number(b?.relevanceScore || 0) - Number(a?.relevanceScore || 0));
    case 'latency-asc':
      return sorted.sort((a, b) => getLatencyValue(a) - getLatencyValue(b));
    case 'date-desc':
      return sorted.sort((a, b) => Number.parseInt(String(b?.vod_year || '0'), 10) - Number.parseInt(String(a?.vod_year || '0'), 10));
    case 'date-asc':
      return sorted.sort((a, b) => Number.parseInt(String(a?.vod_year || '0'), 10) - Number.parseInt(String(b?.vod_year || '0'), 10));
    case 'rating-desc':
      return sorted.sort((a, b) => Number(b?.vod_score || 0) - Number(a?.vod_score || 0));
    case 'name-asc':
      return sorted.sort((a, b) => String(a?.vod_name || '').localeCompare(String(b?.vod_name || ''), 'zh-CN'));
    case 'name-desc':
      return sorted.sort((a, b) => String(b?.vod_name || '').localeCompare(String(a?.vod_name || ''), 'zh-CN'));
    case 'default':
    default:
      return sorted.sort((a, b) => {
        const scoreGap = Number(b?.relevanceScore || 0) - Number(a?.relevanceScore || 0);
        if (scoreGap !== 0) {
          return scoreGap;
        }
        return getLatencyValue(a) - getLatencyValue(b);
      });
  }
}

function buildSearchSourceBadges(videos) {
  const sourceMap = new Map();
  videos.forEach((video) => {
    const sourceId = String(video?.source || '').trim();
    if (!sourceId) {
      return;
    }
    const existing = sourceMap.get(sourceId) || {
      id: sourceId,
      name: String(video?.sourceDisplayName || video?.sourceName || sourceId).trim() || sourceId,
      count: 0,
    };
    existing.count += 1;
    sourceMap.set(sourceId, existing);
  });
  return Array.from(sourceMap.values()).sort((left, right) => {
    if (right.count !== left.count) {
      return right.count - left.count;
    }
    return String(left.name || '').localeCompare(String(right.name || ''), 'zh-CN');
  });
}

function buildSearchTypeBadges(videos) {
  const typeMap = new Map();
  videos.forEach((video) => {
    const typeName = String(video?.type_name || '').trim();
    if (!typeName) {
      return;
    }
    typeMap.set(typeName, (typeMap.get(typeName) || 0) + 1);
  });
  return Array.from(typeMap.entries())
    .map(([type, count]) => ({ type, count }))
    .sort((left, right) => {
      if (right.count !== left.count) {
        return right.count - left.count;
      }
      return String(left.type || '').localeCompare(String(right.type || ''), 'zh-CN');
    });
}

function pruneSearchFilters(availableSources, availableTypes) {
  const availableSourceIds = new Set(availableSources.map((source) => source.id));
  searchState.selectedSources = new Set(
    Array.from(searchState.selectedSources).filter((sourceId) => availableSourceIds.has(sourceId))
  );

  const availableTypeNames = new Set(availableTypes.map((badge) => badge.type));
  searchState.selectedTypes = new Set(
    Array.from(searchState.selectedTypes).filter((type) => availableTypeNames.has(type))
  );
}

function filterSearchResults(videos) {
  return videos.filter((video) => {
    const sourceMatched = searchState.selectedSources.size === 0
      || (video?.source && searchState.selectedSources.has(video.source));
    const typeMatched = searchState.selectedTypes.size === 0
      || (video?.type_name && searchState.selectedTypes.has(String(video.type_name).trim()));
    return sourceMatched && typeMatched;
  });
}

function renderSearchFilters(videos) {
  const sourceBadges = buildSearchSourceBadges(videos);
  const typeBadges = buildSearchTypeBadges(videos);
  pruneSearchFilters(sourceBadges, typeBadges);

  const hasFilterData = sourceBadges.length > 0 || typeBadges.length > 0;
  searchFilterToolbarEl?.classList.toggle('hidden', !hasFilterData);

  const sourceFilterCount = searchState.selectedSources.size;
  const typeFilterCount = searchState.selectedTypes.size;
  const filterSummary = [];
  if (sourceFilterCount > 0) {
    filterSummary.push(`来源 ${sourceFilterCount}`);
  }
  if (typeFilterCount > 0) {
    filterSummary.push(`类型 ${typeFilterCount}`);
  }
  if (searchFilterSummaryEl instanceof HTMLElement) {
    searchFilterSummaryEl.textContent = filterSummary.length ? `已筛选：${filterSummary.join('，')}` : '未启用筛选';
  }
  if (clearSearchFiltersButton instanceof HTMLButtonElement) {
    clearSearchFiltersButton.disabled = sourceFilterCount === 0 && typeFilterCount === 0;
  }

  if (searchSourceBadgesEl instanceof HTMLElement) {
    if (!sourceBadges.length) {
      searchSourceBadgesEl.innerHTML = '<span class="chip">当前结果没有来源标签</span>';
    } else {
      searchSourceBadgesEl.innerHTML = sourceBadges.map((source) => `
        <button
          class="button button-small ${searchState.selectedSources.has(source.id) ? 'primary' : ''}"
          type="button"
          data-search-source-badge="${escapeHtml(source.id)}"
          aria-pressed="${searchState.selectedSources.has(source.id) ? 'true' : 'false'}"
        >
          ${escapeHtml(source.name)} (${source.count})
        </button>
      `).join('');
    }
  }

  if (searchTypeBadgesEl instanceof HTMLElement) {
    if (!typeBadges.length) {
      searchTypeBadgesEl.innerHTML = '<span class="chip">当前结果没有类型标签</span>';
    } else {
      searchTypeBadgesEl.innerHTML = typeBadges.map((badge) => `
        <button
          class="button button-small ${searchState.selectedTypes.has(badge.type) ? 'primary' : ''}"
          type="button"
          data-search-type-badge="${escapeHtml(badge.type)}"
          aria-pressed="${searchState.selectedTypes.has(badge.type) ? 'true' : 'false'}"
        >
          ${escapeHtml(badge.type)} (${badge.count})
        </button>
      `).join('');
    }
  }
}

function groupVideos(videos) {
  const groups = new Map();

  videos.forEach((video) => {
    const key = normalizeTitle(video?.vod_name || '');
    if (!key) {
      return;
    }
    if (!groups.has(key)) {
      groups.set(key, []);
    }
    groups.get(key).push(video);
  });

  return Array.from(groups.values()).map((items) => {
    const sortedItems = [...items].sort((a, b) => getLatencyValue(a) - getLatencyValue(b));
    return {
      representative: sortedItems[0],
      videos: sortedItems,
      name: sortedItems[0]?.vod_name || '未知标题',
    };
  });
}

function groupedDetailUrl(group) {
  const representative = group?.representative || {};
  const params = new URLSearchParams({
    id: String(representative.vod_id ?? ''),
    source: String(representative.source ?? ''),
    title: String(representative.vod_name ?? '未知视频'),
  });
  const groupedSources = Array.isArray(group?.videos) ? group.videos.map((video) => ({
    id: video?.vod_id ?? '',
    source: video?.source ?? '',
    sourceName: video?.sourceDisplayName || video?.sourceName || video?.source || '',
    latency: typeof getLatencyValue(video) === 'number' ? getLatencyValue(video) : undefined,
    pic: video?.vod_pic || undefined,
  })) : [];
  if (groupedSources.length > 1) {
    params.set('groupedSources', JSON.stringify(groupedSources));
  }
  return `/player?${params.toString()}`;
}

function renderDisplayToggle() {
  const buttons = displayToggleEl?.querySelectorAll('[data-display-mode]') || [];
  buttons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.displayMode === searchState.displayMode);
  });
}

function applySearchPresentation() {
  const sortedResults = sortVideos(searchState.rawResults, searchState.sortBy);
  renderSearchFilters(sortedResults);
  const filteredResults = filterSearchResults(sortedResults);

  if (!sortedResults.length) {
    resultsEl.className = 'results-grid empty-state';
    resultsEl.textContent = '暂无结果';
    totalEl.textContent = '结果 0';
    renderDisplayToggle();
    if (sortSelectEl instanceof HTMLSelectElement) {
      sortSelectEl.value = searchState.sortBy;
    }
    return;
  }

  renderDisplayToggle();
  if (sortSelectEl instanceof HTMLSelectElement) {
    sortSelectEl.value = searchState.sortBy;
  }

  if (!filteredResults.length) {
    resultsEl.className = 'results-grid empty-state';
    resultsEl.textContent = '当前筛选条件下暂无结果';
    totalEl.textContent = `结果 ${sortedResults.length} / 筛选 0`;
    return;
  }

  resultsEl.className = 'results-grid';
  if (searchState.displayMode === 'grouped') {
    const grouped = groupVideos(filteredResults)
      .sort((left, right) => sortVideos([left.representative, right.representative], searchState.sortBy)[0] === left.representative ? -1 : 1);

    resultsEl.innerHTML = grouped.map((group) => {
      const representative = group.representative || {};
      const poster = representative.vod_pic
        ? `<img class="result-poster" src="${escapeHtml(representative.vod_pic)}" alt="${escapeHtml(group.name || '未知视频')}" referrerpolicy="no-referrer" />`
        : '<div class="result-poster placeholder">🎬</div>';
      const bestLatency = typeof representative.latency === 'number' ? `<span class="chip">${representative.latency} ms</span>` : '';
      const sourceNames = group.videos
        .slice(0, 3)
        .map((video) => escapeHtml(video.sourceDisplayName || video.sourceName || video.source || '未知来源'))
        .join(' / ');
      return `
        <a class="result-card" href="${groupedDetailUrl(group)}">
          ${poster}
          <div class="stack compact-card-body">
            <strong>${escapeHtml(group.name || '未知标题')}</strong>
            <div class="row wrap gap-sm">
              <span class="chip">${group.videos.length} 个源</span>
              ${bestLatency}
              ${representative.vod_remarks ? `<span class="chip">${escapeHtml(representative.vod_remarks)}</span>` : ''}
            </div>
            <p class="muted">${escapeHtml(sourceNames || '暂无来源信息')}</p>
          </div>
        </a>`;
    }).join('');
    totalEl.textContent = `分组 ${grouped.length} / 条目 ${filteredResults.length} / 总计 ${sortedResults.length}`;
    return;
  }

  resultsEl.innerHTML = filteredResults.map((video) => {
    const poster = video.vod_pic
      ? `<img class="result-poster" src="${escapeHtml(video.vod_pic)}" alt="${escapeHtml(video.vod_name || '未知视频')}" referrerpolicy="no-referrer" />`
      : '<div class="result-poster placeholder">🎬</div>';
    const latency = typeof video.latency === 'number' ? `<span class="chip">${video.latency} ms</span>` : '';
    return `
      <a class="result-card" href="${resultPlayerUrl(video)}">
        ${poster}
        <div class="stack compact-card-body">
          <strong>${escapeHtml(video.vod_name || '未知标题')}</strong>
          <div class="row wrap gap-sm">
            <span class="chip">${escapeHtml(video.sourceDisplayName || video.sourceName || video.source || '未知来源')}</span>
            ${latency}
            ${video.vod_remarks ? `<span class="chip">${escapeHtml(video.vod_remarks)}</span>` : ''}
          </div>
          <p class="muted">${escapeHtml(video.type_name || video.vod_year || '暂无分类信息')}</p>
        </div>
      </a>`;
  }).join('');
  totalEl.textContent = filteredResults.length === sortedResults.length
    ? `结果 ${filteredResults.length}`
    : `结果 ${filteredResults.length} / 总计 ${sortedResults.length}`;
}

async function runSearch(query) {
  const normalizedQuery = String(query || '').trim();
  if (!normalizedQuery) {
    setSearchStatus('error', '请输入搜索关键词');
    return;
  }

  const sources = getEnabledSources(initialSettings);
  if (!sources.length) {
    setSearchStatus('error', '当前没有可用线路，请先在设置中启用至少一个源');
    return;
  }

  stopLatencyPolling();
  searchState.liveLatencies = {};
  searchState.selectedSources = new Set();
  searchState.selectedTypes = new Set();
  searchState.sourceBaseUrls = Object.fromEntries(
    sources
      .filter((source) => typeof source?.id === 'string' && typeof source?.baseUrl === 'string')
      .map((source) => [source.id, source.baseUrl])
  );

  progressEl.textContent = '搜索中...';
  totalEl.textContent = '结果 0';
  setSearchStatus('muted', `正在搜索「${normalizedQuery}」...`);
  resultsEl.className = 'results-grid empty-state';
  resultsEl.textContent = '正在等待流式结果...';

  const response = await fetch('/api/search-parallel', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query: normalizedQuery, sources, page: 1 }),
  });

  if (!response.ok || !response.body) {
    const data = await response.json().catch(() => ({}));
    setSearchStatus('error', data.error || '搜索失败');
    progressEl.textContent = '搜索失败';
    return;
  }

  searchState.rawResults = [];
  const decoder = new TextDecoder();
  const reader = response.body.getReader();
  let buffer = '';
  let totalSources = 0;

  while (true) {
    const { value, done } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (!line.startsWith('data: ')) continue;

      try {
        const payload = JSON.parse(line.slice(6));
        if (payload.type === 'start') {
          totalSources = payload.totalSources || 0;
          progressEl.textContent = `已启动 ${totalSources} 条线路`;
        } else if (payload.type === 'videos') {
          searchState.rawResults.push(...(payload.videos || []));
          applySearchPresentation();
          ensureLatencyPolling();
        } else if (payload.type === 'progress') {
          progressEl.textContent = `进度 ${payload.completedSources || 0}/${totalSources || payload.totalSources || 0}`;
          setSearchStatus('muted', `已收到 ${searchState.rawResults.length} 条结果`);
        } else if (payload.type === 'complete') {
          progressEl.textContent = '搜索完成';
          setSearchStatus('success', `搜索完成，共 ${searchState.rawResults.length} 条结果`);
          ensureLatencyPolling();
        } else if (payload.type === 'error') {
          progressEl.textContent = '搜索失败';
          setSearchStatus('error', payload.message || '搜索失败');
          stopLatencyPolling();
        }
      } catch (error) {
        setSearchStatus('error', error instanceof Error ? error.message : '解析搜索结果失败');
      }
    }
  }

  if (!searchState.rawResults.length) {
    applySearchPresentation();
    setSearchStatus('muted', `搜索完成，没有找到「${normalizedQuery}」相关结果`);
    stopLatencyPolling();
  }

  updateSearchHistory(normalizedQuery, searchState.rawResults.length);
}

window.kvideoRustRunSearch = runSearch;

displayToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-display-mode]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const nextMode = button.dataset.displayMode;
  if (nextMode !== 'normal' && nextMode !== 'grouped') {
    return;
  }
  searchState.displayMode = nextMode;
  applySearchPresentation();
});

sortSelectEl?.addEventListener('change', () => {
  if (!(sortSelectEl instanceof HTMLSelectElement)) {
    return;
  }
  searchState.sortBy = sortSelectEl.value || 'default';
  applySearchPresentation();
});

searchForm?.addEventListener('submit', async (event) => {
  event.preventDefault();
  const query = searchInput.value.trim();
  searchState.historySubmitLockUntil = Date.now() + 800;
  setSearchHistoryDropdown(false);
  searchInput.blur();
  const params = new URLSearchParams(window.location.search);
  if (query) {
    params.set('q', query);
  } else {
    params.delete('q');
  }
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
  await runSearch(query);
});

clearButton?.addEventListener('click', () => {
  stopLatencyPolling();
  searchState.liveLatencies = {};
  searchInput.value = '';
  searchState.rawResults = [];
  searchState.selectedSources = new Set();
  searchState.selectedTypes = new Set();
  renderSearchFilters([]);
  resultsEl.className = 'results-grid empty-state';
  resultsEl.textContent = '请输入关键词开始搜索。';
  progressEl.textContent = '等待搜索';
  totalEl.textContent = '结果 0';
  setSearchStatus('muted', '输入关键词后会直接在下方显示结果。');
  window.history.replaceState({}, '', window.location.pathname);
});

clearSearchFiltersButton?.addEventListener('click', () => {
  searchState.selectedSources = new Set();
  searchState.selectedTypes = new Set();
  applySearchPresentation();
});

searchSourceBadgesEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-search-source-badge]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const sourceId = String(button.dataset.searchSourceBadge || '').trim();
  if (!sourceId) {
    return;
  }
  if (searchState.selectedSources.has(sourceId)) {
    searchState.selectedSources.delete(sourceId);
  } else {
    searchState.selectedSources.add(sourceId);
  }
  applySearchPresentation();
});

searchTypeBadgesEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-search-type-badge]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const type = String(button.dataset.searchTypeBadge || '').trim();
  if (!type) {
    return;
  }
  if (searchState.selectedTypes.has(type)) {
    searchState.selectedTypes.delete(type);
  } else {
    searchState.selectedTypes.add(type);
  }
  applySearchPresentation();
});

clearHistoryButton?.addEventListener('click', () => {
  clearSearchHistory();
  setSearchStatus('success', '已清空搜索历史');
});

document.querySelectorAll('[data-quick-query]').forEach((button) => {
  button.addEventListener('click', () => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    const query = String(button.dataset.quickQuery || '').trim();
    if (!query) {
      return;
    }
    searchInput.value = query;
    const params = new URLSearchParams(window.location.search);
    params.set('q', query);
    window.history.replaceState({}, '', `${window.location.pathname}?${params.toString()}`);
    runSearch(query).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
      progressEl.textContent = '搜索失败';
    });
  });
});

homeLibraryToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest('[data-library-tab]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  setHomeLibraryTab(button.dataset.libraryTab || 'history');
});

homeLibraryToggleEl?.addEventListener('keydown', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLButtonElement) || target.dataset.libraryTab == null) {
    return;
  }
  const buttons = homeLibraryToggleEl.querySelectorAll('[data-library-tab]');
  if (event.key === 'ArrowRight') {
    event.preventDefault();
    moveHomeLibraryTabFocus(buttons, target, 'next');
    return;
  }
  if (event.key === 'ArrowLeft') {
    event.preventDefault();
    moveHomeLibraryTabFocus(buttons, target, 'prev');
    return;
  }
  if (event.key === 'Home') {
    event.preventDefault();
    const firstButton = buttons[0];
    if (firstButton instanceof HTMLButtonElement) {
      firstButton.focus();
      setHomeLibraryTab(firstButton.dataset.libraryTab || 'history');
    }
    return;
  }
  if (event.key === 'End') {
    event.preventDefault();
    const lastButton = buttons[buttons.length - 1];
    if (lastButton instanceof HTMLButtonElement) {
      lastButton.focus();
      setHomeLibraryTab(lastButton.dataset.libraryTab || 'favorites');
    }
  }
});

homeLibraryDrawerToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest('[data-library-drawer-tab]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  setHomeLibraryTab(button.dataset.libraryDrawerTab || 'history');
});

homeLibraryDrawerToggleEl?.addEventListener('keydown', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLButtonElement) || target.dataset.libraryDrawerTab == null) {
    return;
  }
  const buttons = homeLibraryDrawerToggleEl.querySelectorAll('[data-library-drawer-tab]');
  if (event.key === 'ArrowRight') {
    event.preventDefault();
    moveHomeLibraryTabFocus(buttons, target, 'next');
    return;
  }
  if (event.key === 'ArrowLeft') {
    event.preventDefault();
    moveHomeLibraryTabFocus(buttons, target, 'prev');
    return;
  }
  if (event.key === 'Home') {
    event.preventDefault();
    const firstButton = buttons[0];
    if (firstButton instanceof HTMLButtonElement) {
      firstButton.focus();
      setHomeLibraryTab(firstButton.dataset.libraryDrawerTab || 'history');
    }
    return;
  }
  if (event.key === 'End') {
    event.preventDefault();
    const lastButton = buttons[buttons.length - 1];
    if (lastButton instanceof HTMLButtonElement) {
      lastButton.focus();
      setHomeLibraryTab(lastButton.dataset.libraryDrawerTab || 'favorites');
    }
  }
});

openHomeLibraryDrawerButtons.forEach((button) => {
  button.addEventListener('click', () => {
    toggleHomeLibraryDrawer(true);
  });
});

closeHomeLibraryDrawerButton?.addEventListener('click', () => {
  toggleHomeLibraryDrawer(false);
});

homeLibraryDrawerOverlayEl?.addEventListener('click', (event) => {
  if (event.target === homeLibraryDrawerOverlayEl) {
    toggleHomeLibraryDrawer(false);
  }
});

homeLibraryFilterInputs.forEach((input) => {
  input.addEventListener('input', () => {
    if (input instanceof HTMLInputElement) {
      updateHomeLibraryFilterValue(input.value);
    }
  });
});

homeLibrarySortInputs.forEach((input) => {
  input.addEventListener('change', () => {
    if (input instanceof HTMLSelectElement) {
      updateHomeLibrarySortValue(input.value);
    }
  });
});

clearHomeLibraryFilterButtons.forEach((button) => {
  button.addEventListener('click', () => {
    updateHomeLibraryFilterValue('');
  });
});

function triggerHomeLibrarySearch(query) {
  const normalized = String(query || '').trim();
  if (!normalized) {
    return;
  }
  searchInput.value = normalized;
  const params = new URLSearchParams(window.location.search);
  params.set('q', normalized);
  window.history.replaceState({}, '', `${window.location.pathname}?${params.toString()}`);
  toggleHomeLibraryDrawer(false);
  runSearch(normalized).catch((error) => {
    setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
    progressEl.textContent = '搜索失败';
  });
}

document.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const clearButton = target.closest('[data-home-library-clear]');
  if (clearButton instanceof HTMLButtonElement) {
    clearHomeLibraryItems(searchState.libraryTab).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '清空列表失败');
    });
    return;
  }

  const toggleSelectionButton = target.closest('[data-home-library-selection-toggle]');
  if (toggleSelectionButton instanceof HTMLButtonElement) {
    setHomeLibrarySelectionMode(!searchState.librarySelectionMode);
    return;
  }

  const selectAllButton = target.closest('[data-home-library-select-all]');
  if (selectAllButton instanceof HTMLButtonElement) {
    selectAllHomeLibraryItems(searchState.libraryTab);
    return;
  }

  const removeSelectedButton = target.closest('[data-home-library-remove-selected]');
  if (removeSelectedButton instanceof HTMLButtonElement) {
    removeSelectedHomeLibraryItems(searchState.libraryTab).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '批量移除失败');
    });
    return;
  }

  const undoButton = target.closest('[data-home-library-undo]');
  if (undoButton instanceof HTMLButtonElement) {
    undoHomeLibraryRemoval().catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '撤销移除失败');
    });
    return;
  }

  const copyButton = target.closest('[data-home-library-copy]');
  if (copyButton instanceof HTMLButtonElement) {
    copyHomeLibraryItems(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已复制当前${getHomeLibraryLabel(searchState.libraryTab)}列表`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制列表失败');
    });
    return;
  }

  const shareButton = target.closest('[data-home-library-share]');
  if (shareButton instanceof HTMLButtonElement) {
    copyHomeLibrarySharePackage(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已复制当前${getHomeLibraryLabel(searchState.libraryTab)}分享包`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制分享包失败');
    });
    return;
  }

  const shareLinkButton = target.closest('[data-home-library-share-link]');
  if (shareLinkButton instanceof HTMLButtonElement) {
    copyHomeLibraryShareLink(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已复制当前${getHomeLibraryLabel(searchState.libraryTab)}分享链接`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制分享链接失败');
    });
    return;
  }

  const shareLinkMergeButton = target.closest('[data-home-library-share-link-merge]');
  if (shareLinkMergeButton instanceof HTMLButtonElement) {
    copyHomeLibraryShareLink(searchState.libraryTab, true).then(() => {
      setSearchStatus('success', `已复制当前${getHomeLibraryLabel(searchState.libraryTab)}合并分享链接`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制合并分享链接失败');
    });
    return;
  }

  const nativeShareButton = target.closest('[data-home-library-share-native]');
  if (nativeShareButton instanceof HTMLButtonElement) {
    shareHomeLibraryLink(searchState.libraryTab).then(() => {
      setSearchStatus('success', navigator.share
        ? `已发起当前${getHomeLibraryLabel(searchState.libraryTab)}系统分享`
        : `已复制当前${getHomeLibraryLabel(searchState.libraryTab)}分享链接`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '分享当前列表失败');
    });
    return;
  }

  const saveSnapshotButton = target.closest('[data-home-library-snapshot-save]');
  if (saveSnapshotButton instanceof HTMLButtonElement) {
    saveHomeLibrarySnapshot(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已保存${getHomeLibraryLabel(searchState.libraryTab)}快照`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '保存快照失败');
    });
    return;
  }

  const renameSnapshotButton = target.closest('[data-home-library-snapshot-rename]');
  if (renameSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = renameHomeLibrarySnapshot(searchState.libraryTab);
      setSearchStatus('success', `已将快照「${snapshot.previousName}」重命名为「${snapshot.name}」`);
    } catch (error) {
      setSearchStatus('error', error instanceof Error ? error.message : '重命名快照失败');
    }
    return;
  }

  const duplicateSnapshotButton = target.closest('[data-home-library-snapshot-duplicate]');
  if (duplicateSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = duplicateHomeLibrarySnapshot(searchState.libraryTab);
      setSearchStatus('success', `已克隆快照「${snapshot.name}」`);
    } catch (error) {
      setSearchStatus('error', error instanceof Error ? error.message : '克隆快照失败');
    }
    return;
  }

  const restoreSnapshotButton = target.closest('[data-home-library-snapshot-restore]');
  if (restoreSnapshotButton instanceof HTMLButtonElement) {
    restoreHomeLibrarySnapshot(searchState.libraryTab).then((snapshot) => {
      setSearchStatus('success', `已恢复快照「${snapshot.name}」，可撤销`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '恢复快照失败');
    });
    return;
  }

  const mergeSnapshotButton = target.closest('[data-home-library-snapshot-merge]');
  if (mergeSnapshotButton instanceof HTMLButtonElement) {
    restoreHomeLibrarySnapshot(searchState.libraryTab, { merge: true }).then((snapshot) => {
      setSearchStatus('success', `已合并快照「${snapshot.name}」，可撤销`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '合并快照失败');
    });
    return;
  }

  const deleteSnapshotButton = target.closest('[data-home-library-snapshot-delete]');
  if (deleteSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = deleteHomeLibrarySnapshot(searchState.libraryTab);
      if (snapshot) {
        setSearchStatus('success', `已删除快照「${snapshot.name}」`);
      }
    } catch (error) {
      setSearchStatus('error', error instanceof Error ? error.message : '删除快照失败');
    }
    return;
  }

  const exportSnapshotButton = target.closest('[data-home-library-snapshot-export]');
  if (exportSnapshotButton instanceof HTMLButtonElement) {
    try {
      exportHomeLibrarySnapshots(searchState.libraryTab);
      setSearchStatus('success', `已导出${getHomeLibraryLabel(searchState.libraryTab)}快照`);
    } catch (error) {
      setSearchStatus('error', error instanceof Error ? error.message : '导出快照失败');
    }
    return;
  }

  const shareSnapshotButton = target.closest('[data-home-library-snapshot-share]');
  if (shareSnapshotButton instanceof HTMLButtonElement) {
    copyHomeLibrarySnapshotSharePackage(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已复制${getHomeLibraryLabel(searchState.libraryTab)}快照分享包`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制快照分享包失败');
    });
    return;
  }

  const shareSnapshotLinkButton = target.closest('[data-home-library-snapshot-share-link]');
  if (shareSnapshotLinkButton instanceof HTMLButtonElement) {
    copyHomeLibrarySnapshotShareLink(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已复制${getHomeLibraryLabel(searchState.libraryTab)}快照分享链接`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制快照分享链接失败');
    });
    return;
  }

  const shareSnapshotMergeLinkButton = target.closest('[data-home-library-snapshot-share-link-merge]');
  if (shareSnapshotMergeLinkButton instanceof HTMLButtonElement) {
    copyHomeLibrarySnapshotShareLink(searchState.libraryTab, true).then(() => {
      setSearchStatus('success', `已复制${getHomeLibraryLabel(searchState.libraryTab)}快照合并链接`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '复制快照合并链接失败');
    });
    return;
  }

  const importSnapshotButton = target.closest('[data-home-library-snapshot-import]');
  if (importSnapshotButton instanceof HTMLButtonElement) {
    if (homeLibrarySnapshotImportFileInput instanceof HTMLInputElement) {
      homeLibrarySnapshotImportFileInput.dataset.importMode = 'replace';
      homeLibrarySnapshotImportFileInput.value = '';
      homeLibrarySnapshotImportFileInput.click();
    } else {
      setSearchStatus('error', '当前页面不可用快照导入功能');
    }
    return;
  }

  const mergeImportSnapshotButton = target.closest('[data-home-library-snapshot-import-merge]');
  if (mergeImportSnapshotButton instanceof HTMLButtonElement) {
    if (homeLibrarySnapshotImportFileInput instanceof HTMLInputElement) {
      homeLibrarySnapshotImportFileInput.dataset.importMode = 'merge';
      homeLibrarySnapshotImportFileInput.value = '';
      homeLibrarySnapshotImportFileInput.click();
    } else {
      setSearchStatus('error', '当前页面不可用快照合并导入功能');
    }
    return;
  }

  const exportButton = target.closest('[data-home-library-export]');
  if (exportButton instanceof HTMLButtonElement) {
    try {
      exportHomeLibraryItems(searchState.libraryTab);
      setSearchStatus('success', `已导出当前${getHomeLibraryLabel(searchState.libraryTab)}列表`);
    } catch (error) {
      setSearchStatus('error', error instanceof Error ? error.message : '导出列表失败');
    }
    return;
  }

  const importButton = target.closest('[data-home-library-import]');
  if (importButton instanceof HTMLButtonElement) {
    if (homeLibraryImportFileInput instanceof HTMLInputElement) {
      homeLibraryImportFileInput.dataset.importMode = 'replace';
      homeLibraryImportFileInput.value = '';
      homeLibraryImportFileInput.click();
    } else {
      setSearchStatus('error', '当前页面不可用导入功能');
    }
    return;
  }

  const mergeImportButton = target.closest('[data-home-library-import-merge]');
  if (mergeImportButton instanceof HTMLButtonElement) {
    if (homeLibraryImportFileInput instanceof HTMLInputElement) {
      homeLibraryImportFileInput.dataset.importMode = 'merge';
      homeLibraryImportFileInput.value = '';
      homeLibraryImportFileInput.click();
    } else {
      setSearchStatus('error', '当前页面不可用合并导入功能');
    }
    return;
  }

  const clipboardImportButton = target.closest('[data-home-library-import-clipboard]');
  if (clipboardImportButton instanceof HTMLButtonElement) {
    readClipboardImportText(getHomeLibraryLabel(searchState.libraryTab)).then((rawText) =>
      applyHomeLibraryImport(searchState.libraryTab, rawText, { merge: false })
    ).then(() => {
      setSearchStatus('success', `已从剪贴板导入当前${getHomeLibraryLabel(searchState.libraryTab)}列表，可撤销`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '剪贴板导入失败');
    });
    return;
  }

  const clipboardMergeButton = target.closest('[data-home-library-import-clipboard-merge]');
  if (clipboardMergeButton instanceof HTMLButtonElement) {
    readClipboardImportText(getHomeLibraryLabel(searchState.libraryTab)).then((rawText) =>
      applyHomeLibraryImport(searchState.libraryTab, rawText, { merge: true })
    ).then(() => {
      setSearchStatus('success', `已从剪贴板合并导入当前${getHomeLibraryLabel(searchState.libraryTab)}列表，可撤销`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '剪贴板合并导入失败');
    });
    return;
  }

  const dedupeButton = target.closest('[data-home-library-dedupe]');
  if (dedupeButton instanceof HTMLButtonElement) {
    dedupeHomeLibraryItems(searchState.libraryTab).then(() => {
      setSearchStatus('success', `已完成当前${getHomeLibraryLabel(searchState.libraryTab)}列表去重，可撤销`);
    }).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '列表去重失败');
    });
    return;
  }

  const selectItemButton = target.closest('[data-home-library-select-item]');
  if (selectItemButton instanceof HTMLButtonElement) {
    const [kind, indexText] = String(selectItemButton.dataset.homeLibrarySelectItem || 'history:-1').split(':');
    toggleHomeLibraryItemSelection(kind === 'favorites' ? 'favorites' : 'history', Number.parseInt(indexText || '-1', 10));
    return;
  }

  const removeButton = target.closest('[data-home-library-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    const [kind, indexText] = String(removeButton.dataset.homeLibraryRemove || 'history:-1').split(':');
    removeHomeLibraryItem(kind === 'favorites' ? 'favorites' : 'history', Number.parseInt(indexText || '-1', 10)).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '移除条目失败');
    });
    return;
  }

  const queryButton = target.closest('[data-library-query]');
  if (queryButton instanceof HTMLButtonElement) {
    triggerHomeLibrarySearch(String(queryButton.dataset.libraryQuery || ''));
  }
});

homeLibraryImportFileInput?.addEventListener('change', () => {
  if (!(homeLibraryImportFileInput instanceof HTMLInputElement)) {
    return;
  }
  const [file] = Array.from(homeLibraryImportFileInput.files || []);
  if (!file) {
    return;
  }
  const merge = homeLibraryImportFileInput.dataset.importMode === 'merge';
  importHomeLibraryItems(searchState.libraryTab, file, { merge }).then(() => {
    setSearchStatus('success', merge
      ? `已合并导入当前${getHomeLibraryLabel(searchState.libraryTab)}列表，可撤销`
      : `已导入当前${getHomeLibraryLabel(searchState.libraryTab)}列表，可撤销`);
  }).catch((error) => {
    setSearchStatus('error', error instanceof Error ? error.message : '导入列表失败');
  }).finally(() => {
    delete homeLibraryImportFileInput.dataset.importMode;
    homeLibraryImportFileInput.value = '';
  });
});

homeLibrarySnapshotImportFileInput?.addEventListener('change', () => {
  if (!(homeLibrarySnapshotImportFileInput instanceof HTMLInputElement)) {
    return;
  }
  const [file] = Array.from(homeLibrarySnapshotImportFileInput.files || []);
  if (!file) {
    return;
  }
  const merge = homeLibrarySnapshotImportFileInput.dataset.importMode === 'merge';
  importHomeLibrarySnapshots(searchState.libraryTab, file, { merge }).then(() => {
    setSearchStatus('success', merge
      ? `已合并导入${getHomeLibraryLabel(searchState.libraryTab)}快照`
      : `已导入${getHomeLibraryLabel(searchState.libraryTab)}快照`);
  }).catch((error) => {
    setSearchStatus('error', error instanceof Error ? error.message : '导入快照失败');
  }).finally(() => {
    delete homeLibrarySnapshotImportFileInput.dataset.importMode;
    homeLibrarySnapshotImportFileInput.value = '';
  });
});

historyListEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const removeButton = target.closest('[data-search-history-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    const query = removeButton.dataset.searchHistoryRemove || '';
    removeSearchHistoryItem(query);
    setSearchStatus('success', `已移除搜索历史「${query}」`);
    return;
  }

  const queryButton = target.closest('[data-search-history-query]');
  if (!(queryButton instanceof HTMLButtonElement)) {
    return;
  }
  const query = queryButton.dataset.searchHistoryQuery || '';
  if (!query) {
    return;
  }
  searchInput.value = query;
  const params = new URLSearchParams(window.location.search);
  params.set('q', query);
  window.history.replaceState({}, '', `${window.location.pathname}?${params.toString()}`);
  runSearch(query).catch((error) => {
    setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
    progressEl.textContent = '搜索失败';
  });
});

searchInput?.addEventListener('focus', () => {
  openSearchHistoryDropdown();
});

searchInput?.addEventListener('input', () => {
  searchState.historyHighlightedIndex = -1;
  if (searchState.historyDropdownOpen) {
    renderSearchHistoryDropdown();
  }
});

searchInput?.addEventListener('blur', () => {
  closeSearchHistoryDropdown();
});

searchInput?.addEventListener('keydown', (event) => {
  const history = filterSearchHistoryItems(searchInput?.value || '');
  if (!history.length || !searchState.historyDropdownOpen) {
    return;
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    searchState.historyHighlightedIndex = searchState.historyHighlightedIndex < history.length - 1
      ? searchState.historyHighlightedIndex + 1
      : 0;
    renderSearchHistoryDropdown();
    return;
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault();
    searchState.historyHighlightedIndex = searchState.historyHighlightedIndex > 0
      ? searchState.historyHighlightedIndex - 1
      : history.length - 1;
    renderSearchHistoryDropdown();
    return;
  }

  if (event.key === 'Enter' && searchState.historyHighlightedIndex >= 0) {
    const item = history[searchState.historyHighlightedIndex];
    if (item?.query) {
      event.preventDefault();
      searchInput.value = item.query;
      replaceSearchUrl(item.query);
      searchState.historySubmitLockUntil = Date.now() + 800;
      setSearchHistoryDropdown(false);
      searchInput.blur();
      runSearch(item.query).catch((error) => {
        setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
        progressEl.textContent = '搜索失败';
      });
    }
    return;
  }

  if (event.key === 'Escape') {
    setSearchHistoryDropdown(false);
    searchInput.blur();
  }
});

historyDropdownEl?.addEventListener('mousedown', (event) => {
  event.preventDefault();
});

historyDropdownEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const clearAllButton = target.closest('[data-search-history-clear-all]');
  if (clearAllButton instanceof HTMLButtonElement) {
    clearSearchHistory();
    setSearchStatus('success', '已清空搜索历史');
    return;
  }

  const removeButton = target.closest('[data-search-history-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    const query = removeButton.dataset.searchHistoryRemove || '';
    removeSearchHistoryItem(query);
    setSearchStatus('success', `已移除搜索历史「${query}」`);
    return;
  }

  const queryButton = target.closest('[data-search-history-query]');
  if (queryButton instanceof HTMLButtonElement) {
    const query = queryButton.dataset.searchHistoryQuery || '';
    if (!query) {
      return;
    }
    searchInput.value = query;
    replaceSearchUrl(query);
    searchState.historySubmitLockUntil = Date.now() + 800;
    setSearchHistoryDropdown(false);
    searchInput.blur();
    runSearch(query).catch((error) => {
      setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
      progressEl.textContent = '搜索失败';
    });
  }
});

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    toggleHomeLibraryDrawer(false);
  }
});

renderDisplayToggle();
setHomeLibraryTab('history');
if (sortSelectEl instanceof HTMLSelectElement) {
  sortSelectEl.value = searchState.sortBy;
}
searchState.settingsSignature = getSearchSettingsSignature(initialSettings);
syncSearchSettings(readStoredSearchSettings(), { rerun: false });
renderSearchHistory();

window.addEventListener('kvideo:bootstrap-ready', () => {
  syncSearchSettings(readStoredSearchSettings());
});

window.addEventListener('kvideo:storage-updated', (event) => {
  const detail = event && typeof event === 'object' && 'detail' in event ? event.detail : null;
  if (!detail || detail.key !== 'kvideo-settings') {
    return;
  }
  syncSearchSettings(readStoredSearchSettings());
});

applyHomeLibraryShareFromUrl().catch((error) => {
  setSearchStatus('error', error instanceof Error ? error.message : '处理分享链接失败');
});

applyHomeLibrarySnapshotShareFromUrl().catch((error) => {
  setSearchStatus('error', error instanceof Error ? error.message : '处理快照分享链接失败');
});

if (initialQuery) {
  runSearch(initialQuery).catch((error) => {
    setSearchStatus('error', error instanceof Error ? error.message : '搜索失败');
    progressEl.textContent = '搜索失败';
  });
}
"#;

pub(super) const DISCOVERY_SCRIPT: &str = r#"
const discoveryToggleEl = document.getElementById('content-type-toggle');
const discoveryTagsEl = document.getElementById('discovery-tags');
const discoveryResultsEl = document.getElementById('discovery-results');
const discoveryTagCountEl = document.getElementById('discovery-tag-count');
const searchInputEl = document.getElementById('search-query');

const discoveryState = {
  contentType: 'movie',
  selectedTag: 'recommend',
  tags: [],
  subjects: [],
  loading: false,
};

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function setDiscoveryStatus(message) {
  if (discoveryTagCountEl) {
    discoveryTagCountEl.textContent = message;
  }
}

function normalizeTag(rawTag) {
  return {
    id: rawTag && typeof rawTag.id === 'string' ? rawTag.id : `tag_${String(rawTag?.label || rawTag?.value || '')}`,
    label: String(rawTag?.label || rawTag?.value || '未命名标签'),
    value: String(rawTag?.value || rawTag?.label || ''),
  };
}

function currentTagValue() {
  const current = discoveryState.tags.find((tag) => tag.id === discoveryState.selectedTag);
  return current ? current.value : '热门';
}

function renderDiscoveryToggle() {
  const buttons = discoveryToggleEl?.querySelectorAll('[data-content-type]') || [];
  buttons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.contentType === discoveryState.contentType);
  });
}

function renderDiscoveryTags() {
  setDiscoveryStatus(`标签 ${discoveryState.tags.length}`);

  if (!discoveryState.tags.length) {
    discoveryTagsEl.className = 'tag-cloud empty-state';
    discoveryTagsEl.textContent = '当前没有可用豆瓣标签';
    return;
  }

  discoveryTagsEl.className = 'tag-cloud';
  discoveryTagsEl.innerHTML = discoveryState.tags.map((tag) => `
    <button
      class="tag-chip ${tag.id === discoveryState.selectedTag ? 'active' : ''}"
      type="button"
      data-discovery-tag="${escapeHtml(tag.id)}"
    >
      ${escapeHtml(tag.label)}
    </button>`).join('');
}

function renderDiscoveryResults() {
  if (!discoveryState.subjects.length) {
    discoveryResultsEl.className = 'results-grid empty-state';
    discoveryResultsEl.textContent = '当前标签下暂无推荐内容';
    return;
  }

  discoveryResultsEl.className = 'results-grid';
  discoveryResultsEl.innerHTML = discoveryState.subjects.map((subject) => {
    const title = subject.title || subject.rate || '未知条目';
    const poster = subject.cover
      ? `<img class="result-poster" src="${escapeHtml(subject.cover)}" alt="${escapeHtml(title)}" referrerpolicy="no-referrer" />`
      : '<div class="result-poster placeholder">🎬</div>';
    return `
      <button class="result-card result-card-button" type="button" data-discovery-title="${escapeHtml(subject.title || '')}">
        ${poster}
        <div class="stack compact-card-body">
          <strong>${escapeHtml(subject.title || '未知标题')}</strong>
          <div class="row wrap gap-sm">
            ${subject.rate ? `<span class="chip">豆瓣 ${escapeHtml(subject.rate)}</span>` : ''}
            <span class="chip">${discoveryState.contentType === 'movie' ? '电影' : '电视剧'}</span>
          </div>
          <p class="muted">${escapeHtml(subject.year || subject.tag || '点击后带回搜索')}</p>
        </div>
      </button>`;
  }).join('');
}

async function loadDiscoveryTags() {
  discoveryState.loading = true;
  discoveryTagsEl.className = 'tag-cloud empty-state';
  discoveryTagsEl.textContent = '正在加载豆瓣标签...';

  const response = await fetch(`/api/douban/tags?type=${encodeURIComponent(discoveryState.contentType)}`);
  const data = await response.json().catch(() => ({}));

  if (!response.ok) {
    discoveryState.tags = [];
    renderDiscoveryTags();
    throw new Error(data.error || '加载豆瓣标签失败');
  }

  const rawTags = Array.isArray(data.tags) ? data.tags : [];
  discoveryState.tags = [
    { id: 'recommend', label: '热门', value: '热门' },
    ...rawTags.map((tag) => normalizeTag(tag)),
  ];

  if (!discoveryState.tags.find((tag) => tag.id === discoveryState.selectedTag)) {
    discoveryState.selectedTag = 'recommend';
  }

  renderDiscoveryTags();
}

async function loadDiscoveryResults() {
  discoveryResultsEl.className = 'results-grid empty-state';
  discoveryResultsEl.textContent = '正在加载推荐内容...';

  const response = await fetch(
    `/api/douban/recommend?type=${encodeURIComponent(discoveryState.contentType)}&tag=${encodeURIComponent(currentTagValue())}&page_limit=20&page_start=0`
  );
  const data = await response.json().catch(() => ({}));

  if (!response.ok) {
    discoveryState.subjects = [];
    renderDiscoveryResults();
    throw new Error(data.error || '加载豆瓣推荐失败');
  }

  discoveryState.subjects = Array.isArray(data.subjects) ? data.subjects : [];
  renderDiscoveryResults();
}

async function refreshDiscovery() {
  renderDiscoveryToggle();
  setDiscoveryStatus('标签加载中');
  await loadDiscoveryTags();
  await loadDiscoveryResults();
}

discoveryToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-content-type]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const nextType = button.dataset.contentType;
  if (!nextType || nextType === discoveryState.contentType) {
    return;
  }

  discoveryState.contentType = nextType;
  discoveryState.selectedTag = 'recommend';
  discoveryState.subjects = [];
  refreshDiscovery().catch((error) => {
    setDiscoveryStatus(error instanceof Error ? error.message : '刷新豆瓣推荐失败');
  });
});

discoveryTagsEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-discovery-tag]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const tagId = button.dataset.discoveryTag;
  if (!tagId || tagId === discoveryState.selectedTag) {
    return;
  }

  discoveryState.selectedTag = tagId;
  renderDiscoveryTags();
  loadDiscoveryResults().catch((error) => {
    setDiscoveryStatus(error instanceof Error ? error.message : '加载豆瓣推荐失败');
  });
});

discoveryResultsEl?.addEventListener('click', async (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-discovery-title]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const title = button.dataset.discoveryTitle?.trim();
  if (!title) {
    return;
  }

  if (searchInputEl instanceof HTMLInputElement) {
    searchInputEl.value = title;
  }

  const params = new URLSearchParams(window.location.search);
  params.set('q', title);
  const nextUrl = `${window.location.pathname}?${params.toString()}`;
  window.history.replaceState({}, '', nextUrl);

  const searchRunner = window.kvideoRustRunSearch;
  if (typeof searchRunner === 'function') {
    await searchRunner(title);
  }
});

refreshDiscovery().catch((error) => {
  discoveryTagsEl.className = 'tag-cloud empty-state';
  discoveryTagsEl.textContent = '无法加载豆瓣标签';
  discoveryResultsEl.className = 'results-grid empty-state';
  discoveryResultsEl.textContent = '无法加载推荐内容';
  setDiscoveryStatus(error instanceof Error ? error.message : '初始化豆瓣推荐失败');
});
"#;

pub(super) const DETAIL_SCRIPT: &str = r#"
const detailState = JSON.parse(document.getElementById('detail-state').textContent || '{}');
const summaryEl = document.getElementById('detail-summary');
const episodesEl = document.getElementById('episode-list');
const favoriteToggleEl = document.getElementById('detail-favorite-toggle');
const favoriteStatusEl = document.getElementById('detail-favorite-status');

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function episodeUrl(index) {
  const params = new URLSearchParams({
    id: String(detailState.videoId || ''),
    source: String(detailState.source || ''),
    title: String(detailState.title || '未知视频'),
    episode: String(index),
  });
  if (Array.isArray(detailState.groupedSources) && detailState.groupedSources.length > 1) {
    params.set('groupedSources', JSON.stringify(detailState.groupedSources));
  }
  if (detailState.premium) {
    params.set('premium', '1');
  }
  return `/player?${params.toString()}`;
}

function getFavorites() {
  return Array.isArray(detailState.favoritesData) ? detailState.favoritesData : [];
}

function isFavorite() {
  return getFavorites().some((item) =>
    item
    && String(item.videoId ?? '') === String(detailState.videoId ?? '')
    && String(item.source ?? '') === String(detailState.source ?? '')
  );
}

function renderFavoriteState() {
  const favorite = isFavorite();
  favoriteToggleEl.textContent = favorite ? '取消收藏' : '加入收藏';
  favoriteStatusEl.textContent = favorite ? '当前视频已在收藏中' : '当前视频尚未收藏';
  favoriteStatusEl.className = `status ${favorite ? 'success' : 'muted'}`;
}

async function persistFavorites() {
  await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: detailState.favoritesKey, value: detailState.favoritesData || [] }),
  });
}

async function toggleFavorite() {
  if (!detailState.detail) {
    return;
  }

  const favorites = getFavorites();
  const index = favorites.findIndex((item) =>
    item
    && String(item.videoId ?? '') === String(detailState.videoId ?? '')
    && String(item.source ?? '') === String(detailState.source ?? '')
  );

  if (index >= 0) {
    favorites.splice(index, 1);
  } else {
    favorites.unshift({
      videoId: detailState.videoId,
      title: detailState.detail.vod_name || detailState.title || '未知视频',
      poster: detailState.detail.vod_pic || undefined,
      source: detailState.source,
      sourceName: detailState.source,
      addedAt: Date.now(),
      type: detailState.detail.type_name || undefined,
      year: detailState.detail.vod_year || undefined,
      remarks: detailState.detail.vod_remarks || undefined,
    });
  }

  detailState.favoritesData = favorites.slice(0, 100);
  await persistFavorites();
  renderFavoriteState();
}

async function loadDetail() {
  const response = detailState.sourceConfig
    ? await fetch('/api/detail', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: detailState.videoId, source: detailState.sourceConfig }),
      })
    : await fetch(`/api/detail?id=${encodeURIComponent(detailState.videoId)}&source=${encodeURIComponent(detailState.source)}`);
  const data = await response.json().catch(() => ({}));

  if (!response.ok || !data.success || !data.data) {
    summaryEl.className = 'empty-state';
    summaryEl.textContent = data.error || '加载详情失败';
    episodesEl.className = 'episodes-grid empty-state';
    episodesEl.textContent = '无法加载选集';
    return;
  }

  const detail = data.data;
  detailState.detail = detail;
  const poster = detail.vod_pic
    ? `<img class="detail-poster" src="${escapeHtml(detail.vod_pic)}" alt="${escapeHtml(detail.vod_name || detailState.title)}" referrerpolicy="no-referrer" />`
    : '<div class="detail-poster placeholder">🎬</div>';

  summaryEl.className = 'detail-summary';
  summaryEl.innerHTML = `
    <div class="detail-summary-grid">
      ${poster}
      <div class="stack">
        <h2>${escapeHtml(detail.vod_name || detailState.title || '未知视频')}</h2>
        <div class="chip-list">
          ${detail.type_name ? `<span class="chip">${escapeHtml(detail.type_name)}</span>` : ''}
          ${detail.vod_year ? `<span class="chip">${escapeHtml(detail.vod_year)}</span>` : ''}
          ${detail.vod_area ? `<span class="chip">${escapeHtml(detail.vod_area)}</span>` : ''}
          <span class="chip">${escapeHtml(detail.source_code || detailState.source || '未知来源')}</span>
        </div>
        <p class="muted">${escapeHtml(detail.vod_content || '暂无简介')}</p>
        <div class="stack metadata-list">
          ${detail.vod_actor ? `<div><strong>演员：</strong>${escapeHtml(detail.vod_actor)}</div>` : ''}
          ${detail.vod_director ? `<div><strong>导演：</strong>${escapeHtml(detail.vod_director)}</div>` : ''}
          ${detail.vod_remarks ? `<div><strong>备注：</strong>${escapeHtml(detail.vod_remarks)}</div>` : ''}
        </div>
      </div>
    </div>`;

  const episodes = Array.isArray(detail.episodes) ? detail.episodes : [];
  if (!episodes.length) {
    episodesEl.className = 'episodes-grid empty-state';
    episodesEl.textContent = '当前没有可播放选集';
    return;
  }

  episodesEl.className = 'episodes-grid';
  episodesEl.innerHTML = episodes.map((episode, index) => `
    <a class="episode-item" href="${episodeUrl(index)}">
      <strong>${escapeHtml(episode.name || `第${index + 1}集`)}</strong>
      <span class="muted">打开播放器</span>
    </a>`).join('');

  renderFavoriteState();
}

favoriteToggleEl?.addEventListener('click', () => {
  toggleFavorite().catch((error) => {
    favoriteStatusEl.textContent = error instanceof Error ? error.message : '收藏操作失败';
    favoriteStatusEl.className = 'status error';
  });
});

loadDetail().catch((error) => {
  summaryEl.className = 'empty-state';
  summaryEl.textContent = error instanceof Error ? error.message : '加载详情失败';
  episodesEl.className = 'episodes-grid empty-state';
  episodesEl.textContent = '无法加载选集';
  favoriteStatusEl.textContent = '无法读取收藏状态';
  favoriteStatusEl.className = 'status error';
});
"#;

pub(super) const ADMIN_SCRIPT: &str = r#"
const createForm = document.getElementById('create-user-form');
const passwordForm = document.getElementById('admin-password-form');
const refreshButton = document.getElementById('refresh-users');
const usersEl = document.getElementById('admin-users');
const countEl = document.getElementById('admin-users-count');
const statusEl = document.getElementById('admin-status');
const passwordStatusEl = document.getElementById('admin-password-status');

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function setStatus(target, kind, message) {
  target.textContent = message;
  target.className = `status ${kind}`;
}

function renderUsers(users) {
  countEl.textContent = `用户 ${users.length}`;

  if (!Array.isArray(users) || users.length === 0) {
    usersEl.className = 'user-grid empty-state';
    usersEl.textContent = '当前还没有可管理的用户';
    return;
  }

  usersEl.className = 'user-grid';
  usersEl.innerHTML = users.map((user) => `
    <article class="user-card" data-user-id="${user.id}">
      <div class="row space-between wrap gap-sm">
        <div class="stack compact-card-body">
          <strong>${escapeHtml(user.username)}</strong>
          <div class="row wrap gap-sm">
            <span class="chip">${user.isAdmin ? '管理员' : '普通用户'}</span>
            <span class="chip">${user.disablePremium ? 'Premium 已禁用' : 'Premium 已启用'}</span>
          </div>
          <span class="muted tiny">创建时间：${escapeHtml(user.createdAt || '')}</span>
        </div>
      </div>

      <div class="stack form-grid">
        <label class="field">
          <span>用户名</span>
          <input class="user-name-input" value="${escapeHtml(user.username)}" ${user.isAdmin ? 'disabled' : ''} />
        </label>
        <label class="field">
          <span>重置密码（留空则不修改）</span>
          <input class="user-password-input" type="password" autocomplete="new-password" placeholder="至少 6 位" ${user.isAdmin ? 'disabled' : ''} />
        </label>
        <label class="checkbox-row ${user.isAdmin ? 'is-disabled' : ''}">
          <input class="user-disable-premium" type="checkbox" ${user.disablePremium ? 'checked' : ''} ${user.isAdmin ? 'disabled' : ''} />
          <span>禁用 Premium 内容</span>
        </label>
      </div>

      <div class="row wrap gap-sm">
        <button class="button primary" data-action="save" ${user.isAdmin ? 'disabled' : ''}>保存</button>
        <button class="button danger" data-action="delete" ${user.isAdmin ? 'disabled' : ''}>删除</button>
      </div>
    </article>`).join('');
}

async function fetchUsers() {
  usersEl.className = 'user-grid empty-state';
  usersEl.textContent = '正在加载用户列表...';

  const response = await fetch('/api/admin/users');
  if (response.status === 403) {
    window.location.href = '/';
    return;
  }

  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setStatus(statusEl, 'error', data.error || '获取用户列表失败');
    usersEl.className = 'user-grid empty-state';
    usersEl.textContent = '无法加载用户列表';
    return;
  }

  renderUsers(data.users || []);
}

createForm?.addEventListener('submit', async (event) => {
  event.preventDefault();
  const username = document.getElementById('create-username').value.trim();
  const password = document.getElementById('create-password').value;
  const disablePremium = document.getElementById('create-disable-premium').checked;

  const response = await fetch('/api/admin/users', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password, disablePremium }),
  });
  const data = await response.json().catch(() => ({}));

  if (!response.ok) {
    setStatus(statusEl, 'error', data.error || '创建用户失败');
    return;
  }

  createForm.reset();
  document.getElementById('create-disable-premium').checked = true;
  setStatus(statusEl, 'success', `用户 ${username} 创建成功`);
  await fetchUsers();
});

passwordForm?.addEventListener('submit', async (event) => {
  event.preventDefault();
  const currentPassword = document.getElementById('admin-current-password').value;
  const newPassword = document.getElementById('admin-new-password').value;

  const response = await fetch('/api/auth/password', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ currentPassword, newPassword }),
  });
  const data = await response.json().catch(() => ({}));

  if (!response.ok) {
    setStatus(passwordStatusEl, 'error', data.error || '修改密码失败');
    return;
  }

  passwordForm.reset();
  setStatus(passwordStatusEl, 'success', data.message || '密码已更新');
});

refreshButton?.addEventListener('click', () => {
  fetchUsers().catch((error) => setStatus(statusEl, 'error', error instanceof Error ? error.message : '刷新失败'));
});

usersEl?.addEventListener('click', async (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest('[data-action]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  const card = button.closest('[data-user-id]');
  if (!(card instanceof HTMLElement)) {
    return;
  }

  const userId = card.dataset.userId;
  if (!userId) {
    return;
  }

  if (button.dataset.action === 'delete') {
    const username = card.querySelector('.user-name-input')?.value || '该用户';
    if (!window.confirm(`确定删除用户「${username}」？`)) {
      return;
    }

    const response = await fetch(`/api/admin/users/${userId}`, { method: 'DELETE' });
    const data = await response.json().catch(() => ({}));
    if (!response.ok) {
      setStatus(statusEl, 'error', data.error || '删除用户失败');
      return;
    }

    setStatus(statusEl, 'success', `用户 ${username} 已删除`);
    await fetchUsers();
    return;
  }

  const usernameInput = card.querySelector('.user-name-input');
  const passwordInput = card.querySelector('.user-password-input');
  const disablePremiumInput = card.querySelector('.user-disable-premium');

  if (!(usernameInput instanceof HTMLInputElement) || !(passwordInput instanceof HTMLInputElement) || !(disablePremiumInput instanceof HTMLInputElement)) {
    return;
  }

  const payload = {
    username: usernameInput.value.trim(),
    disablePremium: disablePremiumInput.checked,
  };
  if (passwordInput.value.trim()) {
    payload.password = passwordInput.value.trim();
  }

  const response = await fetch(`/api/admin/users/${userId}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setStatus(statusEl, 'error', data.error || '更新用户失败');
    return;
  }

  passwordInput.value = '';
  setStatus(statusEl, 'success', `用户 ${payload.username || userId} 已更新`);
  await fetchUsers();
});

fetchUsers().catch((error) => {
  setStatus(statusEl, 'error', error instanceof Error ? error.message : '获取用户列表失败');
  usersEl.className = 'user-grid empty-state';
  usersEl.textContent = '无法加载用户列表';
});
"#;

pub(super) const PLAYER_SCRIPT: &str = r#"
const playerState = JSON.parse(document.getElementById('player-state').textContent || '{}');
const videoEl = document.getElementById('player-video');
const statusEl = document.getElementById('player-status');
const favoriteToggleEl = document.getElementById('player-favorite-toggle');
const favoriteStatusEl = document.getElementById('player-favorite-status');
const metadataEl = document.getElementById('player-metadata');
const episodesEl = document.getElementById('player-episodes');
const episodeCountEl = document.getElementById('episode-count');
const proxyButton = document.getElementById('toggle-proxy');
const reloadButton = document.getElementById('reload-player');
const prevButton = document.getElementById('prev-episode');
const nextButton = document.getElementById('next-episode');
const seekBackwardButton = document.getElementById('seek-backward');
const seekForwardButton = document.getElementById('seek-forward');
const fullscreenButton = document.getElementById('toggle-fullscreen');
const pipButton = document.getElementById('toggle-pip');
const remotePlaybackButton = document.getElementById('remote-playback');
const copyStreamButton = document.getElementById('copy-stream-url');
const playerPreferencesButton = document.getElementById('toggle-player-preferences');
const playerPreferencesPanelEl = document.getElementById('player-preferences-panel');
const closePlayerPreferencesButton = document.getElementById('close-player-preferences');
const playerPrefFullscreenTypeEl = document.getElementById('player-pref-fullscreen-type');
const playerPrefAdFilterModeEl = document.getElementById('player-pref-ad-filter-mode');
const playerPrefShowModeIndicatorEl = document.getElementById('player-pref-show-mode-indicator');
const playerPrefAutoNextEpisodeEl = document.getElementById('player-pref-auto-next-episode');
const playerPrefAutoSwitchSourceOnFailureEl = document.getElementById('player-pref-auto-switch-source-on-failure');
const playerPrefAutoSkipIntroEl = document.getElementById('player-pref-auto-skip-intro');
const playerPrefSkipIntroSecondsEl = document.getElementById('player-pref-skip-intro-seconds');
const playerPrefAutoSkipOutroEl = document.getElementById('player-pref-auto-skip-outro');
const playerPrefSkipOutroSecondsEl = document.getElementById('player-pref-skip-outro-seconds');
const playerPrefClearSourcePreferenceButton = document.getElementById('player-pref-clear-source-preference');
const sourceDiagnosticsEl = document.getElementById('player-source-diagnostics');
const playerPrefClearFailedTrailButton = document.getElementById('player-pref-clear-failed-trail');
const playerPrefCopySourceDiagnosticsButton = document.getElementById('player-pref-copy-source-diagnostics');
const retryDiagnosticsEl = document.getElementById('player-retry-diagnostics');
const playerPrefForceDirectButton = document.getElementById('player-pref-force-direct');
const playerPrefForceProxyButton = document.getElementById('player-pref-force-proxy');
const playerPrefResetRetryStateButton = document.getElementById('player-pref-reset-retry-state');
const playerPrefCopyRetryDiagnosticsButton = document.getElementById('player-pref-copy-retry-diagnostics');
const playerPrefCopyPlaybackDiagnosticsJsonButton = document.getElementById('player-pref-copy-playback-diagnostics-json');
const playerPrefExportPlaybackDiagnosticsButton = document.getElementById('player-pref-export-playback-diagnostics');
const playerBookmarksSummaryEl = document.getElementById('player-bookmarks-summary');
const playerBookmarksListEl = document.getElementById('player-bookmarks-list');
const playerPrefSaveBookmarkButton = document.getElementById('player-pref-save-bookmark');
const playerPrefClearBookmarksButton = document.getElementById('player-pref-clear-bookmarks');
const playerPrefCopyPageLinkButton = document.getElementById('player-pref-copy-page-link');
const playerPrefCopyOriginalLinkButton = document.getElementById('player-pref-copy-original-link');
const playerPrefCopyProxyLinkButton = document.getElementById('player-pref-copy-proxy-link');
const playerPrefCopyActiveLinkButton = document.getElementById('player-pref-copy-active-link');
const modeBadge = document.getElementById('playback-mode-badge');
const resumeButton = document.getElementById('resume-playback');
const playbackRateSelectEl = document.getElementById('playback-rate-select');
const progressIndicatorEl = document.getElementById('player-progress-indicator');
const shortcutsEl = document.getElementById('player-shortcuts');
const skipBackwardIndicatorEl = document.getElementById('skip-backward-indicator');
const skipForwardIndicatorEl = document.getElementById('skip-forward-indicator');
const libraryEl = document.getElementById('player-library');
const libraryCountEl = document.getElementById('player-library-count');
const clearLibraryButton = document.getElementById('clear-library');
const sourceCountEl = document.getElementById('player-source-count');
const sourceListEl = document.getElementById('player-sources');
const refreshSourcesButton = document.getElementById('refresh-sources');

const settings = playerState.settings && typeof playerState.settings === 'object' ? playerState.settings : {};
const SOURCE_PREFERENCES_STORAGE_KEY = 'kvideo-source-preferences';
const PLAYER_BOOKMARKS_STORAGE_KEY = 'kvideo-playback-bookmarks';
const storedPlaybackRate = typeof window !== 'undefined'
  ? Number(window.localStorage.getItem('kvideo-playback-rate') || '1')
  : 1;
const isiOSDevice = (() => {
  if (typeof navigator === 'undefined') {
    return false;
  }
  const userAgent = navigator.userAgent || '';
  const platform = navigator.platform || '';
  const maxTouchPoints = Number(navigator.maxTouchPoints || 0);
  return /iPad|iPhone|iPod/i.test(userAgent)
    || (/Mac/i.test(platform) && maxTouchPoints > 1);
})();

if (videoEl instanceof HTMLVideoElement) {
  videoEl.setAttribute('playsinline', 'true');
  videoEl.setAttribute('webkit-playsinline', 'true');
  videoEl.setAttribute('x-webkit-airplay', 'allow');
}
const detailState = {
  detail: null,
  episodes: [],
  currentEpisode: Number(playerState.episode || 0),
  useProxy: settings.proxyMode === 'always',
  retryCount: 0,
  saveTimer: 0,
  lastSavedAt: 0,
  hls: null,
  hlsBootstrapTimer: 0,
  hlsBootstrapAttempts: 0,
  nativeHlsBlobUrls: [],
  nativeHlsAbortController: null,
  nativeHlsAbortReason: '',
  nativeHlsPrepareTimeoutId: 0,
  resumePosition: 0,
  resumeApplied: false,
  playbackRate: Number.isFinite(storedPlaybackRate) && storedPlaybackRate > 0 ? storedPlaybackRate : 1,
  libraryTab: 'history',
  sourceResults: [],
  introSkipped: false,
  outroHandled: false,
  currentPlaybackUrl: '',
  currentRawPlaybackUrl: '',
  playbackRequestToken: 0,
  windowFullscreen: false,
  playbackPollingTimer: 0,
  lastPolledTime: 0,
  lastPolledPaused: true,
  skipIndicatorTimer: 0,
  skipForwardAmount: 0,
  skipBackwardAmount: 0,
  lastTapAt: 0,
  lastTapSide: '',
  castReady: false,
  castDevicesAvailable: false,
  castingActive: false,
  castListenersBound: false,
  airplayAvailable: false,
  failedSources: [],
};
detailState.failedSources = loadFailedSourceTrail();

function getGroupedSources() {
  return Array.isArray(playerState.groupedSources)
    ? playerState.groupedSources.filter((item) => item && typeof item === 'object' && typeof item.source === 'string')
    : [];
}

function getNavigationGroupedSources() {
  const sourceResults = Array.isArray(detailState.sourceResults)
    ? detailState.sourceResults
        .filter((item) => item && typeof item === 'object' && typeof item.source === 'string')
        .map((item) => ({
          id: item?.vod_id ?? item?.videoId ?? '',
          source: item?.source ?? '',
          sourceName: item?.sourceDisplayName || item?.sourceName || item?.source || '',
          latency: typeof item?.latency === 'number' ? item.latency : undefined,
          pic: item?.vod_pic || item?.pic || undefined,
        }))
    : [];
  if (sourceResults.length > 1) {
    return sourceResults;
  }
  return getGroupedSources();
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('\"', '&quot;')
    .replaceAll("'", '&#39;');
}

function getNormalizedLibrarySort(value) {
  switch (String(value || '')) {
    case 'recent-asc':
    case 'title-asc':
    case 'title-desc':
    case 'source-asc':
    case 'source-desc':
      return String(value);
    case 'recent-desc':
    default:
      return 'recent-desc';
  }
}

function getLibrarySortLabel(value) {
  switch (getNormalizedLibrarySort(value)) {
    case 'recent-asc':
      return '最早添加';
    case 'title-asc':
      return '标题 A-Z';
    case 'title-desc':
      return '标题 Z-A';
    case 'source-asc':
      return '来源 A-Z';
    case 'source-desc':
      return '来源 Z-A';
    case 'recent-desc':
    default:
      return '最近添加';
  }
}

function sortLibraryEntries(entries, sortBy) {
  const normalizedSort = getNormalizedLibrarySort(sortBy);
  return [...entries].sort((left, right) => {
    const leftTitle = String(left?.item?.title || '').trim();
    const rightTitle = String(right?.item?.title || '').trim();
    const leftSource = String(left?.item?.sourceName || left?.item?.source || '').trim();
    const rightSource = String(right?.item?.sourceName || right?.item?.source || '').trim();

    switch (normalizedSort) {
      case 'recent-asc':
        return left.index - right.index;
      case 'title-asc':
        return leftTitle.localeCompare(rightTitle, 'zh-CN') || left.index - right.index;
      case 'title-desc':
        return rightTitle.localeCompare(leftTitle, 'zh-CN') || left.index - right.index;
      case 'source-asc':
        return leftSource.localeCompare(rightSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'source-desc':
        return rightSource.localeCompare(leftSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'recent-desc':
      default:
        return right.index - left.index;
    }
  });
}

function encodeLibrarySharePackage(payload) {
  const json = JSON.stringify(payload);
  const utf8 = encodeURIComponent(json).replace(/%([0-9A-F]{2})/g, (_, hex) =>
    String.fromCharCode(Number.parseInt(hex, 16))
  );
  return btoa(utf8).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/g, '');
}

function decodeLibrarySharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://library/',
    'kvideo://library-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  const json = decodeURIComponent(percentEncoded);
  return JSON.parse(json);
}

function buildLibraryShareUrl(pathname, kind, merge = false) {
  const sharePackage = pathname === '/premium'
    ? buildPremiumLibrarySharePackage(kind)
    : buildHomeLibrarySharePackage(kind);
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  return `${window.location.origin}${pathname}?${params.toString()}`;
}

function readLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  const libraryShare = String(params.get('libraryShare') || '').trim();
  if (!libraryShare) {
    return null;
  }
  return {
    rawValue: libraryShare,
    merge: params.get('libraryShareMode') === 'merge',
  };
}

function clearLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('libraryShare');
  params.delete('libraryShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function setStatus(kind, message) {
  statusEl.textContent = message;
  statusEl.className = `status ${kind}`;
}

async function copyText(value) {
  const text = String(value || '').trim();
  if (!text) {
    throw new Error('当前没有可复制的播放地址');
  }

  if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', 'readonly');
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand('copy');
  document.body.removeChild(textarea);
}

function formatTime(totalSeconds) {
  if (!Number.isFinite(totalSeconds) || totalSeconds < 0) {
    return '00:00';
  }
  const wholeSeconds = Math.floor(totalSeconds);
  const hours = Math.floor(wholeSeconds / 3600);
  const minutes = Math.floor((wholeSeconds % 3600) / 60);
  const seconds = wholeSeconds % 60;
  if (hours > 0) {
    return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
  }
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

function getCurrentEpisodeLabel() {
  const episode = Array.isArray(detailState.episodes) ? detailState.episodes[detailState.currentEpisode] : null;
  return episode?.name || `第${Number(detailState.currentEpisode || 0) + 1}集`;
}

function getNonNegativeSettingNumber(value) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return 0;
  }
  return parsed;
}

function persistPlayerSettings() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem('kvideo-settings', JSON.stringify(settings));
}

function loadFailedSourceTrail() {
  const params = new URLSearchParams(window.location.search);
  const failedSources = [];
  params.getAll('failedSource').forEach((value) => {
    String(value || '')
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean)
      .forEach((item) => {
        if (!failedSources.includes(item)) {
          failedSources.push(item);
        }
      });
  });
  return failedSources;
}

function getPlaybackBookmarkKey() {
  return `${playerState.premium ? 'premium' : 'standard'}:${String(playerState.videoId || '')}:${getSourcePreferenceKey(currentDisplayTitle())}`;
}

function loadAllPlaybackBookmarks() {
  if (typeof window === 'undefined') {
    return {};
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PLAYER_BOOKMARKS_STORAGE_KEY) || '{}');
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch (_) {
    return {};
  }
}

function persistAllPlaybackBookmarks(bookmarks) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(PLAYER_BOOKMARKS_STORAGE_KEY, JSON.stringify(bookmarks));
}

function getCurrentPlaybackBookmarks() {
  const key = getPlaybackBookmarkKey();
  if (!key) {
    return [];
  }
  const allBookmarks = loadAllPlaybackBookmarks();
  const items = Array.isArray(allBookmarks[key]) ? allBookmarks[key] : [];
  return items.filter((item) =>
    item
    && typeof item === 'object'
    && typeof item.id === 'string'
    && Number.isFinite(Number(item.time))
  );
}

function buildPlaybackBookmarkLink(bookmark) {
  const seconds = Math.max(0, Math.floor(Number(bookmark?.time || 0)));
  return `${window.location.origin}${buildRustPlayerUrl(Number(bookmark?.episode ?? detailState.currentEpisode))}#t=${seconds}`;
}

function renderPlaybackBookmarks() {
  const bookmarks = getCurrentPlaybackBookmarks()
    .slice()
    .sort((left, right) => Number(left.time || 0) - Number(right.time || 0));
  if (playerBookmarksSummaryEl instanceof HTMLElement) {
    playerBookmarksSummaryEl.textContent = bookmarks.length
      ? `当前片目书签 ${bookmarks.length} 个`
      : '当前还没有播放书签。';
  }
  if (playerPrefClearBookmarksButton instanceof HTMLButtonElement) {
    playerPrefClearBookmarksButton.disabled = bookmarks.length === 0;
  }
  if (!(playerBookmarksListEl instanceof HTMLElement)) {
    return;
  }
  if (!bookmarks.length) {
    playerBookmarksListEl.className = 'saved-list empty-state';
    playerBookmarksListEl.textContent = '当前还没有播放书签。';
    return;
  }
  playerBookmarksListEl.className = 'saved-list';
  playerBookmarksListEl.innerHTML = bookmarks.map((bookmark, index) => `
    <article class="saved-item library-item">
      <div class="row space-between wrap gap-sm">
        <div class="stack compact-card-body">
          <strong>${escapeHtml(bookmark.label || `书签 ${index + 1}`)}</strong>
          <span class="muted tiny">第 ${Number(bookmark.episode || 0) + 1} 集 · ${formatTime(Number(bookmark.time || 0))}</span>
        </div>
        <div class="row wrap gap-sm">
          <button class="button button-small" type="button" data-player-bookmark-jump="${escapeHtml(bookmark.id)}">跳转</button>
          <button class="button button-small" type="button" data-player-bookmark-copy="${escapeHtml(bookmark.id)}">复制链接</button>
          <button class="button button-small danger" type="button" data-player-bookmark-delete="${escapeHtml(bookmark.id)}">删除</button>
        </div>
      </div>
    </article>
  `).join('');
}

function saveCurrentPlaybackBookmark() {
  if (!(videoEl instanceof HTMLVideoElement)) {
    throw new Error('当前播放器不可用');
  }
  const currentTime = Number(videoEl.currentTime || 0);
  if (!Number.isFinite(currentTime) || currentTime <= 1) {
    throw new Error('请先播放到有效时间点再保存书签');
  }
  const key = getPlaybackBookmarkKey();
  const allBookmarks = loadAllPlaybackBookmarks();
  const existing = Array.isArray(allBookmarks[key]) ? allBookmarks[key] : [];
  const bookmark = {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    label: `${currentDisplayTitle()} · ${formatTime(currentTime)}`,
    episode: detailState.currentEpisode,
    time: Math.floor(currentTime),
    createdAt: new Date().toISOString(),
  };
  const nextBookmarks = [bookmark, ...existing].slice(0, 50);
  allBookmarks[key] = nextBookmarks;
  persistAllPlaybackBookmarks(allBookmarks);
  renderPlaybackBookmarks();
  return bookmark;
}

function removePlaybackBookmark(bookmarkId) {
  const key = getPlaybackBookmarkKey();
  const allBookmarks = loadAllPlaybackBookmarks();
  const existing = Array.isArray(allBookmarks[key]) ? allBookmarks[key] : [];
  const nextBookmarks = existing.filter((item) => item?.id !== bookmarkId);
  if (nextBookmarks.length === existing.length) {
    return null;
  }
  allBookmarks[key] = nextBookmarks;
  persistAllPlaybackBookmarks(allBookmarks);
  renderPlaybackBookmarks();
  return existing.find((item) => item?.id === bookmarkId) || null;
}

function clearCurrentPlaybackBookmarks() {
  const key = getPlaybackBookmarkKey();
  const allBookmarks = loadAllPlaybackBookmarks();
  const existing = Array.isArray(allBookmarks[key]) ? allBookmarks[key] : [];
  if (!existing.length) {
    return 0;
  }
  delete allBookmarks[key];
  persistAllPlaybackBookmarks(allBookmarks);
  renderPlaybackBookmarks();
  return existing.length;
}

function findPlaybackBookmark(bookmarkId) {
  return getCurrentPlaybackBookmarks().find((item) => item?.id === bookmarkId) || null;
}

function jumpToPlaybackBookmark(bookmarkId) {
  const bookmark = findPlaybackBookmark(bookmarkId);
  if (!bookmark) {
    throw new Error('未找到对应的播放书签');
  }
  const targetEpisode = Number(bookmark.episode || 0);
  if (targetEpisode !== detailState.currentEpisode) {
    detailState.resumePosition = Number(bookmark.time || 0);
    detailState.resumeApplied = false;
    loadEpisode(targetEpisode, true);
    return bookmark;
  }
  if (!(videoEl instanceof HTMLVideoElement)) {
    throw new Error('当前播放器不可用');
  }
  videoEl.currentTime = Math.max(0, Number(bookmark.time || 0));
  updateProgressIndicator();
  return bookmark;
}

async function copyPlaybackBookmarkLink(bookmarkId) {
  const bookmark = findPlaybackBookmark(bookmarkId);
  if (!bookmark) {
    throw new Error('未找到对应的播放书签');
  }
  await copyText(buildPlaybackBookmarkLink(bookmark));
  return bookmark;
}

function getHashResumePosition() {
  const hash = String(window.location.hash || '').trim();
  if (!hash.startsWith('#t=')) {
    return 0;
  }
  const value = Number(hash.slice(3));
  return Number.isFinite(value) && value > 0 ? value : 0;
}

function syncPlayerPreferenceControls() {
  if (playerPrefFullscreenTypeEl instanceof HTMLSelectElement) {
    playerPrefFullscreenTypeEl.value = settings.fullscreenType === 'window' ? 'window' : 'native';
  }
  if (playerPrefAdFilterModeEl instanceof HTMLSelectElement) {
    playerPrefAdFilterModeEl.value = ['off', 'keyword', 'heuristic', 'aggressive'].includes(settings.adFilterMode)
      ? settings.adFilterMode
      : (settings.adFilter ? 'heuristic' : 'off');
  }
  if (playerPrefShowModeIndicatorEl instanceof HTMLInputElement) {
    playerPrefShowModeIndicatorEl.checked = Boolean(settings.showModeIndicator);
  }
  if (playerPrefAutoNextEpisodeEl instanceof HTMLInputElement) {
    playerPrefAutoNextEpisodeEl.checked = settings.autoNextEpisode !== false;
  }
  if (playerPrefAutoSwitchSourceOnFailureEl instanceof HTMLInputElement) {
    playerPrefAutoSwitchSourceOnFailureEl.checked = Boolean(settings.autoSwitchSourceOnFailure);
  }
  if (playerPrefAutoSkipIntroEl instanceof HTMLInputElement) {
    playerPrefAutoSkipIntroEl.checked = Boolean(settings.autoSkipIntro);
  }
  if (playerPrefSkipIntroSecondsEl instanceof HTMLInputElement) {
    playerPrefSkipIntroSecondsEl.value = String(getNonNegativeSettingNumber(settings.skipIntroSeconds));
  }
  if (playerPrefAutoSkipOutroEl instanceof HTMLInputElement) {
    playerPrefAutoSkipOutroEl.checked = Boolean(settings.autoSkipOutro);
  }
  if (playerPrefSkipOutroSecondsEl instanceof HTMLInputElement) {
    playerPrefSkipOutroSecondsEl.value = String(getNonNegativeSettingNumber(settings.skipOutroSeconds));
  }
  updateSourcePreferenceButton();
  updateRetryDiagnostics();
  renderPlaybackBookmarks();
}

function setPlayerPreferencesPanel(open) {
  if (!(playerPreferencesPanelEl instanceof HTMLElement)) {
    return;
  }
  playerPreferencesPanelEl.classList.toggle('hidden', !open);
  if (playerPreferencesButton instanceof HTMLButtonElement) {
    playerPreferencesButton.setAttribute('aria-expanded', open ? 'true' : 'false');
  }
}

function updateProgressIndicator() {
  if (!progressIndicatorEl) {
    return;
  }
  progressIndicatorEl.textContent = `${formatTime(videoEl?.currentTime || 0)} / ${formatTime(videoEl?.duration || 0)}`;
}

function getLibraryTabButton(tab) {
  return document.querySelector(`[data-library-tab="${tab}"]`);
}

function currentDisplayTitle() {
  return String(detailState.detail?.vod_name || playerState.title || '未知视频');
}

function hasPreviousEpisode() {
  const previousIndex = detailState.currentEpisode - getEpisodeStep();
  return previousIndex >= 0 && previousIndex < detailState.episodes.length;
}

function hasNextEpisode() {
  const nextIndex = detailState.currentEpisode + getEpisodeStep();
  return nextIndex >= 0 && nextIndex < detailState.episodes.length;
}

function buildRustPlayerUrl(episode) {
  const params = new URLSearchParams({
    id: String(playerState.videoId || ''),
    source: String(playerState.source || ''),
    title: String(playerState.title || '未知视频'),
    episode: String(episode),
  });
  const groupedSources = getNavigationGroupedSources();
  if (groupedSources.length > 1) {
    params.set('groupedSources', JSON.stringify(groupedSources));
  }
  if (playerState.premium) {
    params.set('premium', '1');
  }
  return `/player?${params.toString()}`;
}

function clearFailedSourceTrailFromUrl() {
  if (!detailState.failedSources.length) {
    updateSourceDiagnostics();
    return;
  }
  detailState.failedSources = [];
  updateSourceDiagnostics();
  window.history.replaceState({}, '', buildRustPlayerUrl(detailState.currentEpisode));
}

function getResolvedPlayUrl(rawUrl) {
  if (!rawUrl) {
    return '';
  }
  const shouldFilterAds = shouldFilterAdsForUrl(rawUrl);
  return detailState.useProxy
    || shouldFilterAds
    ? `/api/proxy?url=${encodeURIComponent(rawUrl)}&retry=${detailState.retryCount}`
    : rawUrl;
}

function isHlsUrl(url) {
  const value = String(url || '');
  if (/\.m3u8(?:$|[?#/&])/i.test(value)) {
    return true;
  }

  try {
    const parsed = new URL(value, window.location.href);
    const proxiedUrl = parsed.searchParams.get('url');
    if (proxiedUrl && proxiedUrl !== value) {
      return isHlsUrl(proxiedUrl);
    }
  } catch (_) {
  }

  return false;
}

function shouldFilterAdsForUrl(url) {
  return Boolean(settings.adFilterMode && settings.adFilterMode !== 'off' && isHlsUrl(url));
}

function getAbsoluteUrl(url) {
  try {
    return new URL(String(url || ''), window.location.href).toString();
  } catch (_) {
    return String(url || '');
  }
}

function hasGoogleCastApi() {
  return Boolean(
    window.cast
    && window.cast.framework
    && window.chrome
    && window.chrome.cast
    && window.chrome.cast.media
  );
}

function canUseGoogleCast() {
  return detailState.castDevicesAvailable || detailState.castingActive;
}

function hasAirPlayApi() {
  return Boolean(
    videoEl instanceof HTMLVideoElement
    && typeof videoEl.webkitShowPlaybackTargetPicker === 'function'
    && typeof window !== 'undefined'
    && 'WebKitPlaybackTargetAvailabilityEvent' in window
  );
}

function canUseAirPlay() {
  return Boolean(
    hasAirPlayApi()
    && detailState.airplayAvailable
  );
}

function canUseWebkitPictureInPicture() {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return false;
  }
  if (typeof videoEl.webkitSupportsPresentationMode === 'function') {
    return Boolean(videoEl.webkitSupportsPresentationMode('picture-in-picture'));
  }
  return typeof videoEl.webkitSetPresentationMode === 'function'
    || typeof videoEl.webkitPresentationMode === 'string';
}

function canUsePictureInPicture() {
  return Boolean(
    (document.pictureInPictureEnabled && videoEl instanceof HTMLVideoElement && !videoEl.disablePictureInPicture)
    || canUseWebkitPictureInPicture()
  );
}

function isPictureInPictureActive() {
  return Boolean(
    document.pictureInPictureElement === videoEl
    || (videoEl instanceof HTMLVideoElement && videoEl.webkitPresentationMode === 'picture-in-picture')
  );
}

function updateAirPlayAvailability(availability) {
  detailState.airplayAvailable = availability === true || availability === 'available';
  updatePlaybackButtons();
}

function bindGoogleCastContext() {
  if (!hasGoogleCastApi()) {
    return;
  }

  const castContext = window.cast.framework.CastContext.getInstance();
  castContext.setOptions({
    receiverApplicationId: window.chrome.cast.media.DEFAULT_MEDIA_RECEIVER_APP_ID,
    autoJoinPolicy: window.chrome.cast.AutoJoinPolicy.ORIGIN_SCOPED,
  });

  detailState.castReady = true;

  if (!detailState.castListenersBound) {
    castContext.addEventListener(
      window.cast.framework.CastContextEventType.CAST_STATE_CHANGED,
      (event) => {
        const nextState = String(event?.castState || '');
        detailState.castDevicesAvailable = nextState !== String(window.cast.framework.CastState.NO_DEVICES_AVAILABLE || '');
        updatePlaybackButtons();
      }
    );

    castContext.addEventListener(
      window.cast.framework.CastContextEventType.SESSION_STATE_CHANGED,
      (event) => {
        const sessionState = String(event?.sessionState || '');
        detailState.castingActive = sessionState === String(window.cast.framework.SessionState.SESSION_STARTED || '')
          || sessionState === String(window.cast.framework.SessionState.SESSION_RESUMED || '');
        updatePlaybackButtons();
      }
    );

    detailState.castListenersBound = true;
  }

  const initialState = String(castContext.getCastState?.() || '');
  detailState.castDevicesAvailable = initialState !== String(window.cast.framework.CastState.NO_DEVICES_AVAILABLE || '');
  detailState.castingActive = Boolean(castContext.getCurrentSession?.());
  updatePlaybackButtons();
}

function installGoogleCastCallback() {
  if (hasGoogleCastApi()) {
    bindGoogleCastContext();
    updatePlaybackButtons();
    return;
  }

  const previousCallback = typeof window.__onGCastApiAvailable === 'function'
    ? window.__onGCastApiAvailable
    : null;

  window.__onGCastApiAvailable = (isAvailable) => {
    if (typeof previousCallback === 'function') {
      previousCallback(isAvailable);
    }
    if (isAvailable) {
      bindGoogleCastContext();
      updatePlaybackButtons();
    }
  };
}

function getCastTargetUrl() {
  const rawUrl = detailState.currentRawPlaybackUrl || detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || window.location.href;
  return getAbsoluteUrl(rawUrl);
}

async function openGoogleCast() {
  if (!hasGoogleCastApi()) {
    throw new Error('当前浏览器尚未就绪 Google Cast');
  }

  const castContext = window.cast.framework.CastContext.getInstance();
  bindGoogleCastContext();

  await Promise.resolve(castContext.requestSession());

  const session = castContext.getCurrentSession();
  if (!session) {
    throw new Error('未找到可用投屏会话');
  }

  const targetUrl = getCastTargetUrl();
  const mediaInfo = new window.chrome.cast.media.MediaInfo(
    targetUrl,
    isHlsUrl(targetUrl) ? 'application/x-mpegurl' : 'video/mp4'
  );
  const request = new window.chrome.cast.media.LoadRequest(mediaInfo);

  if (videoEl instanceof HTMLVideoElement && Number.isFinite(videoEl.currentTime) && videoEl.currentTime > 0) {
    request.currentTime = videoEl.currentTime;
  }

  await Promise.resolve(session.loadMedia(request));
  detailState.castingActive = true;
  updatePlaybackButtons();

  if (videoEl instanceof HTMLVideoElement) {
    videoEl.pause();
  }
}

function canUseNativeHls() {
  if (!(videoEl instanceof HTMLVideoElement) || typeof videoEl.canPlayType !== 'function') {
    return false;
  }

  const nativeSupport = String(videoEl.canPlayType('application/vnd.apple.mpegurl') || '').trim();
  if (!nativeSupport) {
    return false;
  }

  const userAgent = String(window.navigator?.userAgent || '').toLowerCase();
  const isApplePlatform = /iphone|ipad|ipod|macintosh/.test(userAgent);
  const isChromiumFamily = /chrome|chromium|crios|edg|opr|opera/.test(userAgent);
  const hasWebkitFullscreen = typeof videoEl.webkitEnterFullscreen === 'function';

  return hasWebkitFullscreen || (isApplePlatform && !isChromiumFamily);
}

function isAbortError(error) {
  return error instanceof DOMException
    ? error.name === 'AbortError'
    : String(error?.name || '') === 'AbortError';
}

function clearNativeHlsPrepareTimeout() {
  if (detailState.nativeHlsPrepareTimeoutId) {
    window.clearTimeout(detailState.nativeHlsPrepareTimeoutId);
    detailState.nativeHlsPrepareTimeoutId = 0;
  }
}

async function prepareNativeHlsPlaybackUrl(url, signal) {
  const createdBlobUrls = [];

  const makeAbsoluteUrl = (value, base) => {
    try {
      return new URL(String(value || ''), base).toString();
    } catch (_) {
      return String(value || '');
    }
  };

  const fetchPlaylistText = async (playlistUrl) => {
    const response = await fetch(playlistUrl, { credentials: 'same-origin', signal });
    if (!response.ok) {
      throw new Error(`native-hls-fetch-failed:${response.status}`);
    }
    return response.text();
  };

  const createPlaylistBlob = (content) => {
    const blobUrl = URL.createObjectURL(new Blob([content], { type: 'application/vnd.apple.mpegurl' }));
    createdBlobUrls.push(blobUrl);
    return blobUrl;
  };

  const rewritePlaylist = async (playlistText, playlistUrl) => {
    if (!playlistText.includes('#EXTM3U')) {
      return playlistText;
    }

    const lines = playlistText.split(/\r?\n/);
    const rewrittenLines = await Promise.all(lines.map(async (line, index) => {
      const trimmed = line.trim();
      if (!trimmed) {
        return line;
      }

      if (trimmed.startsWith('#EXT-X-MEDIA') && trimmed.includes('URI="')) {
        const match = trimmed.match(/URI="([^"]+)"/);
        const originalUri = match?.[1] || '';
        if (!originalUri) {
          return line;
        }
        const childUrl = makeAbsoluteUrl(originalUri, playlistUrl);
        try {
          const childText = await fetchPlaylistText(childUrl);
          const childBlobUrl = createPlaylistBlob(await rewritePlaylist(childText, childUrl));
          return line.replace(`URI="${originalUri}"`, `URI="${childBlobUrl}"`);
        } catch (_) {
          return line;
        }
      }

      const previousLine = index > 0 ? lines[index - 1].trim() : '';
      if (previousLine.startsWith('#EXT-X-STREAM-INF') && !trimmed.startsWith('#')) {
        const childUrl = makeAbsoluteUrl(trimmed, playlistUrl);
        try {
          const childText = await fetchPlaylistText(childUrl);
          return createPlaylistBlob(await rewritePlaylist(childText, childUrl));
        } catch (_) {
          return line;
        }
      }

      return line;
    }));

    return rewrittenLines.join('\n');
  };

  try {
    const playlistText = await fetchPlaylistText(url);
    const preparedContent = await rewritePlaylist(playlistText, getAbsoluteUrl(url));
    const preparedUrl = createPlaylistBlob(preparedContent);
    detailState.nativeHlsBlobUrls = createdBlobUrls;
    return preparedUrl;
  } catch (error) {
    createdBlobUrls.forEach((blobUrl) => {
      if (blobUrl.startsWith('blob:')) {
        try {
          URL.revokeObjectURL(blobUrl);
        } catch (_) {
        }
      }
    });
    throw error;
  }
}

function getOrientationController() {
  return typeof screen !== 'undefined' ? screen.orientation : null;
}

function getEffectiveFullscreenType() {
  if (isiOSDevice) {
    return 'window';
  }
  return settings.fullscreenType === 'window' ? 'window' : 'native';
}

function isLandscapeViewport() {
  if (typeof window === 'undefined') {
    return true;
  }
  return window.innerWidth >= window.innerHeight;
}

function syncWindowFullscreenLayout() {
  const shell = videoEl?.closest('.video-shell');
  if (!(shell instanceof HTMLElement)) {
    return;
  }

  shell.classList.toggle(
    'force-landscape',
    detailState.windowFullscreen && getEffectiveFullscreenType() === 'window' && isiOSDevice && !isLandscapeViewport()
  );
}

function getNativeFullscreenElement() {
  return document.fullscreenElement
    || document.webkitFullscreenElement
    || document.mozFullScreenElement
    || document.msFullscreenElement
    || null;
}

function isNativeFullscreenActive() {
  return Boolean(getNativeFullscreenElement());
}

async function lockLandscapeOrientation() {
  const orientation = getOrientationController();
  if (orientation?.lock) {
    await orientation.lock('landscape').catch(() => {});
  }
}

function unlockOrientation() {
  const orientation = getOrientationController();
  if (orientation?.unlock) {
    try {
      orientation.unlock();
    } catch (_) {
    }
  }
}

async function enterNativeFullscreen(shell) {
  if (!(shell instanceof HTMLElement)) {
    return;
  }

  if (typeof shell.requestFullscreen === 'function') {
    await shell.requestFullscreen().catch(() => {});
  } else if (typeof shell.webkitRequestFullscreen === 'function') {
    await Promise.resolve(shell.webkitRequestFullscreen()).catch(() => {});
  } else if (typeof shell.mozRequestFullScreen === 'function') {
    await Promise.resolve(shell.mozRequestFullScreen()).catch(() => {});
  } else if (typeof shell.msRequestFullscreen === 'function') {
    await Promise.resolve(shell.msRequestFullscreen()).catch(() => {});
  } else if (videoEl instanceof HTMLVideoElement && typeof videoEl.webkitEnterFullscreen === 'function') {
    videoEl.webkitEnterFullscreen();
  }

  await lockLandscapeOrientation();
}

async function exitNativeFullscreen() {
  if (typeof document.exitFullscreen === 'function') {
    await document.exitFullscreen().catch(() => {});
  } else if (typeof document.webkitExitFullscreen === 'function') {
    await Promise.resolve(document.webkitExitFullscreen()).catch(() => {});
  } else if (typeof document.mozCancelFullScreen === 'function') {
    await Promise.resolve(document.mozCancelFullScreen()).catch(() => {});
  } else if (typeof document.msExitFullscreen === 'function') {
    await Promise.resolve(document.msExitFullscreen()).catch(() => {});
  }

  unlockOrientation();
}

function getHlsConstructor() {
  return typeof window !== 'undefined' ? window.Hls : null;
}

function clearHlsBootstrapRetry(resetAttempts = true) {
  if (detailState.hlsBootstrapTimer) {
    window.clearTimeout(detailState.hlsBootstrapTimer);
    detailState.hlsBootstrapTimer = 0;
  }
  if (resetAttempts) {
    detailState.hlsBootstrapAttempts = 0;
  }
}

function scheduleHlsBootstrapRetry(rawUrl, playbackRequestToken) {
  if (!isHlsUrl(detailState.currentPlaybackUrl || rawUrl) || canUseNativeHls()) {
    clearHlsBootstrapRetry();
    return false;
  }

  clearHlsBootstrapRetry(false);
  detailState.hlsBootstrapAttempts += 1;

  if (detailState.hlsBootstrapAttempts > 20) {
    clearHlsBootstrapRetry();
    return false;
  }

  detailState.hlsBootstrapTimer = window.setTimeout(() => {
    detailState.hlsBootstrapTimer = 0;

    if (
      detailState.playbackRequestToken !== playbackRequestToken
      || detailState.currentRawPlaybackUrl !== rawUrl
      || detailState.hls
    ) {
      return;
    }

    const HlsConstructor = getHlsConstructor();
    const hlsSupported = Boolean(
      HlsConstructor
      && typeof HlsConstructor.isSupported === 'function'
      && HlsConstructor.isSupported()
    );

    if (hlsSupported) {
      setStatus('muted', 'HLS.js 已就绪，正在重新挂载视频流...');
      attachPlaybackSource(rawUrl);
      return;
    }

    scheduleHlsBootstrapRetry(rawUrl, playbackRequestToken);
  }, 250);

  return true;
}

function destroyHls() {
  clearHlsBootstrapRetry();
  clearNativeHlsPrepareTimeout();
  if (detailState.nativeHlsAbortController && typeof detailState.nativeHlsAbortController.abort === 'function') {
    try {
      detailState.nativeHlsAbortReason = 'replaced';
      detailState.nativeHlsAbortController.abort();
    } catch (_) {
    }
  }
  detailState.nativeHlsAbortController = null;
  detailState.nativeHlsAbortReason = '';
  if (detailState.hls && typeof detailState.hls.destroy === 'function') {
    detailState.hls.destroy();
  }
  detailState.hls = null;
  if (Array.isArray(detailState.nativeHlsBlobUrls)) {
    detailState.nativeHlsBlobUrls.forEach((url) => {
      if (typeof url === 'string' && url.startsWith('blob:')) {
        try {
          URL.revokeObjectURL(url);
        } catch (_) {
        }
      }
    });
  }
  detailState.nativeHlsBlobUrls = [];
}

function getShowIdentifier(title, source, videoId) {
  return `${source}:${videoId}:${String(title || '').toLowerCase().trim()}`;
}

function getCurrentHistoryEntry() {
  const historyList = Array.isArray(playerState.historyData) ? playerState.historyData : [];
  return historyList.find((item) =>
    item
    && String(item.videoId ?? '') === String(playerState.videoId ?? '')
    && String(item.source ?? '') === String(playerState.source ?? '')
    && Number(item.episodeIndex ?? -1) === Number(detailState.currentEpisode)
  ) || historyList.find((item) =>
    item
    && String(item.videoId ?? '') === String(playerState.videoId ?? '')
    && Number(item.episodeIndex ?? -1) === Number(detailState.currentEpisode)
  ) || null;
}

function refreshResumeState() {
  const historyEntry = getCurrentHistoryEntry();
  const playbackPosition = Number(historyEntry?.playbackPosition || 0);
  const hashResumePosition = getHashResumePosition();
  detailState.resumePosition = Math.max(
    Number.isFinite(playbackPosition) ? playbackPosition : 0,
    hashResumePosition
  );
  detailState.resumeApplied = false;

  if (!resumeButton) {
    return;
  }

  if (detailState.resumePosition > 1) {
    resumeButton.disabled = false;
    resumeButton.textContent = `恢复到 ${formatTime(detailState.resumePosition)}`;
  } else {
    resumeButton.disabled = true;
    resumeButton.textContent = '恢复进度';
  }
}

function applyPlaybackRate() {
  const rate = Number(detailState.playbackRate || 1);
  if (videoEl) {
    videoEl.playbackRate = Number.isFinite(rate) && rate > 0 ? rate : 1;
  }
  if (playbackRateSelectEl instanceof HTMLSelectElement) {
    playbackRateSelectEl.value = String(Number.isFinite(rate) && rate > 0 ? rate : 1);
  }
}

function maybeResumePlayback(force = false) {
  if (!videoEl || !detailState.resumePosition || (!force && detailState.resumeApplied)) {
    return;
  }
  if (!Number.isFinite(videoEl.duration) || videoEl.duration <= 0) {
    return;
  }

  const safePosition = Math.min(detailState.resumePosition, Math.max(videoEl.duration - 3, 0));
  if (safePosition <= 1) {
    return;
  }

  videoEl.currentTime = safePosition;
  detailState.resumeApplied = true;
  updateProgressIndicator();
  setStatus('success', `已恢复到 ${formatTime(safePosition)}`);
}

function getEpisodeStep() {
  return settings.episodeReverseOrder ? -1 : 1;
}

function isEditableTarget(target) {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tagName = target.tagName.toLowerCase();
  return tagName === 'input'
    || tagName === 'textarea'
    || tagName === 'select'
    || target.isContentEditable;
}

function updatePlaybackButtons() {
  if (prevButton instanceof HTMLButtonElement) {
    prevButton.disabled = !hasPreviousEpisode();
  }
  if (nextButton instanceof HTMLButtonElement) {
    nextButton.disabled = !hasNextEpisode();
  }
  if (pipButton instanceof HTMLButtonElement) {
    pipButton.disabled = !canUsePictureInPicture();
    pipButton.textContent = isPictureInPictureActive() ? '退出画中画' : '画中画';
  }
  if (fullscreenButton instanceof HTMLButtonElement) {
    const isFullscreen = detailState.windowFullscreen || isNativeFullscreenActive();
    const label = getEffectiveFullscreenType() === 'window'
      ? (isFullscreen ? '退出网页全屏' : '网页全屏')
      : (isFullscreen ? '退出全屏' : '进入全屏');
    fullscreenButton.textContent = label;
  }
  if (remotePlaybackButton instanceof HTMLButtonElement) {
    if (detailState.castingActive) {
      remotePlaybackButton.textContent = '正在投屏';
    } else if (canUseGoogleCast()) {
      remotePlaybackButton.textContent = 'Google Cast';
    } else if (canUseAirPlay()) {
      remotePlaybackButton.textContent = 'AirPlay';
    } else if (videoEl && videoEl.remote && typeof videoEl.remote.prompt === 'function') {
      remotePlaybackButton.textContent = '投屏';
    } else if (navigator.share) {
      remotePlaybackButton.textContent = '系统分享';
    } else {
      remotePlaybackButton.textContent = '分享链接';
    }
  }
  if (copyStreamButton instanceof HTMLButtonElement) {
    copyStreamButton.disabled = !detailState.currentPlaybackUrl && !(videoEl instanceof HTMLVideoElement && videoEl.currentSrc);
  }
  updateRetryDiagnostics();
}

function setShortcutHint(extraMessage = '') {
  if (!shortcutsEl) {
    return;
  }
  const base = '快捷键：空格播放/暂停，左右方向键快退/快进，F 全屏，P 画中画，M 静音，[ / ] 切集，C 复制播放地址。';
  shortcutsEl.textContent = extraMessage ? `${base}${extraMessage}` : base;
}

async function attemptPlayback(context = 'auto') {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return false;
  }

  try {
    await videoEl.play();
    return true;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error || '');
    const blockedByPolicy = error instanceof DOMException
      ? error.name === 'NotAllowedError' || error.name === 'AbortError'
      : /notallowed|abort/i.test(message);

    if (blockedByPolicy) {
      setStatus('muted', '浏览器阻止了自动播放，请手动点击播放');
      setShortcutHint(' 浏览器阻止了自动播放，请手动点击播放。');
      return false;
    }

    if (context === 'manual') {
      setStatus('error', '开始播放失败，请尝试重新加载或切换线路');
    }
    return false;
  }
}

function updateCodecWarning(hlsInstance) {
  if (!hlsInstance || !Array.isArray(hlsInstance.levels)) {
    return;
  }

  const hasHevc = hlsInstance.levels.some((level) => {
    const codec = String(level?.videoCodec || '').toLowerCase();
    return codec.includes('hev') || codec.includes('h265');
  });

  if (hasHevc) {
    setStatus('error', '检测到 HEVC/H.265 编码，当前浏览器可能不支持');
    setShortcutHint(' 检测到 HEVC/H.265 编码。');
  }
}

function syncVolume(volumeDelta) {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return;
  }
  const nextVolume = Math.min(1, Math.max(0, Number((videoEl.volume + volumeDelta).toFixed(2))));
  videoEl.volume = nextVolume;
  videoEl.muted = nextVolume === 0;
  setStatus('muted', `音量 ${Math.round(nextVolume * 100)}%`);
}

function renderSkipIndicator(direction, amount) {
  const indicatorEl = direction === 'forward' ? skipForwardIndicatorEl : skipBackwardIndicatorEl;
  if (!(indicatorEl instanceof HTMLElement)) {
    return;
  }

  indicatorEl.textContent = `${direction === 'forward' ? '+' : '-'}${amount}秒`;
  indicatorEl.classList.remove('hidden');
  window.requestAnimationFrame(() => {
    indicatorEl.classList.add('visible');
  });

  if (detailState.skipIndicatorTimer) {
    window.clearTimeout(detailState.skipIndicatorTimer);
  }
  detailState.skipIndicatorTimer = window.setTimeout(() => {
    [skipBackwardIndicatorEl, skipForwardIndicatorEl].forEach((element) => {
      if (element instanceof HTMLElement) {
        element.classList.remove('visible');
      }
    });
    window.setTimeout(() => {
      [skipBackwardIndicatorEl, skipForwardIndicatorEl].forEach((element) => {
        if (element instanceof HTMLElement) {
          element.classList.add('hidden');
        }
      });
      detailState.skipForwardAmount = 0;
      detailState.skipBackwardAmount = 0;
    }, 180);
  }, 420);
}

function seekBy(seconds) {
  if (!(videoEl instanceof HTMLVideoElement) || !Number.isFinite(videoEl.duration) || videoEl.duration <= 0) {
    return;
  }
  const nextTime = Math.min(videoEl.duration, Math.max(0, (videoEl.currentTime || 0) + seconds));
  videoEl.currentTime = nextTime;
  updateProgressIndicator();
  setStatus('muted', `${seconds > 0 ? '已快进' : '已快退'}到 ${formatTime(nextTime)}`);
}

function handleSkipGesture(direction) {
  const amount = direction === 'forward'
    ? (detailState.skipForwardAmount || 0) + 10
    : (detailState.skipBackwardAmount || 0) + 10;

  if (direction === 'forward') {
    detailState.skipForwardAmount = amount;
    detailState.skipBackwardAmount = 0;
    seekBy(10);
  } else {
    detailState.skipBackwardAmount = amount;
    detailState.skipForwardAmount = 0;
    seekBy(-10);
  }

  renderSkipIndicator(direction, amount);
}

function resolveTapSide(clientX) {
  if (!(videoEl instanceof HTMLVideoElement) || !Number.isFinite(clientX)) {
    return '';
  }
  const rect = videoEl.getBoundingClientRect();
  const offsetX = clientX - rect.left;
  return offsetX < rect.width / 2 ? 'left' : 'right';
}

function handleVideoDoubleTap(side) {
  if (side === 'left') {
    handleSkipGesture('backward');
    return;
  }
  if (side === 'right') {
    handleSkipGesture('forward');
  }
}

function maybeAutoSkipSegments() {
  if (!(videoEl instanceof HTMLVideoElement) || !Number.isFinite(videoEl.duration) || videoEl.duration <= 0) {
    return;
  }

  const introSeconds = getNonNegativeSettingNumber(settings.skipIntroSeconds);
  if (settings.autoSkipIntro && !detailState.introSkipped && introSeconds > 0) {
    if (videoEl.currentTime < introSeconds) {
      const nextTime = Math.min(introSeconds, Math.max(videoEl.duration - 1, 0));
      if (nextTime > videoEl.currentTime) {
        videoEl.currentTime = nextTime;
        updateProgressIndicator();
        setStatus('muted', `已自动跳过片头，定位到 ${formatTime(nextTime)}`);
      }
    }
    detailState.introSkipped = true;
  }

  const outroSeconds = getNonNegativeSettingNumber(settings.skipOutroSeconds);
  if (!settings.autoSkipOutro || detailState.outroHandled || outroSeconds <= 0) {
    return;
  }

  const remaining = videoEl.duration - videoEl.currentTime;
  if (!Number.isFinite(remaining) || remaining > outroSeconds) {
    return;
  }

  detailState.outroHandled = true;
  if (settings.autoNextEpisode && hasNextEpisode()) {
    setStatus('success', '已自动跳过片尾，正在切换下一集');
    loadEpisode(detailState.currentEpisode + getEpisodeStep());
    return;
  }

  const safeTime = Math.max(videoEl.duration - 0.2, 0);
  if (safeTime > videoEl.currentTime) {
    videoEl.currentTime = safeTime;
    updateProgressIndicator();
  }
  setStatus('muted', '已自动跳过片尾');
}

function togglePlayback() {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return;
  }
  if (videoEl.paused) {
    attemptPlayback('manual').catch(() => {});
  } else {
    videoEl.pause();
  }
}

async function toggleFullscreen() {
  const shell = videoEl?.closest('.video-shell');
  if (!(shell instanceof HTMLElement)) {
    return;
  }

  if (getEffectiveFullscreenType() === 'window') {
    detailState.windowFullscreen = !detailState.windowFullscreen;
    shell.classList.toggle('is-web-fullscreen', detailState.windowFullscreen);
    document.body.classList.toggle('player-web-fullscreen', detailState.windowFullscreen);
    syncWindowFullscreenLayout();

    if (detailState.windowFullscreen) {
      await lockLandscapeOrientation();
    } else {
      unlockOrientation();
    }
  } else if (isNativeFullscreenActive()) {
    await exitNativeFullscreen();
  } else {
    await enterNativeFullscreen(shell);
  }
  updatePlaybackButtons();
}

async function togglePictureInPicture() {
  if (!(videoEl instanceof HTMLVideoElement) || !canUsePictureInPicture()) {
    return;
  }

  if (document.pictureInPictureElement === videoEl) {
    await document.exitPictureInPicture().catch(() => {});
  } else if (videoEl.webkitPresentationMode === 'picture-in-picture' && videoEl.webkitSetPresentationMode) {
    videoEl.webkitSetPresentationMode('inline');
  } else if (document.pictureInPictureEnabled && !videoEl.disablePictureInPicture) {
    await videoEl.requestPictureInPicture().catch(() => {});
  } else if (videoEl.webkitSupportsPresentationMode?.('picture-in-picture') && videoEl.webkitSetPresentationMode) {
    videoEl.webkitSetPresentationMode('picture-in-picture');
  }
  updatePlaybackButtons();
}

async function openRemotePlayback() {
  if (canUseGoogleCast()) {
    await openGoogleCast();
    setStatus('success', '已推送到 Google Cast 设备');
    return;
  }

  if (canUseAirPlay()) {
    videoEl.webkitShowPlaybackTargetPicker();
    setStatus('success', '已打开 AirPlay 设备选择器');
    return;
  }

  if (videoEl && videoEl.remote && typeof videoEl.remote.prompt === 'function') {
    await videoEl.remote.prompt();
    setStatus('success', '已打开系统投屏面板');
    return;
  }

  if (navigator.share) {
    await navigator.share({
      title: currentDisplayTitle(),
      text: `在 RVideo 中打开 ${currentDisplayTitle()}`,
      url: window.location.href,
    });
    setStatus('success', '已打开系统分享面板');
    return;
  }

  await copyText(window.location.href);
  setStatus('success', '当前页面链接已复制');
}

async function copyCurrentPlaybackUrl() {
  const targetUrl = detailState.currentRawPlaybackUrl || detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || window.location.href;
  await copyText(targetUrl);
  setStatus('success', '当前播放地址已复制');
}

async function copyOriginalPlaybackUrl() {
  const targetUrl = detailState.currentRawPlaybackUrl || '';
  await copyText(targetUrl || window.location.href);
  setStatus('success', targetUrl ? '原始播放地址已复制' : '当前没有单独的原始播放地址，已复制页面链接');
}

async function copyProxyPlaybackUrl() {
  const rawUrl = detailState.currentRawPlaybackUrl || '';
  const targetUrl = rawUrl
    ? `/api/proxy?url=${encodeURIComponent(rawUrl)}&retry=${detailState.retryCount}`
    : detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || window.location.href;
  await copyText(targetUrl);
  setStatus('success', rawUrl ? '代理播放地址已复制' : '当前没有单独的代理播放地址，已复制当前链接');
}

async function copyActivePlaybackUrl() {
  const targetUrl = detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || detailState.currentRawPlaybackUrl || window.location.href;
  await copyText(targetUrl);
  setStatus('success', '当前生效播放链接已复制');
}

async function copyCurrentPageUrl() {
  await copyText(window.location.href);
  setStatus('success', '当前页面链接已复制');
}

const stallState = {
  lastCurrentTime: 0,
  lastMovementAt: 0,
  timer: 0,
  active: false,
};

function clearStallMonitor() {
  if (stallState.timer) {
    window.clearInterval(stallState.timer);
    stallState.timer = 0;
  }
  stallState.active = false;
  stallState.lastCurrentTime = 0;
  stallState.lastMovementAt = 0;
}

function clearPlaybackPolling() {
  if (detailState.playbackPollingTimer) {
    window.clearInterval(detailState.playbackPollingTimer);
    detailState.playbackPollingTimer = 0;
  }
}

function startPlaybackPolling() {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return;
  }

  clearPlaybackPolling();
  detailState.lastPolledTime = Number(videoEl.currentTime || 0);
  detailState.lastPolledPaused = videoEl.paused;

  detailState.playbackPollingTimer = window.setInterval(() => {
    if (!(videoEl instanceof HTMLVideoElement)) {
      return;
    }

    const currentTime = Number(videoEl.currentTime || 0);
    const isPaused = videoEl.paused;
    const duration = Number(videoEl.duration || 0);
    const timeChanged = Math.abs(currentTime - detailState.lastPolledTime) > 0.05;
    const pauseChanged = isPaused !== detailState.lastPolledPaused;

    if (timeChanged) {
      detailState.lastPolledTime = currentTime;
      updateProgressIndicator();
    }

    if (pauseChanged) {
      detailState.lastPolledPaused = isPaused;
      updatePlaybackButtons();
    }

    if (!isPaused && Number.isFinite(duration) && duration > 0) {
      const now = Date.now();
      if (now - detailState.lastSavedAt >= 5000) {
        detailState.lastSavedAt = now;
        saveHistory().catch(() => {});
      }
    }
  }, 500);
}

function startStallMonitor() {
  if (!(videoEl instanceof HTMLVideoElement)) {
    return;
  }

  clearStallMonitor();
  stallState.lastCurrentTime = Number(videoEl.currentTime || 0);
  stallState.lastMovementAt = Date.now();

  stallState.timer = window.setInterval(() => {
    if (!(videoEl instanceof HTMLVideoElement) || videoEl.paused || videoEl.seeking || videoEl.ended) {
      stallState.active = false;
      return;
    }

    const currentTime = Number(videoEl.currentTime || 0);
    const now = Date.now();

    if (currentTime !== stallState.lastCurrentTime) {
      stallState.lastCurrentTime = currentTime;
      stallState.lastMovementAt = now;
      if (stallState.active) {
        stallState.active = false;
        setStatus('success', `已恢复播放 ${formatTime(currentTime)}`);
      }
      return;
    }

    if (now - stallState.lastMovementAt > 1500 && !stallState.active) {
      stallState.active = true;
      setStatus('muted', '检测到播放卡顿，正在等待缓冲...');
    }
  }, 100);
}

function getFavorites() {
  return Array.isArray(playerState.favoritesData) ? playerState.favoritesData : [];
}

function getHistoryItems() {
  return Array.isArray(playerState.historyData) ? playerState.historyData : [];
}

function getSearchableSources() {
  const sourceKey = playerState.premium ? 'premiumSources' : 'sources';
  const sourceList = Array.isArray(settings[sourceKey]) ? settings[sourceKey] : [];
  return sourceList.filter((source) =>
    source
    && typeof source.id === 'string'
    && source.enabled !== false
  );
}

function isFavorite() {
  return getFavorites().some((item) =>
    item
    && String(item.videoId ?? '') === String(playerState.videoId ?? '')
    && String(item.source ?? '') === String(playerState.source ?? '')
  );
}

function renderFavoriteState() {
  const favorite = isFavorite();
  favoriteToggleEl.textContent = favorite ? '取消收藏' : '加入收藏';
  favoriteStatusEl.textContent = favorite ? '当前视频已在收藏中' : '当前视频尚未收藏';
  favoriteStatusEl.className = `status ${favorite ? 'success' : 'muted'}`;
}

async function persistFavorites() {
  await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: playerState.favoritesKey, value: playerState.favoritesData || [] }),
  });
}

async function persistHistory() {
  await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: playerState.historyKey, value: playerState.historyData || [] }),
  });
}

async function toggleFavorite() {
  if (!detailState.detail) {
    return;
  }

  const favorites = getFavorites();
  const index = favorites.findIndex((item) =>
    item
    && String(item.videoId ?? '') === String(playerState.videoId ?? '')
    && String(item.source ?? '') === String(playerState.source ?? '')
  );

  if (index >= 0) {
    favorites.splice(index, 1);
  } else {
    favorites.unshift({
      videoId: playerState.videoId,
      title: detailState.detail.vod_name || playerState.title || '未知视频',
      poster: detailState.detail.vod_pic || undefined,
      source: playerState.source,
      sourceName: playerState.source,
      addedAt: Date.now(),
      type: detailState.detail.type_name || undefined,
      year: detailState.detail.vod_year || undefined,
      remarks: detailState.detail.vod_remarks || undefined,
    });
  }

  playerState.favoritesData = favorites.slice(0, 100);
  await persistFavorites();
  renderFavoriteState();
  renderLibraryPanel();
}

async function saveHistory() {
  if (settings.watchHistory === false) {
    return;
  }

  if (!detailState.detail || !videoEl.duration || !Number.isFinite(videoEl.duration) || videoEl.currentTime <= 1) {
    return;
  }

  const historyList = Array.isArray(playerState.historyData) ? [...playerState.historyData] : [];
  const title = detailState.detail.vod_name || playerState.title || '未知视频';
  const showIdentifier = getShowIdentifier(title, playerState.source, playerState.videoId);
  const item = {
    videoId: playerState.videoId,
    title,
    url: detailState.episodes[detailState.currentEpisode]?.url || '',
    episodeIndex: detailState.currentEpisode,
    source: playerState.source,
    timestamp: Date.now(),
    playbackPosition: Math.floor(videoEl.currentTime),
    duration: Math.floor(videoEl.duration),
    poster: detailState.detail.vod_pic || undefined,
    episodes: detailState.episodes.map((episode, index) => ({
      name: episode.name || `第${index + 1}集`,
      url: episode.url,
      index,
    })),
    showIdentifier,
  };

  const existingIndex = historyList.findIndex((entry) => entry && entry.showIdentifier === showIdentifier);
  if (existingIndex >= 0) {
    historyList.splice(existingIndex, 1);
  }
  historyList.unshift(item);
  playerState.historyData = historyList.slice(0, 50);

  await persistHistory();
  renderLibraryPanel();
}

function handlePlaybackFailure(message) {
  if (!detailState.useProxy && settings.proxyMode === 'retry') {
    detailState.useProxy = true;
    detailState.retryCount += 1;
    updateModeBadge();
    loadEpisode(detailState.currentEpisode, false, true);
    return true;
  }

  if (attemptAutoSwitchSource(message)) {
    return true;
  }

  setStatus('error', message);
  return false;
}

function updateModeBadge() {
  if (!settings.showModeIndicator) {
    modeBadge.classList.add('hidden');
    return;
  }
  modeBadge.classList.remove('hidden');
  modeBadge.textContent = detailState.useProxy ? '代理模式' : '直连模式';
  modeBadge.className = `mode-badge ${detailState.useProxy ? 'proxy' : 'direct'}`;
}

function applyPlayerPreferenceChanges() {
  const previousAdFilterMode = settings.adFilterMode;

  settings.fullscreenType = playerPrefFullscreenTypeEl instanceof HTMLSelectElement && playerPrefFullscreenTypeEl.value === 'window'
    ? 'window'
    : 'native';
  settings.adFilterMode = playerPrefAdFilterModeEl instanceof HTMLSelectElement
    && ['keyword', 'heuristic', 'aggressive'].includes(playerPrefAdFilterModeEl.value)
    ? playerPrefAdFilterModeEl.value
    : 'off';
  settings.adFilter = settings.adFilterMode !== 'off';
  settings.showModeIndicator = playerPrefShowModeIndicatorEl instanceof HTMLInputElement
    ? playerPrefShowModeIndicatorEl.checked
    : Boolean(settings.showModeIndicator);
  settings.autoNextEpisode = playerPrefAutoNextEpisodeEl instanceof HTMLInputElement
    ? playerPrefAutoNextEpisodeEl.checked
    : settings.autoNextEpisode !== false;
  settings.autoSwitchSourceOnFailure = playerPrefAutoSwitchSourceOnFailureEl instanceof HTMLInputElement
    ? playerPrefAutoSwitchSourceOnFailureEl.checked
    : Boolean(settings.autoSwitchSourceOnFailure);
  settings.autoSkipIntro = playerPrefAutoSkipIntroEl instanceof HTMLInputElement
    ? playerPrefAutoSkipIntroEl.checked
    : Boolean(settings.autoSkipIntro);
  settings.skipIntroSeconds = playerPrefSkipIntroSecondsEl instanceof HTMLInputElement
    ? getNonNegativeSettingNumber(playerPrefSkipIntroSecondsEl.value)
    : getNonNegativeSettingNumber(settings.skipIntroSeconds);
  settings.autoSkipOutro = playerPrefAutoSkipOutroEl instanceof HTMLInputElement
    ? playerPrefAutoSkipOutroEl.checked
    : Boolean(settings.autoSkipOutro);
  settings.skipOutroSeconds = playerPrefSkipOutroSecondsEl instanceof HTMLInputElement
    ? getNonNegativeSettingNumber(playerPrefSkipOutroSecondsEl.value)
    : getNonNegativeSettingNumber(settings.skipOutroSeconds);

  persistPlayerSettings();
  syncPlayerPreferenceControls();
  updateModeBadge();
  updatePlaybackButtons();
  renderEpisodes();
  maybeAutoSkipSegments();

  if (previousAdFilterMode !== settings.adFilterMode && detailState.episodes[detailState.currentEpisode]) {
    setStatus('muted', '已更新广告过滤模式，正在重新加载当前视频');
    loadEpisode(detailState.currentEpisode, false, true);
    return;
  }

  setStatus('success', '已更新当前播放器偏好');
}

function renderMetadata(detail) {
  const poster = detail.vod_pic
    ? `<img class=\"detail-poster\" src=\"${escapeHtml(detail.vod_pic)}\" alt=\"${escapeHtml(detail.vod_name || playerState.title)}\" referrerpolicy=\"no-referrer\" />`
    : '<div class=\"detail-poster placeholder\">🎬</div>';

  metadataEl.className = 'detail-summary';
  metadataEl.innerHTML = `
    <div class=\"detail-summary-grid\">
      ${poster}
      <div class=\"stack\">
        <h2>${escapeHtml(detail.vod_name || playerState.title || '未知视频')}</h2>
        <div class=\"chip-list\">
          ${detail.type_name ? `<span class=\"chip\">${escapeHtml(detail.type_name)}</span>` : ''}
          ${detail.vod_year ? `<span class=\"chip\">${escapeHtml(detail.vod_year)}</span>` : ''}
          ${detail.vod_area ? `<span class=\"chip\">${escapeHtml(detail.vod_area)}</span>` : ''}
          <span class=\"chip\">${escapeHtml(detail.source_code || playerState.source || '未知来源')}</span>
        </div>
        <p class=\"muted\">${escapeHtml(detail.vod_content || '暂无简介')}</p>
      </div>
    </div>`;

  renderFavoriteState();
  loadAlternativeSources().catch((error) => {
    if (sourceListEl) {
      sourceListEl.className = 'saved-list empty-state';
      sourceListEl.textContent = error instanceof Error ? error.message : '加载来源失败';
    }
    if (sourceCountEl) {
      sourceCountEl.textContent = '来源 0';
    }
  }).finally(() => {
    if (refreshSourcesButton instanceof HTMLButtonElement) {
      refreshSourcesButton.disabled = false;
    }
  });
}

function buildLibraryItemUrl(item) {
  const videoId = item?.videoId ?? item?.vod_id;
  const source = item?.source;
  const title = item?.title || '未知视频';
  if (!videoId || !source) {
    return '/';
  }

  const params = new URLSearchParams({
    id: String(videoId),
    source: String(source),
    title: String(title),
  });

  if (detailState.libraryTab === 'history') {
    const episodeIndex = Number(item?.episodeIndex ?? 0);
    if (Number.isInteger(episodeIndex) && episodeIndex >= 0) {
      params.set('episode', String(episodeIndex));
    }
  }

  if (playerState.premium) {
    params.set('premium', '1');
  }

  return `/player?${params.toString()}`;
}

function normalizeTitleForSourceMatch(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/[\s\-_:：·•,，.。!！?？"'`~《》“”‘’()\[\]【】]/g, '')
    .trim();
}

function getSourcePreferenceKey(title = currentDisplayTitle()) {
  return normalizeTitleForSourceMatch(title);
}

function loadSourcePreferences() {
  if (typeof window === 'undefined') {
    return {};
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(SOURCE_PREFERENCES_STORAGE_KEY) || '{}');
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch (_) {
    return {};
  }
}

function persistSourcePreferences(preferences) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(SOURCE_PREFERENCES_STORAGE_KEY, JSON.stringify(preferences));
}

function getPreferredSourceForTitle(title = currentDisplayTitle()) {
  const key = getSourcePreferenceKey(title);
  if (!key) {
    return '';
  }
  const preferences = loadSourcePreferences();
  return String(preferences[key] || '').trim();
}

function setPreferredSourceForTitle(title, source) {
  const key = getSourcePreferenceKey(title);
  const sourceId = String(source || '').trim();
  if (!key || !sourceId) {
    return;
  }
  const preferences = loadSourcePreferences();
  preferences[key] = sourceId;
  const entries = Object.entries(preferences).slice(-200);
  persistSourcePreferences(Object.fromEntries(entries));
}

function clearPreferredSourceForTitle(title = currentDisplayTitle()) {
  const key = getSourcePreferenceKey(title);
  if (!key) {
    return false;
  }
  const preferences = loadSourcePreferences();
  if (!preferences[key]) {
    return false;
  }
  delete preferences[key];
  persistSourcePreferences(preferences);
  return true;
}

function updateSourcePreferenceButton() {
  if (!(playerPrefClearSourcePreferenceButton instanceof HTMLButtonElement)) {
    updateSourceDiagnostics();
    return;
  }
  const preferredSource = getPreferredSourceForTitle();
  playerPrefClearSourcePreferenceButton.disabled = !preferredSource;
  playerPrefClearSourcePreferenceButton.textContent = preferredSource
    ? `清除当前片名来源偏好（${preferredSource}）`
    : '清除当前片名来源偏好';
  updateSourceDiagnostics();
}

function buildSourceDiagnosticsText() {
  const currentSource = String(playerState.source || '').trim() || '未识别';
  const preferredSource = getPreferredSourceForTitle() || '未设置';
  const failedTrail = Array.isArray(detailState.failedSources) && detailState.failedSources.length
    ? detailState.failedSources.join('、')
    : '无';
  return `片名：${currentDisplayTitle()} | 当前来源：${currentSource} | 偏好来源：${preferredSource} | 失败轨迹：${failedTrail}`;
}

function updateSourceDiagnostics() {
  const diagnosticsText = buildSourceDiagnosticsText();
  if (sourceDiagnosticsEl instanceof HTMLElement) {
    sourceDiagnosticsEl.textContent = diagnosticsText;
  }
  if (playerPrefClearFailedTrailButton instanceof HTMLButtonElement) {
    const failedCount = Array.isArray(detailState.failedSources) ? detailState.failedSources.length : 0;
    playerPrefClearFailedTrailButton.disabled = failedCount === 0;
    playerPrefClearFailedTrailButton.textContent = failedCount > 0
      ? `清除失败来源轨迹 (${failedCount})`
      : '清除失败来源轨迹';
  }
  if (playerPrefCopySourceDiagnosticsButton instanceof HTMLButtonElement) {
    playerPrefCopySourceDiagnosticsButton.textContent = '复制来源诊断';
  }
}

async function copySourceDiagnostics() {
  const diagnosticsText = buildSourceDiagnosticsText();
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(diagnosticsText);
    return diagnosticsText;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

function buildRetryDiagnosticsText() {
  const configuredProxyMode = String(settings.proxyMode || 'retry');
  const activeMode = detailState.useProxy ? '代理模式' : '直连模式';
  const activeUrl = detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || '尚未开始播放';
  return `配置代理模式：${configuredProxyMode} | 当前播放模式：${activeMode} | 重试次数：${detailState.retryCount} | 当前播放地址：${activeUrl}`;
}

function updateRetryDiagnostics() {
  if (retryDiagnosticsEl instanceof HTMLElement) {
    retryDiagnosticsEl.textContent = buildRetryDiagnosticsText();
  }
  if (playerPrefForceDirectButton instanceof HTMLButtonElement) {
    playerPrefForceDirectButton.disabled = !detailState.useProxy;
  }
  if (playerPrefForceProxyButton instanceof HTMLButtonElement) {
    playerPrefForceProxyButton.disabled = detailState.useProxy;
  }
  if (playerPrefResetRetryStateButton instanceof HTMLButtonElement) {
    playerPrefResetRetryStateButton.disabled = detailState.retryCount === 0 && !detailState.failedSources.length;
  }
}

async function copyRetryDiagnostics() {
  const diagnosticsText = buildRetryDiagnosticsText();
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(diagnosticsText);
    return diagnosticsText;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

function buildPlaybackDiagnosticsPayload() {
  const currentTime = videoEl instanceof HTMLVideoElement ? Number(videoEl.currentTime || 0) : 0;
  const duration = videoEl instanceof HTMLVideoElement ? Number(videoEl.duration || 0) : 0;
  return {
    format: 'kvideo-playback-diagnostics',
    version: 1,
    capturedAt: new Date().toISOString(),
    title: currentDisplayTitle(),
    videoId: String(playerState.videoId || ''),
    source: String(playerState.source || ''),
    currentEpisode: detailState.currentEpisode,
    configuredProxyMode: String(settings.proxyMode || 'retry'),
    activeMode: detailState.useProxy ? 'proxy' : 'direct',
    retryCount: detailState.retryCount,
    preferredSource: getPreferredSourceForTitle() || null,
    failedSources: Array.isArray(detailState.failedSources) ? detailState.failedSources.slice() : [],
    currentPlaybackUrl: detailState.currentPlaybackUrl || (videoEl instanceof HTMLVideoElement ? videoEl.currentSrc : '') || '',
    currentRawPlaybackUrl: detailState.currentRawPlaybackUrl || '',
    currentTime,
    duration,
    isPremium: Boolean(playerState.premium),
    groupedSources: getNavigationGroupedSources(),
  };
}

async function copyPlaybackDiagnosticsJson() {
  const payload = buildPlaybackDiagnosticsPayload();
  const text = JSON.stringify(payload, null, 2);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return text;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

function exportPlaybackDiagnostics() {
  const payload = buildPlaybackDiagnosticsPayload();
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-playback-diagnostics-${new Date().toISOString().slice(0, 19).replaceAll(':', '-')}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function setForcedPlaybackMode(useProxy) {
  const nextUseProxy = Boolean(useProxy);
  if (detailState.useProxy === nextUseProxy) {
    updateRetryDiagnostics();
    return false;
  }
  detailState.useProxy = nextUseProxy;
  detailState.retryCount += 1;
  updateRetryDiagnostics();
  loadEpisode(detailState.currentEpisode, false, true);
  return true;
}

function resetRetryDiagnosticsState() {
  const hadChanges = detailState.retryCount > 0 || detailState.useProxy !== (settings.proxyMode === 'always') || detailState.failedSources.length > 0;
  detailState.retryCount = 0;
  detailState.useProxy = settings.proxyMode === 'always';
  clearFailedSourceTrailFromUrl();
  updateRetryDiagnostics();
  if (hadChanges) {
    loadEpisode(detailState.currentEpisode, false);
  }
  return hadChanges;
}

function sortSourceResultsWithPreference(results) {
  const currentSource = String(playerState.source || '').trim();
  const preferredSource = getPreferredSourceForTitle();
  return [...results].sort((left, right) => {
    const leftSource = String(left?.source || '').trim();
    const rightSource = String(right?.source || '').trim();
    if (leftSource === currentSource) {
      return -1;
    }
    if (rightSource === currentSource) {
      return 1;
    }
    if (preferredSource) {
      if (leftSource === preferredSource) {
        return -1;
      }
      if (rightSource === preferredSource) {
        return 1;
      }
    }
    return Number(right?.score || 0) - Number(left?.score || 0);
  });
}

function scoreSourceMatch(targetTitle, candidateTitle) {
  const normalizedTarget = normalizeTitleForSourceMatch(targetTitle);
  const normalizedCandidate = normalizeTitleForSourceMatch(candidateTitle);
  if (!normalizedTarget || !normalizedCandidate) {
    return 0;
  }
  if (normalizedTarget === normalizedCandidate) {
    return 3;
  }
  if (normalizedCandidate.includes(normalizedTarget) || normalizedTarget.includes(normalizedCandidate)) {
    return 2;
  }
  if (candidateTitle.includes(targetTitle) || targetTitle.includes(candidateTitle)) {
    return 1;
  }
  return 0;
}

function buildSourceSwitchUrl(item, options = {}) {
  const params = new URLSearchParams({
    id: String(item?.vod_id ?? item?.videoId ?? ''),
    source: String(item?.source ?? ''),
    title: String(item?.vod_name || item?.title || currentDisplayTitle()),
    episode: String(detailState.currentEpisode || 0),
  });
  const groupedSources = getNavigationGroupedSources();
  if (groupedSources.length > 1) {
    params.set('groupedSources', JSON.stringify(groupedSources));
  }
  if (playerState.premium) {
    params.set('premium', '1');
  }
  if (Array.isArray(options.failedSources)) {
    options.failedSources
      .map((item) => String(item || '').trim())
      .filter(Boolean)
      .forEach((item) => {
        params.append('failedSource', item);
      });
  }
  return `/player?${params.toString()}`;
}

function getAutoSwitchCandidates() {
  const currentSource = String(playerState.source || '').trim();
  const failedSources = new Set(
    detailState.failedSources
      .map((item) => String(item || '').trim())
      .filter(Boolean)
  );
  failedSources.add(currentSource);
  return getNavigationGroupedSources().filter((item) => {
    const sourceId = String(item?.source || '').trim();
    return Boolean(sourceId) && !failedSources.has(sourceId);
  });
}

function attemptAutoSwitchSource(message) {
  if (!settings.autoSwitchSourceOnFailure) {
    return false;
  }
  const nextSource = getAutoSwitchCandidates()[0];
  if (!nextSource) {
    return false;
  }
  const nextFailedSources = Array.from(new Set([
    ...detailState.failedSources,
    String(playerState.source || '').trim(),
  ].filter(Boolean)));
  const nextLabel = String(
    nextSource?.sourceDisplayName || nextSource?.sourceName || nextSource?.source || '下一来源'
  ).trim();
  setStatus('muted', `${message}，正在自动切换到 ${nextLabel}`);
  window.location.replace(buildSourceSwitchUrl(nextSource, { failedSources: nextFailedSources }));
  return true;
}

function renderSourceResults() {
  if (!sourceListEl || !sourceCountEl) {
    return;
  }

  updateSourcePreferenceButton();
  updateSourceDiagnostics();
  const results = Array.isArray(detailState.sourceResults) ? detailState.sourceResults : [];
  const preferredSource = getPreferredSourceForTitle();
  sourceCountEl.textContent = `来源 ${results.length}`;

  if (!results.length) {
    sourceListEl.className = 'saved-list empty-state';
    sourceListEl.textContent = '当前没有发现其他可切换来源';
    return;
  }

  sourceListEl.className = 'saved-list';
  sourceListEl.innerHTML = results.map((item) => {
    const isCurrent = String(item?.source ?? '') === String(playerState.source ?? '');
    const isPreferred = String(item?.source ?? '') === preferredSource;
    const sourceLabel = escapeHtml(item?.sourceDisplayName || item?.sourceName || item?.source || '未知来源');
    const remarks = escapeHtml(item?.vod_remarks || item?.type_name || '点击后切换来源');
    const href = escapeHtml(buildSourceSwitchUrl(item));
    return `
      <article class="saved-item library-item${isCurrent ? ' active-saved-item' : ''}">
        <div class="row space-between wrap gap-sm">
          <a class="library-item-main" href="${href}">
            <strong>${sourceLabel}</strong>
            <span class="muted">${escapeHtml(item?.vod_name || currentDisplayTitle())}</span>
            <span class="muted tiny">${remarks}</span>
          </a>
          <span class="chip">${isCurrent && isPreferred ? '当前 / 偏好' : isCurrent ? '当前来源' : isPreferred ? '偏好来源' : '可切换'}</span>
        </div>
      </article>`;
  }).join('');
}

function mergeInitialGroupedSources() {
  const groupedSources = getGroupedSources();
  if (!groupedSources.length) {
    return;
  }

  const resultMap = new Map(
    (Array.isArray(detailState.sourceResults) ? detailState.sourceResults : []).map((item) => [String(item?.source || ''), item])
  );
  groupedSources.forEach((item) => {
    const sourceId = String(item?.source || '').trim();
    if (!sourceId || resultMap.has(sourceId)) {
      return;
    }
    resultMap.set(sourceId, {
      vod_id: item?.id ?? playerState.videoId,
      vod_name: playerState.title,
      source: sourceId,
      sourceName: item?.sourceName || sourceId,
      sourceDisplayName: item?.sourceName || sourceId,
      latency: item?.latency,
      vod_pic: item?.pic,
      score: sourceId === String(playerState.source || '') ? 4 : 3,
      vod_remarks: sourceId === String(playerState.source || '') ? '当前播放来源' : '分组来源',
    });
  });
  detailState.sourceResults = sortSourceResultsWithPreference(Array.from(resultMap.values()));
}

async function loadAlternativeSources() {
  if (!sourceListEl) {
    return;
  }

  const query = currentDisplayTitle().trim();
  const sources = getSearchableSources();

  if (!query) {
    detailState.sourceResults = [];
    renderSourceResults();
    return;
  }

  if (!sources.length) {
    detailState.sourceResults = [];
    sourceListEl.className = 'saved-list empty-state';
    sourceListEl.textContent = '当前没有已启用线路可用于来源切换';
    if (sourceCountEl) {
      sourceCountEl.textContent = '来源 0';
    }
    return;
  }

  sourceListEl.className = 'saved-list empty-state';
  sourceListEl.textContent = `正在搜索「${query}」的其他来源...`;
  if (refreshSourcesButton instanceof HTMLButtonElement) {
    refreshSourcesButton.disabled = true;
  }

  const response = await fetch('/api/search-parallel', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, sources, page: 1 }),
  });

  if (!response.ok || !response.body) {
    const data = await response.json().catch(() => ({}));
    throw new Error(data.error || '搜索来源失败');
  }

  const decoder = new TextDecoder();
  const reader = response.body.getReader();
  let buffer = '';
  const allVideos = [];

  while (true) {
    const { value, done } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (!line.startsWith('data: ')) {
        continue;
      }

      try {
        const payload = JSON.parse(line.slice(6));
        if (payload.type === 'videos' && Array.isArray(payload.videos)) {
          allVideos.push(...payload.videos);
        }
      } catch (_) {
      }
    }
  }

  const resultMap = new Map();
  const currentTitle = currentDisplayTitle();
  for (const video of allVideos) {
    const sourceId = String(video?.source || '');
    if (!sourceId) {
      continue;
    }
    const score = scoreSourceMatch(currentTitle, String(video?.vod_name || ''));
    if (!score && sourceId !== String(playerState.source || '')) {
      continue;
    }

    const existing = resultMap.get(sourceId);
    if (!existing || score > existing.score) {
      resultMap.set(sourceId, { ...video, score });
    }
  }

  if (!resultMap.has(String(playerState.source || ''))) {
    resultMap.set(String(playerState.source || ''), {
      vod_id: playerState.videoId,
      vod_name: currentTitle,
      source: playerState.source,
      sourceName: playerState.source,
      score: 4,
      vod_remarks: '当前播放来源',
    });
  }

  detailState.sourceResults = sortSourceResultsWithPreference(Array.from(resultMap.values()));
  mergeInitialGroupedSources();
  renderSourceResults();
}

function isCurrentLibraryItem(item) {
  return String(item?.videoId ?? item?.vod_id ?? '') === String(playerState.videoId ?? '')
    && String(item?.source ?? '') === String(playerState.source ?? '');
}

function getCurrentLibraryItems() {
  return detailState.libraryTab === 'favorites' ? getFavorites() : getHistoryItems();
}

function updateLibraryTabs() {
  ['history', 'favorites'].forEach((tab) => {
    const button = getLibraryTabButton(tab);
    if (button instanceof HTMLButtonElement) {
      button.classList.toggle('active', tab === detailState.libraryTab);
    }
  });
}

function renderLibraryPanel() {
  if (!libraryEl || !libraryCountEl) {
    return;
  }

  updateLibraryTabs();

  const tabLabel = detailState.libraryTab === 'favorites' ? '收藏' : '历史';
  const items = getCurrentLibraryItems();
  libraryCountEl.textContent = `${tabLabel} ${items.length}`;

  if (!items.length) {
    libraryEl.className = 'saved-list empty-state';
    libraryEl.textContent = detailState.libraryTab === 'favorites'
      ? '当前还没有收藏内容'
      : '当前还没有同步历史记录';
    return;
  }

  libraryEl.className = 'saved-list';
  libraryEl.innerHTML = items.slice(0, 12).map((item, index) => {
    const title = escapeHtml(item?.title || '未知视频');
    const source = escapeHtml(item?.sourceName || item?.source || '未知来源');
    const href = escapeHtml(buildLibraryItemUrl(item));
    const currentClass = isCurrentLibraryItem(item) ? ' active-saved-item' : '';
    const subtitle = detailState.libraryTab === 'favorites'
      ? `${escapeHtml(item?.type || item?.remarks || '收藏内容')}${item?.year ? ` · ${escapeHtml(item.year)}` : ''}`
      : `${escapeHtml(item?.episodes?.[item?.episodeIndex]?.name || `第${Number(item?.episodeIndex ?? 0) + 1}集`)} · ${formatTime(Number(item?.playbackPosition || 0))}/${formatTime(Number(item?.duration || 0))}`;

    return `
      <article class="saved-item library-item${currentClass}">
        <div class="row space-between wrap gap-sm">
          <a class="library-item-main" href="${href}">
            <strong>${title}</strong>
            <span class="muted">${source}</span>
            <span class="muted tiny">${subtitle}</span>
          </a>
          <button class="button danger button-small" type="button" data-library-remove="${index}">移除</button>
        </div>
      </article>`;
  }).join('');
}

async function clearCurrentLibrary() {
  if (detailState.libraryTab === 'favorites') {
    playerState.favoritesData = [];
    await persistFavorites();
  } else {
    playerState.historyData = [];
    await persistHistory();
  }
  renderLibraryPanel();
}

async function removeLibraryItem(index) {
  if (!Number.isInteger(index) || index < 0) {
    return;
  }

  if (detailState.libraryTab === 'favorites') {
    const favorites = getFavorites().slice();
    if (!favorites[index]) {
      return;
    }
    favorites.splice(index, 1);
    playerState.favoritesData = favorites;
    await persistFavorites();
    renderFavoriteState();
  } else {
    const historyItems = getHistoryItems().slice();
    if (!historyItems[index]) {
      return;
    }
    historyItems.splice(index, 1);
    playerState.historyData = historyItems;
    await persistHistory();
    refreshResumeState();
  }

  renderLibraryPanel();
}

function renderEpisodes() {
  episodeCountEl.textContent = `选集 ${detailState.episodes.length}`;
  if (!detailState.episodes.length) {
    episodesEl.className = 'episodes-grid empty-state';
    episodesEl.textContent = '当前没有可播放选集';
    return;
  }

  const episodeEntries = detailState.episodes.map((episode, index) => ({ episode, index }));
  if (settings.episodeReverseOrder) {
    episodeEntries.reverse();
  }

  episodesEl.className = 'episodes-grid';
  episodesEl.innerHTML = episodeEntries.map(({ episode, index }) => `
    <button class=\"episode-item ${index === detailState.currentEpisode ? 'active' : ''}\" type=\"button\" data-episode-index=\"${index}\">
      <strong>${escapeHtml(episode.name || `第${index + 1}集`)}</strong>
      <span class=\"muted\">${index === detailState.currentEpisode ? '当前播放' : '切换播放'}</span>
    </button>`).join('');
  updatePlaybackButtons();
}

function attachPlaybackSource(rawUrl) {
  destroyHls();
  detailState.resumeApplied = false;
  detailState.playbackRequestToken += 1;
  const playbackRequestToken = detailState.playbackRequestToken;
  const resolvedUrl = getResolvedPlayUrl(rawUrl);
  const shouldFilterAds = shouldFilterAdsForUrl(rawUrl);
  detailState.currentRawPlaybackUrl = rawUrl || '';
  detailState.currentPlaybackUrl = resolvedUrl;
  const HlsConstructor = getHlsConstructor();
  const hlsSupported = Boolean(
    HlsConstructor
    && typeof HlsConstructor.isSupported === 'function'
    && HlsConstructor.isSupported()
  );
  const shouldUseHlsJs = Boolean(
    videoEl
    && isHlsUrl(resolvedUrl)
    && hlsSupported
    && (!canUseNativeHls() || shouldFilterAds)
  );
  const nativeHlsSupported = canUseNativeHls();

  clearHlsBootstrapRetry();

  if (videoEl) {
    videoEl.pause();
    videoEl.removeAttribute('src');
    videoEl.load();
  }

  if (!resolvedUrl) {
    updatePlaybackButtons();
    if (!handlePlaybackFailure('当前选集没有可用播放地址')) {
      setStatus('error', '当前选集没有可用播放地址');
    }
    return;
  }

  if (isHlsUrl(resolvedUrl) && !shouldUseHlsJs && !nativeHlsSupported) {
    if (scheduleHlsBootstrapRetry(rawUrl, playbackRequestToken)) {
      updatePlaybackButtons();
      setStatus('muted', `正在等待 HLS.js 加载 (${detailState.hlsBootstrapAttempts}/20)...`);
      return;
    }
    updatePlaybackButtons();
    if (!handlePlaybackFailure('当前浏览器不支持 HLS 视频播放，请尝试切换线路或使用支持 HLS 的浏览器')) {
      setStatus('error', '当前浏览器不支持 HLS 视频播放，请尝试切换线路或使用支持 HLS 的浏览器');
      setShortcutHint(' 当前浏览器不支持 HLS 视频播放，请尝试切换线路。');
    }
    return;
  }

  if (shouldUseHlsJs) {
    clearHlsBootstrapRetry();
    const hls = new HlsConstructor({
      enableWorker: true,
      lowLatencyMode: false,
      backBufferLength: 30,
      maxBufferLength: 60,
      maxMaxBufferLength: 120,
      maxBufferSize: 60 * 1000 * 1000,
      maxBufferHole: 0.5,
      startFragPrefetch: true,
      abrEwmaDefaultEstimate: 500000,
      abrEwmaFastLive: 3,
      abrEwmaSlowLive: 9,
      abrEwmaFastVoD: 3,
      abrEwmaSlowVoD: 9,
      abrBandWidthFactor: 0.8,
      abrBandWidthUpFactor: 0.7,
      fragLoadingMaxRetry: 6,
      fragLoadingRetryDelay: 1000,
      fragLoadingMaxRetryTimeout: 64000,
      manifestLoadingMaxRetry: 4,
      manifestLoadingRetryDelay: 1000,
      manifestLoadingMaxRetryTimeout: 64000,
      levelLoadingMaxRetry: 4,
      levelLoadingRetryDelay: 1000,
      levelLoadingMaxRetryTimeout: 64000,
      fragLoadingTimeOut: 20000,
      manifestLoadingTimeOut: 10000,
      levelLoadingTimeOut: 10000,
    });
    let networkErrorRetries = 0;
    let mediaErrorRetries = 0;
    const maxRetries = 3;

    detailState.hls = hls;
    hls.loadSource(resolvedUrl);
    hls.attachMedia(videoEl);

    hls.on(HlsConstructor.Events.FRAG_LOADED, (_, data) => {
      if (videoEl.paused && data?.frag?.start === 0) {
        attemptPlayback('auto').catch(() => {});
      }
    });

    hls.on(HlsConstructor.Events.MANIFEST_PARSED, () => {
      updateCodecWarning(hls);
      applyPlaybackRate();
      maybeResumePlayback();
      attemptPlayback('auto').catch(() => {});
    });

    hls.on(HlsConstructor.Events.ERROR, (_, data) => {
      if (!data?.fatal) {
        return;
      }

      if (data.type === HlsConstructor.ErrorTypes.NETWORK_ERROR) {
        networkErrorRetries += 1;
        if (networkErrorRetries <= maxRetries && detailState.hls && typeof detailState.hls.startLoad === 'function') {
          setStatus('muted', `HLS 网络错误，正在重试 ${networkErrorRetries}/${maxRetries}...`);
          detailState.hls.startLoad();
          return;
        }

        const recovered = handlePlaybackFailure('网络错误：无法加载视频流');
        if (!recovered && detailState.hls && typeof detailState.hls.startLoad === 'function') {
          detailState.hls.startLoad();
        }
        return;
      }

      if (data.type === HlsConstructor.ErrorTypes.MEDIA_ERROR) {
        mediaErrorRetries += 1;
        if (mediaErrorRetries <= maxRetries && detailState.hls && typeof detailState.hls.recoverMediaError === 'function') {
          setStatus('muted', `媒体错误，正在恢复 ${mediaErrorRetries}/${maxRetries}...`);
          detailState.hls.recoverMediaError();
          return;
        }

        handlePlaybackFailure('媒体错误：视频格式不支持或已损坏');
        return;
      }

      handlePlaybackFailure(data.details || 'HLS 播放失败');
    });

    setStatus('muted', '正在通过 HLS.js 加载视频流...');
    updatePlaybackButtons();
    return;
  }

  if (videoEl) {
    clearHlsBootstrapRetry();
    if (shouldFilterAds && isHlsUrl(resolvedUrl) && nativeHlsSupported) {
      detailState.nativeHlsAbortController = typeof AbortController !== 'undefined'
        ? new AbortController()
        : null;
      detailState.nativeHlsAbortReason = '';
      clearNativeHlsPrepareTimeout();
      if (detailState.nativeHlsAbortController) {
        detailState.nativeHlsPrepareTimeoutId = window.setTimeout(() => {
          if (
            detailState.nativeHlsAbortController
            && typeof detailState.nativeHlsAbortController.abort === 'function'
          ) {
            detailState.nativeHlsAbortReason = 'timeout';
            detailState.nativeHlsAbortController.abort();
          }
        }, 8000);
      }
      setStatus('muted', '正在准备原生 HLS 广告过滤回放...');
      setShortcutHint(' 当前使用原生 HLS 广告过滤兜底。');
      prepareNativeHlsPlaybackUrl(
        resolvedUrl,
        detailState.nativeHlsAbortController ? detailState.nativeHlsAbortController.signal : undefined
      ).then((preparedUrl) => {
        clearNativeHlsPrepareTimeout();
        if (
          !(videoEl instanceof HTMLVideoElement)
          || detailState.currentPlaybackUrl !== resolvedUrl
          || detailState.playbackRequestToken !== playbackRequestToken
        ) {
          return;
        }
        videoEl.src = preparedUrl;
        applyPlaybackRate();
        videoEl.load();
        detailState.nativeHlsAbortReason = '';
        setStatus('muted', '正在通过原生 HLS 加载已过滤视频流...');
        attemptPlayback('auto').catch(() => {});
      }).catch((error) => {
        clearNativeHlsPrepareTimeout();
        if (isAbortError(error)) {
          if (detailState.nativeHlsAbortReason !== 'timeout') {
            return;
          }
        }
        if (
          !(videoEl instanceof HTMLVideoElement)
          || detailState.currentPlaybackUrl !== resolvedUrl
          || detailState.playbackRequestToken !== playbackRequestToken
        ) {
          return;
        }
        videoEl.src = resolvedUrl;
        applyPlaybackRate();
        videoEl.load();
        if (detailState.nativeHlsAbortReason === 'timeout') {
          setStatus('muted', '原生 HLS 广告过滤准备超时，已回退到当前过滤流');
          setShortcutHint(' 原生 HLS 广告过滤准备超时，已回退。');
        } else {
          setStatus('muted', '原生 HLS 广告过滤初始化失败，已回退到当前过滤流');
          setShortcutHint(' 原生 HLS 广告过滤初始化失败，已回退。');
        }
        detailState.nativeHlsAbortReason = '';
        attemptPlayback('auto').catch(() => {});
      });
      updatePlaybackButtons();
      return;
    }
    videoEl.src = resolvedUrl;
    applyPlaybackRate();
    videoEl.load();
    attemptPlayback('auto').catch(() => {});
  }
  updatePlaybackButtons();
}

installGoogleCastCallback();
detailState.airplayAvailable = hasAirPlayApi();

function loadEpisode(index, syncUrl = true, preserveRetryCount = false) {
  if (!detailState.episodes[index]) {
    return;
  }
  detailState.currentEpisode = index;
  if (!preserveRetryCount) {
    detailState.retryCount = 0;
  }
  detailState.introSkipped = false;
  detailState.outroHandled = false;
  const episode = detailState.episodes[index];
  refreshResumeState();
  attachPlaybackSource(episode.url);
  renderEpisodes();
  updateModeBadge();
  updatePlaybackButtons();
  setStatus('success', `正在播放 ${episode.name || `第${index + 1}集`}`);

  if (syncUrl) {
    window.history.replaceState({}, '', buildRustPlayerUrl(index));
  }
}

async function loadDetail() {
  setStatus('muted', '正在加载视频详情...');
  const response = playerState.sourceConfig
    ? await fetch('/api/detail', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: playerState.videoId, source: playerState.sourceConfig }),
      })
    : await fetch(`/api/detail?id=${encodeURIComponent(playerState.videoId)}&source=${encodeURIComponent(playerState.source)}`);
  const payload = await response.json().catch(() => ({}));

  if (!response.ok || !payload.success || !payload.data) {
    metadataEl.className = 'empty-state';
    metadataEl.textContent = payload.error || '加载详情失败';
    episodesEl.className = 'episodes-grid empty-state';
    episodesEl.textContent = '无法加载选集';
    setStatus('error', payload.error || '加载详情失败');
    return;
  }

  detailState.detail = payload.data;
  detailState.episodes = Array.isArray(payload.data.episodes) ? payload.data.episodes : [];
  if (!detailState.episodes.length) {
    metadataEl.className = 'empty-state';
    metadataEl.textContent = '该来源没有可播放剧集';
    episodesEl.className = 'episodes-grid empty-state';
    episodesEl.textContent = '当前没有可播放选集';
    setStatus('error', '该来源没有可播放剧集');
    return;
  }

  if (detailState.currentEpisode < 0 || detailState.currentEpisode >= detailState.episodes.length) {
    detailState.currentEpisode = settings.episodeReverseOrder ? detailState.episodes.length - 1 : 0;
  }

  renderMetadata(payload.data);
  renderEpisodes();
  refreshResumeState();
  applyPlaybackRate();
  updateProgressIndicator();
  loadEpisode(detailState.currentEpisode, false);
}

episodesEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-episode-index]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const index = Number(button.dataset.episodeIndex);
  if (Number.isInteger(index)) {
    loadEpisode(index);
  }
});

proxyButton?.addEventListener('click', () => {
  detailState.useProxy = !detailState.useProxy;
  detailState.retryCount += 1;
  loadEpisode(detailState.currentEpisode, false, true);
});

reloadButton?.addEventListener('click', () => {
  detailState.retryCount += 1;
  loadEpisode(detailState.currentEpisode, false, true);
});

resumeButton?.addEventListener('click', () => {
  maybeResumePlayback(true);
});

playbackRateSelectEl?.addEventListener('change', () => {
  if (!(playbackRateSelectEl instanceof HTMLSelectElement)) {
    return;
  }
  const nextRate = Number(playbackRateSelectEl.value || '1');
  detailState.playbackRate = Number.isFinite(nextRate) && nextRate > 0 ? nextRate : 1;
  if (typeof window !== 'undefined') {
    window.localStorage.setItem('kvideo-playback-rate', String(detailState.playbackRate));
  }
  applyPlaybackRate();
});

playerPreferencesButton?.addEventListener('click', () => {
  syncPlayerPreferenceControls();
  const nextOpen = playerPreferencesPanelEl instanceof HTMLElement
    ? playerPreferencesPanelEl.classList.contains('hidden')
    : false;
  setPlayerPreferencesPanel(nextOpen);
});

closePlayerPreferencesButton?.addEventListener('click', () => {
  setPlayerPreferencesPanel(false);
});

[
  playerPrefFullscreenTypeEl,
  playerPrefAdFilterModeEl,
  playerPrefShowModeIndicatorEl,
  playerPrefAutoNextEpisodeEl,
  playerPrefAutoSwitchSourceOnFailureEl,
  playerPrefAutoSkipIntroEl,
  playerPrefSkipIntroSecondsEl,
  playerPrefAutoSkipOutroEl,
  playerPrefSkipOutroSecondsEl,
].forEach((element) => {
  element?.addEventListener('change', applyPlayerPreferenceChanges);
});

playerPrefCopyPageLinkButton?.addEventListener('click', () => {
  copyCurrentPageUrl().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制页面链接失败');
  });
});

playerPrefCopyOriginalLinkButton?.addEventListener('click', () => {
  copyOriginalPlaybackUrl().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制原始播放链接失败');
  });
});

playerPrefCopyProxyLinkButton?.addEventListener('click', () => {
  copyProxyPlaybackUrl().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制代理播放链接失败');
  });
});

playerPrefClearSourcePreferenceButton?.addEventListener('click', () => {
  if (!clearPreferredSourceForTitle()) {
    setStatus('muted', '当前片名还没有已记忆的来源偏好');
    updateSourcePreferenceButton();
    return;
  }
  updateSourcePreferenceButton();
  renderSourceResults();
  setStatus('success', '已清除当前片名的来源偏好');
});

playerPrefClearFailedTrailButton?.addEventListener('click', () => {
  if (!detailState.failedSources.length) {
    updateSourceDiagnostics();
    setStatus('muted', '当前还没有失败来源轨迹');
    return;
  }
  clearFailedSourceTrailFromUrl();
  setStatus('success', '已清除失败来源轨迹');
});

playerPrefCopySourceDiagnosticsButton?.addEventListener('click', () => {
  copySourceDiagnostics().then(() => {
    updateSourceDiagnostics();
    setStatus('success', '已复制来源诊断信息');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制来源诊断失败');
  });
});

playerPrefForceDirectButton?.addEventListener('click', () => {
  if (!setForcedPlaybackMode(false)) {
    setStatus('muted', '当前已经是直连模式');
    return;
  }
  setStatus('success', '已强制切换到直连模式并重新加载');
});

playerPrefForceProxyButton?.addEventListener('click', () => {
  if (!setForcedPlaybackMode(true)) {
    setStatus('muted', '当前已经是代理模式');
    return;
  }
  setStatus('success', '已强制切换到代理模式并重新加载');
});

playerPrefResetRetryStateButton?.addEventListener('click', () => {
  if (!resetRetryDiagnosticsState()) {
    setStatus('muted', '当前没有可重置的重试状态');
    return;
  }
  setStatus('success', '已重置重试状态并恢复默认代理策略');
});

playerPrefCopyRetryDiagnosticsButton?.addEventListener('click', () => {
  copyRetryDiagnostics().then(() => {
    updateRetryDiagnostics();
    setStatus('success', '已复制重试诊断信息');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制重试诊断失败');
  });
});

playerPrefCopyPlaybackDiagnosticsJsonButton?.addEventListener('click', () => {
  copyPlaybackDiagnosticsJson().then(() => {
    setStatus('success', '已复制播放诊断 JSON');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制播放诊断 JSON 失败');
  });
});

playerPrefExportPlaybackDiagnosticsButton?.addEventListener('click', () => {
  try {
    exportPlaybackDiagnostics();
    setStatus('success', '已导出播放诊断文件');
  } catch (error) {
    setStatus('error', error instanceof Error ? error.message : '导出播放诊断失败');
  }
});

playerPrefSaveBookmarkButton?.addEventListener('click', () => {
  try {
    const bookmark = saveCurrentPlaybackBookmark();
    setStatus('success', `已保存播放书签「${bookmark.label}」`);
  } catch (error) {
    setStatus('error', error instanceof Error ? error.message : '保存播放书签失败');
  }
});

playerPrefClearBookmarksButton?.addEventListener('click', () => {
  const count = clearCurrentPlaybackBookmarks();
  if (!count) {
    setStatus('muted', '当前片目还没有播放书签');
    return;
  }
  setStatus('success', `已清空 ${count} 个播放书签`);
});

playerPrefCopyActiveLinkButton?.addEventListener('click', () => {
  copyActivePlaybackUrl().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制当前播放链接失败');
  });
});

document.querySelectorAll('[data-library-tab]').forEach((button) => {
  button.addEventListener('click', () => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    const nextTab = button.dataset.libraryTab;
    if (nextTab !== 'history' && nextTab !== 'favorites') {
      return;
    }
    detailState.libraryTab = nextTab;
    renderLibraryPanel();
  });
});

clearLibraryButton?.addEventListener('click', () => {
  clearCurrentLibrary().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '清空失败');
  });
});

playerBookmarksListEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const jumpButton = target.closest('[data-player-bookmark-jump]');
  if (jumpButton instanceof HTMLButtonElement) {
    try {
      const bookmark = jumpToPlaybackBookmark(String(jumpButton.dataset.playerBookmarkJump || ''));
      setStatus('success', `已跳转到书签「${bookmark.label}」`);
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '跳转播放书签失败');
    }
    return;
  }

  const copyButton = target.closest('[data-player-bookmark-copy]');
  if (copyButton instanceof HTMLButtonElement) {
    copyPlaybackBookmarkLink(String(copyButton.dataset.playerBookmarkCopy || '')).then((bookmark) => {
      setStatus('success', `已复制书签链接「${bookmark.label}」`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制书签链接失败');
    });
    return;
  }

  const deleteButton = target.closest('[data-player-bookmark-delete]');
  if (deleteButton instanceof HTMLButtonElement) {
    const bookmark = removePlaybackBookmark(String(deleteButton.dataset.playerBookmarkDelete || ''));
    if (bookmark) {
      setStatus('success', `已删除书签「${bookmark.label}」`);
    }
  }
});

refreshSourcesButton?.addEventListener('click', () => {
  loadAlternativeSources().catch((error) => {
    if (sourceListEl) {
      sourceListEl.className = 'saved-list empty-state';
      sourceListEl.textContent = error instanceof Error ? error.message : '刷新来源失败';
    }
    if (sourceCountEl) {
      sourceCountEl.textContent = '来源 0';
    }
  }).finally(() => {
    if (refreshSourcesButton instanceof HTMLButtonElement) {
      refreshSourcesButton.disabled = false;
    }
  });
});

seekBackwardButton?.addEventListener('click', () => {
  handleSkipGesture('backward');
});

seekForwardButton?.addEventListener('click', () => {
  handleSkipGesture('forward');
});

fullscreenButton?.addEventListener('click', () => {
  toggleFullscreen().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '无法切换全屏');
  });
});

pipButton?.addEventListener('click', () => {
  togglePictureInPicture().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '无法切换画中画');
  });
});

remotePlaybackButton?.addEventListener('click', () => {
  openRemotePlayback().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '无法打开投屏或分享');
  });
});

copyStreamButton?.addEventListener('click', () => {
  copyCurrentPlaybackUrl().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制播放地址失败');
  });
});

favoriteToggleEl?.addEventListener('click', () => {
  toggleFavorite().catch((error) => {
    favoriteStatusEl.textContent = error instanceof Error ? error.message : '收藏操作失败';
    favoriteStatusEl.className = 'status error';
  });
});

prevButton?.addEventListener('click', () => {
  const nextIndex = detailState.currentEpisode - getEpisodeStep();
  if (nextIndex >= 0 && nextIndex < detailState.episodes.length) {
    loadEpisode(nextIndex);
  }
});

nextButton?.addEventListener('click', () => {
  const nextIndex = detailState.currentEpisode + getEpisodeStep();
  if (nextIndex >= 0 && nextIndex < detailState.episodes.length) {
    loadEpisode(nextIndex);
  }
});

libraryEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-library-remove]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const index = Number(button.dataset.libraryRemove);
  removeLibraryItem(index).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '移除失败');
  });
});

document.addEventListener('keydown', (event) => {
  if (event.defaultPrevented || event.repeat || isEditableTarget(event.target)) {
    return;
  }
  if (event.metaKey || event.ctrlKey || event.altKey) {
    return;
  }

  switch (event.key) {
    case ' ':
      event.preventDefault();
      togglePlayback();
      break;
    case 'ArrowLeft':
      event.preventDefault();
      handleSkipGesture('backward');
      break;
    case 'ArrowRight':
      event.preventDefault();
      handleSkipGesture('forward');
      break;
    case 'ArrowUp':
      event.preventDefault();
      syncVolume(0.1);
      break;
    case 'ArrowDown':
      event.preventDefault();
      syncVolume(-0.1);
      break;
    case 'f':
    case 'F':
      event.preventDefault();
      toggleFullscreen().catch(() => {});
      break;
    case 'p':
    case 'P':
      event.preventDefault();
      togglePictureInPicture().catch(() => {});
      break;
    case 'm':
    case 'M':
      event.preventDefault();
      if (videoEl instanceof HTMLVideoElement) {
        videoEl.muted = !videoEl.muted;
        setStatus('muted', videoEl.muted ? '已静音' : '已取消静音');
      }
      break;
    case 'c':
    case 'C':
      event.preventDefault();
      copyCurrentPlaybackUrl().catch(() => {});
      break;
    case '[':
      event.preventDefault();
      if (hasPreviousEpisode()) {
        loadEpisode(detailState.currentEpisode - getEpisodeStep());
      }
      break;
    case ']':
      event.preventDefault();
      if (hasNextEpisode()) {
        loadEpisode(detailState.currentEpisode + getEpisodeStep());
      }
      break;
    case 'Escape':
      if (detailState.windowFullscreen) {
        event.preventDefault();
        toggleFullscreen().catch(() => {});
      }
      break;
    default:
      break;
  }
});

videoEl?.addEventListener('error', () => {
  handlePlaybackFailure('播放器加载失败，请尝试切换代理或重新加载');
});

videoEl?.addEventListener('dblclick', (event) => {
  const side = resolveTapSide(event.clientX);
  if (!side) {
    return;
  }
  event.preventDefault();
  handleVideoDoubleTap(side);
});

videoEl?.addEventListener('touchend', (event) => {
  const touch = event.changedTouches?.[0];
  if (!touch) {
    return;
  }
  const side = resolveTapSide(touch.clientX);
  if (!side) {
    return;
  }
  const now = Date.now();
  const isSameSideDoubleTap = detailState.lastTapSide === side && now - detailState.lastTapAt < 300;
  detailState.lastTapAt = now;
  detailState.lastTapSide = side;
  if (!isSameSideDoubleTap) {
    return;
  }
  event.preventDefault();
  detailState.lastTapAt = 0;
  detailState.lastTapSide = '';
  handleVideoDoubleTap(side);
});

videoEl?.addEventListener('loadedmetadata', () => {
  applyPlaybackRate();
  maybeResumePlayback();
  updateProgressIndicator();
});

videoEl?.addEventListener('canplay', () => {
  applyPlaybackRate();
  maybeResumePlayback();
  updateProgressIndicator();
});

videoEl?.addEventListener('timeupdate', () => {
  maybeAutoSkipSegments();
  updateProgressIndicator();
  const now = Date.now();
  if (now - detailState.lastSavedAt < 5000) {
    return;
  }
  detailState.lastSavedAt = now;
  saveHistory().catch(() => {});
});

videoEl?.addEventListener('play', () => {
  setShortcutHint();
  startStallMonitor();
});

videoEl?.addEventListener('playing', () => {
  setPreferredSourceForTitle(currentDisplayTitle(), playerState.source);
  updateSourcePreferenceButton();
  renderSourceResults();
  clearFailedSourceTrailFromUrl();
  stallState.active = false;
  stallState.lastMovementAt = Date.now();
  stallState.lastCurrentTime = Number(videoEl?.currentTime || 0);
  setStatus('success', `正在播放 ${getCurrentEpisodeLabel()}`);
});

videoEl?.addEventListener('waiting', () => {
  setStatus('muted', '视频缓冲中...');
});

videoEl?.addEventListener('pause', () => {
  stallState.active = false;
  if (typeof statusEl?.textContent === 'string'
      && (statusEl.textContent.includes('视频缓冲中') || statusEl.textContent.includes('检测到播放卡顿'))) {
    setStatus('muted', '已暂停播放');
  }
});

videoEl?.addEventListener('volumechange', () => {
  if (videoEl instanceof HTMLVideoElement) {
    const volumeLabel = videoEl.muted ? '静音' : `音量 ${Math.round(videoEl.volume * 100)}%`;
    setShortcutHint(` 当前 ${volumeLabel}。`);
  }
});

videoEl?.addEventListener('ended', () => {
  clearStallMonitor();
  saveHistory().catch(() => {});
  if (settings.autoNextEpisode) {
    const nextIndex = detailState.currentEpisode + getEpisodeStep();
    if (nextIndex >= 0 && nextIndex < detailState.episodes.length) {
      loadEpisode(nextIndex);
    }
  }
});

['fullscreenchange', 'webkitfullscreenchange', 'mozfullscreenchange', 'MSFullscreenChange'].forEach((eventName) => {
  document.addEventListener(eventName, () => {
    if (!isNativeFullscreenActive()) {
      unlockOrientation();
    }
    updatePlaybackButtons();
  });
});

window.addEventListener('resize', syncWindowFullscreenLayout);
window.addEventListener('orientationchange', syncWindowFullscreenLayout);

videoEl?.addEventListener('enterpictureinpicture', () => {
  updatePlaybackButtons();
});

videoEl?.addEventListener('leavepictureinpicture', () => {
  updatePlaybackButtons();
});

videoEl?.addEventListener('webkitpresentationmodechanged', () => {
  updatePlaybackButtons();
});

videoEl?.addEventListener('webkitplaybacktargetavailabilitychanged', (event) => {
  updateAirPlayAvailability(event?.availability);
});

window.addEventListener('beforeunload', () => {
  clearPlaybackPolling();
  clearStallMonitor();
  syncWindowFullscreenLayout();
  if (detailState.windowFullscreen) {
    document.body.classList.remove('player-web-fullscreen');
  }
  destroyHls();
  saveHistory().catch(() => {});
});

updateModeBadge();
applyPlaybackRate();
refreshResumeState();
updateProgressIndicator();
updatePlaybackButtons();
syncWindowFullscreenLayout();
syncPlayerPreferenceControls();
setPlayerPreferencesPanel(false);
setShortcutHint();
renderLibraryPanel();
mergeInitialGroupedSources();
renderSourceResults();
startPlaybackPolling();
loadDetail().catch((error) => {
  metadataEl.className = 'empty-state';
  metadataEl.textContent = error instanceof Error ? error.message : '加载播放器失败';
  episodesEl.className = 'episodes-grid empty-state';
  episodesEl.textContent = '无法加载选集';
  setStatus('error', error instanceof Error ? error.message : '加载播放器失败');
  favoriteStatusEl.textContent = '无法读取收藏状态';
  favoriteStatusEl.className = 'status error';
});
"#;

pub(super) const PREMIUM_PAGE_SCRIPT: &str = r#"
const premiumState = JSON.parse(document.getElementById('premium-state').textContent || '{}');
const statusEl = document.getElementById('premium-status');
const tagsEl = document.getElementById('premium-tags');
const tagCountEl = document.getElementById('premium-tag-count');
const resultsEl = document.getElementById('premium-results');
const pageIndicatorEl = document.getElementById('premium-page-indicator');
const refreshButton = document.getElementById('premium-refresh');
const loadMoreButton = document.getElementById('premium-load-more');
const premiumSearchForm = document.getElementById('premium-search-form');
const premiumSearchInput = document.getElementById('premium-search-query');
const premiumSearchHistoryDropdownEl = document.getElementById('premium-search-history-dropdown');
const premiumSearchClearButton = document.getElementById('premium-search-clear');
const premiumSearchSection = document.getElementById('premium-search-section');
const premiumSearchResultsEl = document.getElementById('premium-search-results');
const premiumSearchSummaryEl = document.getElementById('premium-search-summary');
const premiumSearchProgressEl = document.getElementById('premium-search-progress');
const premiumSearchTotalEl = document.getElementById('premium-search-total');
const premiumSearchDisplayToggleEl = document.getElementById('premium-search-display-toggle');
const premiumSearchSortSelectEl = document.getElementById('premium-search-sort-select');
const premiumSearchFilterToolbarEl = document.getElementById('premium-search-filter-toolbar');
const premiumSearchFilterSummaryEl = document.getElementById('premium-search-filter-summary');
const clearPremiumSearchFiltersButton = document.getElementById('clear-premium-search-filters');
const premiumSearchSourceBadgesEl = document.getElementById('premium-search-source-badges');
const premiumSearchTypeBadgesEl = document.getElementById('premium-search-type-badges');
const premiumDiscoverySectionsEl = document.getElementById('premium-discovery-sections');
const premiumHistoryListEl = document.getElementById('premium-history-list');
const premiumHistoryCountEl = document.getElementById('premium-history-count');
const premiumHistoryClearButton = document.getElementById('premium-history-clear');
const premiumLibraryToggleEl = document.getElementById('premium-library-toggle');
const premiumLibraryPanelEls = document.querySelectorAll('[data-premium-library-panel]');
const premiumLibraryMainPanelEls = document.querySelectorAll('[data-premium-library-main-panel]');
const premiumLibraryStatusEl = document.getElementById('premium-library-status');
const premiumLibraryOverlayEl = document.getElementById('premium-library-overlay');
const premiumLibraryDrawerEl = document.getElementById('premium-library-drawer');
const openPremiumLibraryDrawerButtons = document.querySelectorAll('[data-open-premium-library]');
const closePremiumLibraryDrawerButton = document.getElementById('close-premium-library-drawer');
const premiumLibrarySummaryEl = document.getElementById('premium-library-summary');
const premiumLibraryFilterInputs = document.querySelectorAll('[data-premium-library-filter]');
const clearPremiumLibraryFilterButtons = document.querySelectorAll('[data-premium-library-filter-clear]');
const premiumLibrarySortInputs = document.querySelectorAll('[data-premium-library-sort]');
const clearPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-clear]');
const togglePremiumLibrarySelectionButtons = document.querySelectorAll('[data-premium-library-selection-toggle]');
const selectAllPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-select-all]');
const removeSelectedPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-remove-selected]');
const undoPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-undo]');
const copyPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-copy]');
const sharePremiumLibraryButtons = document.querySelectorAll('[data-premium-library-share]');
const shareLinkPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-share-link]');
const shareLinkMergePremiumLibraryButtons = document.querySelectorAll('[data-premium-library-share-link-merge]');
const nativeSharePremiumLibraryButtons = document.querySelectorAll('[data-premium-library-share-native]');
const savePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-save]');
const renamePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-rename]');
const duplicatePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-duplicate]');
const restorePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-restore]');
const mergePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-merge]');
const deletePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-delete]');
const exportPremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-export]');
const sharePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-share]');
const shareLinkPremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-share-link]');
const shareLinkMergePremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-share-link-merge]');
const importPremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-import]');
const mergeImportPremiumLibrarySnapshotButtons = document.querySelectorAll('[data-premium-library-snapshot-import-merge]');
const exportPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-export]');
const importPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-import]');
const mergeImportPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-import-merge]');
const clipboardImportPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-import-clipboard]');
const clipboardMergeImportPremiumLibraryButtons = document.querySelectorAll('[data-premium-library-import-clipboard-merge]');
const dedupePremiumLibraryButtons = document.querySelectorAll('[data-premium-library-dedupe]');
const premiumLibraryImportFileInput = document.getElementById('premium-library-import-file');
const premiumLibrarySnapshotImportFileInput = document.getElementById('premium-library-snapshot-import-file');
const premiumTagManageToggleButton = document.getElementById('premium-tag-manage-toggle');
const restorePremiumTagsButton = document.getElementById('restore-premium-tags');
const exportPremiumTagsButton = document.getElementById('export-premium-tags');
const importPremiumTagsTriggerButton = document.getElementById('import-premium-tags-trigger');
const importPremiumTagsMergeTriggerButton = document.getElementById('import-premium-tags-merge-trigger');
const sharePremiumTagsButton = document.getElementById('share-premium-tags');
const sharePremiumTagsLinkButton = document.getElementById('share-premium-tags-link');
const sharePremiumTagsLinkMergeButton = document.getElementById('share-premium-tags-link-merge');
const premiumTagsImportFileInput = document.getElementById('premium-tags-import-file');
const PREMIUM_SEARCH_HISTORY_KEY = 'kvideo-premium-search-history';
const PREMIUM_TAGS_STORAGE_KEY = 'kvideo_premium_custom_tags';
const PREMIUM_SEARCH_LATENCY_INTERVAL_MS = 5000;
const PREMIUM_LIBRARY_FILTERS_STORAGE_KEY = 'kvideo-premium-library-filters';
const PREMIUM_LIBRARY_SORTS_STORAGE_KEY = 'kvideo-premium-library-sorts';
const PREMIUM_LIBRARY_SNAPSHOTS_STORAGE_KEY = 'kvideo-premium-library-snapshots';

const pageState = {
  tags: [],
  selectedTag: 'recommend',
  page: 1,
  loading: false,
  hasMore: true,
  videos: [],
  historyItems: Array.isArray(premiumState.history) ? premiumState.history : [],
  favoriteItems: Array.isArray(premiumState.favorites) ? premiumState.favorites : [],
  librarySelectionMode: false,
  selectedLibraryItems: {
    history: new Set(),
    favorites: new Set(),
  },
  libraryFilters: loadPremiumLibraryFilters(),
  librarySorts: loadPremiumLibrarySorts(),
  libraryUndo: null,
  searchQuery: typeof premiumState.initialQuery === 'string' ? premiumState.initialQuery : '',
  hasSearched: Boolean(typeof premiumState.initialQuery === 'string' && premiumState.initialQuery.trim()),
  searchResults: [],
  libraryTab: 'history',
  historyDropdownOpen: false,
  historyHighlightedIndex: -1,
  historyBlurTimer: 0,
  historySubmitLockUntil: 0,
  drawerLastFocused: null,
  drawerTrapHandler: null,
  liveLatencies: {},
  sourceBaseUrls: {},
  latencyTimer: 0,
  latencyRequestId: 0,
  settingsSignature: '',
  loadMoreObserver: null,
  managingTags: false,
  selectedSources: new Set(),
  selectedTypes: new Set(),
  displayMode: premiumState.searchDisplayMode === 'grouped' ? 'grouped' : 'normal',
  sortBy: typeof premiumState.sortBy === 'string' ? premiumState.sortBy : 'default',
};

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('\"', '&quot;')
    .replaceAll("'", '&#39;');
}

function getNormalizedLibrarySort(value) {
  switch (String(value || '')) {
    case 'recent-asc':
    case 'title-asc':
    case 'title-desc':
    case 'source-asc':
    case 'source-desc':
      return String(value);
    case 'recent-desc':
    default:
      return 'recent-desc';
  }
}

function getLibrarySortLabel(value) {
  switch (getNormalizedLibrarySort(value)) {
    case 'recent-asc':
      return '最早添加';
    case 'title-asc':
      return '标题 A-Z';
    case 'title-desc':
      return '标题 Z-A';
    case 'source-asc':
      return '来源 A-Z';
    case 'source-desc':
      return '来源 Z-A';
    case 'recent-desc':
    default:
      return '最近添加';
  }
}

function sortLibraryEntries(entries, sortBy) {
  const normalizedSort = getNormalizedLibrarySort(sortBy);
  return [...entries].sort((left, right) => {
    const leftTitle = String(left?.item?.title || '').trim();
    const rightTitle = String(right?.item?.title || '').trim();
    const leftSource = String(left?.item?.sourceName || left?.item?.source || '').trim();
    const rightSource = String(right?.item?.sourceName || right?.item?.source || '').trim();

    switch (normalizedSort) {
      case 'recent-asc':
        return left.index - right.index;
      case 'title-asc':
        return leftTitle.localeCompare(rightTitle, 'zh-CN') || left.index - right.index;
      case 'title-desc':
        return rightTitle.localeCompare(leftTitle, 'zh-CN') || left.index - right.index;
      case 'source-asc':
        return leftSource.localeCompare(rightSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'source-desc':
        return rightSource.localeCompare(leftSource, 'zh-CN')
          || leftTitle.localeCompare(rightTitle, 'zh-CN')
          || left.index - right.index;
      case 'recent-desc':
      default:
        return right.index - left.index;
    }
  });
}

function encodeLibrarySharePackage(payload) {
  const json = JSON.stringify(payload);
  const utf8 = encodeURIComponent(json).replace(/%([0-9A-F]{2})/g, (_, hex) =>
    String.fromCharCode(Number.parseInt(hex, 16))
  );
  return btoa(utf8).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/g, '');
}

function decodeLibrarySharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://library/',
    'kvideo://library-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  const json = decodeURIComponent(percentEncoded);
  return JSON.parse(json);
}

function buildLibraryShareUrl(pathname, kind, merge = false) {
  const sharePackage = pathname === '/premium'
    ? buildPremiumLibrarySharePackage(kind)
    : buildHomeLibrarySharePackage(kind);
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  return `${window.location.origin}${pathname}?${params.toString()}`;
}

function readLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  const libraryShare = String(params.get('libraryShare') || '').trim();
  if (!libraryShare) {
    return null;
  }
  return {
    rawValue: libraryShare,
    merge: params.get('libraryShareMode') === 'merge',
  };
}

function clearLibraryShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('libraryShare');
  params.delete('libraryShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function setStatus(kind, message) {
  statusEl.textContent = message;
  statusEl.className = `status ${kind}`;
}

function getEnabledPremiumSources() {
  const sources = Array.isArray(premiumState.premiumSources) ? premiumState.premiumSources : [];
  return sources.filter((source) => source && source.enabled !== false);
}

function readStoredPremiumSettings() {
  if (typeof window === 'undefined') {
    return {};
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem('kvideo-settings') || '{}');
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
  } catch (_) {
    return {};
  }
}

function getPremiumSettingsSignature(settings) {
  const enabledSources = Array.isArray(settings?.premiumSources)
    ? settings.premiumSources
        .filter((source) => source && typeof source.id === 'string' && source.enabled !== false)
        .map((source) => ({
          id: String(source.id || ''),
          enabled: source.enabled !== false,
          baseUrl: String(source.baseUrl || ''),
        }))
    : [];
  return JSON.stringify({
    premiumSources: enabledSources,
    realtimeLatency: settings?.realtimeLatency === true,
    searchHistory: settings?.searchHistory !== false,
    searchDisplayMode: settings?.searchDisplayMode === 'grouped' ? 'grouped' : 'normal',
    sortBy: typeof settings?.sortBy === 'string' ? settings.sortBy : 'default',
  });
}

function syncPremiumSettings(nextSettings, options = {}) {
  const normalized = nextSettings && typeof nextSettings === 'object' && !Array.isArray(nextSettings)
    ? nextSettings
    : {};
  const previousSignature = pageState.settingsSignature;
  Object.keys(premiumState).forEach((key) => delete premiumState[key]);
  Object.assign(premiumState, normalized);
  premiumState.initialQuery = typeof premiumState.initialQuery === 'string' ? premiumState.initialQuery : pageState.searchQuery;
  pageState.settingsSignature = getPremiumSettingsSignature(premiumState);
  pageState.displayMode = premiumState.searchDisplayMode === 'grouped' ? 'grouped' : 'normal';
  pageState.sortBy = typeof premiumState.sortBy === 'string' ? premiumState.sortBy : 'default';
  renderPremiumSearchHistory();
  renderPremiumSearchResults();

  if (premiumSearchSortSelectEl instanceof HTMLSelectElement) {
    premiumSearchSortSelectEl.value = pageState.sortBy;
  }

  const shouldReload = options.reload !== false
    && previousSignature
    && previousSignature !== pageState.settingsSignature;
  if (!shouldReload) {
    return;
  }

  if (pageState.hasSearched && pageState.searchQuery) {
    runPremiumSearch(pageState.searchQuery).catch((error) => {
      if (premiumSearchProgressEl instanceof HTMLElement) {
        premiumSearchProgressEl.textContent = '搜索失败';
      }
      setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
    });
    return;
  }

  refreshPage().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '刷新 Premium 内容失败');
  });
}

function buildPremiumLibraryPlayerUrl(item) {
  const videoId = item?.videoId ?? item?.vod_id;
  const source = item?.source;
  const title = item?.title || '未知视频';
  if (!videoId || !source) {
    return '/player?premium=1';
  }
  const params = new URLSearchParams({
    id: String(videoId),
    source: String(source),
    title: String(title),
    premium: '1',
  });
  const episodeIndex = Number(item?.episodeIndex);
  if (Number.isInteger(episodeIndex) && episodeIndex >= 0) {
    params.set('episode', String(episodeIndex));
  }
  return `/player?${params.toString()}`;
}

function buildPremiumLibraryDetailUrl(item) {
  const videoId = item?.videoId ?? item?.vod_id;
  const source = item?.source;
  const title = item?.title || '未知视频';
  if (!videoId || !source) {
    return '/detail?premium=1';
  }
  const params = new URLSearchParams({
    id: String(videoId),
    source: String(source),
    title: String(title),
    premium: '1',
  });
  return `/detail?${params.toString()}`;
}

function getPremiumLibraryItems(kind) {
  return kind === 'favorites' ? pageState.favoriteItems : pageState.historyItems;
}

function setPremiumLibraryItems(kind, items) {
  if (kind === 'favorites') {
    pageState.favoriteItems = items;
  } else {
    pageState.historyItems = items;
  }
  getPremiumSelectedSet(kind).clear();
}

function getPremiumSelectedSet(kind) {
  return kind === 'favorites'
    ? pageState.selectedLibraryItems.favorites
    : pageState.selectedLibraryItems.history;
}

function getPremiumLibraryStorageKey(kind) {
  return kind === 'favorites' ? 'kvideo-premium-favorites-store' : 'kvideo-premium-history-store';
}

function getPremiumLibraryDataKey(kind) {
  return kind === 'favorites' ? 'premium-favorites' : 'premium-history';
}

function getPremiumLibraryLabel(kind) {
  return kind === 'favorites' ? 'Premium 收藏' : 'Premium 历史';
}

function getPremiumLibraryFilter(kind) {
  return kind === 'favorites'
    ? pageState.libraryFilters.favorites
    : pageState.libraryFilters.history;
}

function setPremiumLibraryFilter(kind, value) {
  if (kind === 'favorites') {
    pageState.libraryFilters.favorites = value;
  } else {
    pageState.libraryFilters.history = value;
  }
}

function getPremiumLibrarySort(kind) {
  return kind === 'favorites'
    ? pageState.librarySorts.favorites
    : pageState.librarySorts.history;
}

function setPremiumLibrarySort(kind, value) {
  const nextSort = getNormalizedLibrarySort(value);
  if (kind === 'favorites') {
    pageState.librarySorts.favorites = nextSort;
  } else {
    pageState.librarySorts.history = nextSort;
  }
}

function loadPremiumLibraryFilters() {
  if (typeof window === 'undefined') {
    return { history: '', favorites: '' };
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PREMIUM_LIBRARY_FILTERS_STORAGE_KEY) || '{}');
    return {
      history: typeof parsed?.history === 'string' ? parsed.history : '',
      favorites: typeof parsed?.favorites === 'string' ? parsed.favorites : '',
    };
  } catch (_) {
    return { history: '', favorites: '' };
  }
}

function persistPremiumLibraryFilters() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(PREMIUM_LIBRARY_FILTERS_STORAGE_KEY, JSON.stringify({
    history: pageState.libraryFilters.history,
    favorites: pageState.libraryFilters.favorites,
  }));
}

function loadPremiumLibrarySorts() {
  if (typeof window === 'undefined') {
    return { history: 'recent-desc', favorites: 'recent-desc' };
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PREMIUM_LIBRARY_SORTS_STORAGE_KEY) || '{}');
    return {
      history: getNormalizedLibrarySort(parsed?.history),
      favorites: getNormalizedLibrarySort(parsed?.favorites),
    };
  } catch (_) {
    return { history: 'recent-desc', favorites: 'recent-desc' };
  }
}

function persistPremiumLibrarySorts() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(PREMIUM_LIBRARY_SORTS_STORAGE_KEY, JSON.stringify({
    history: getPremiumLibrarySort('history'),
    favorites: getPremiumLibrarySort('favorites'),
  }));
}

function updatePremiumLibraryFilterInputs() {
  const value = getPremiumLibraryFilter(pageState.libraryTab);
  premiumLibraryFilterInputs.forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.value = value;
    }
  });
  clearPremiumLibraryFilterButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = !value;
    }
  });
}

function updatePremiumLibrarySortInputs() {
  const value = getPremiumLibrarySort(pageState.libraryTab);
  premiumLibrarySortInputs.forEach((input) => {
    if (input instanceof HTMLSelectElement) {
      input.value = value;
    }
  });
}

function filterPremiumLibraryEntries(kind) {
  const filterText = String(getPremiumLibraryFilter(kind) || '').trim().toLowerCase();
  const items = getPremiumLibraryItems(kind).map((item, index) => ({ item, index }));
  if (!filterText) {
    return items;
  }
  return items.filter(({ item }) => {
    const title = String(item?.title || '').toLowerCase();
    const source = String(item?.sourceName || item?.source || '').toLowerCase();
    return title.includes(filterText) || source.includes(filterText);
  });
}

function getSortedPremiumLibraryEntries(kind) {
  return sortLibraryEntries(filterPremiumLibraryEntries(kind), getPremiumLibrarySort(kind));
}

function getSelectedPremiumLibraryEntries(kind) {
  const selectedItems = getPremiumSelectedSet(kind);
  if (!selectedItems.size) {
    return [];
  }
  return getPremiumLibraryItems(kind)
    .map((item, index) => ({ item, index }))
    .filter(({ index }) => selectedItems.has(index));
}

function getPremiumLibraryActionEntries(kind) {
  const selectedEntries = getSelectedPremiumLibraryEntries(kind);
  if (selectedEntries.length) {
    return {
      entries: selectedEntries,
      usingSelection: true,
      selectedCount: selectedEntries.length,
    };
  }
  return {
    entries: getPremiumLibraryItems(kind).map((item, index) => ({ item, index })),
    usingSelection: false,
    selectedCount: 0,
  };
}

function normalizeImportedLibraryItem(item) {
  if (!item || typeof item !== 'object') {
    return null;
  }
  if (item.raw && typeof item.raw === 'object') {
    return item.raw;
  }
  const videoId = item.videoId ?? item.vod_id ?? '';
  const normalized = {
    videoId: videoId ? String(videoId) : '',
    title: String(item.title || '未知视频'),
    source: String(item.source || ''),
    sourceName: String(item.sourceName || item.source || ''),
  };
  if (item.premium === true) {
    normalized.premium = true;
  }
  const episodeIndex = Number(item.episodeIndex);
  if (Number.isInteger(episodeIndex) && episodeIndex >= 0) {
    normalized.episodeIndex = episodeIndex;
  }
  return normalized;
}

function getLibraryItemIdentity(item) {
  const source = String(item?.source || '').trim();
  const videoId = String(item?.videoId ?? item?.vod_id ?? '').trim();
  const title = String(item?.title || '').trim().toLowerCase();
  const episodeIndex = Number.isInteger(Number(item?.episodeIndex)) ? Number(item?.episodeIndex) : -1;
  return [source, videoId || title, episodeIndex].join('::');
}

function dedupeLibraryItems(items) {
  const seen = new Set();
  return items.filter((item) => {
    const identity = getLibraryItemIdentity(item);
    if (!identity || seen.has(identity)) {
      return false;
    }
    seen.add(identity);
    return true;
  });
}

function parseLibraryImportPayload(rawText) {
  const sharedPayload = decodeLibrarySharePackage(rawText);
  let parsed;
  if (sharedPayload) {
    parsed = sharedPayload;
  } else {
    try {
      parsed = JSON.parse(rawText);
    } catch (_) {
      throw new Error('导入内容不是有效的 JSON 或分享包');
    }
  }
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.items)) {
    throw new Error('导入文件缺少 items 列表');
  }
  const items = parsed.items
    .map((item) => normalizeImportedLibraryItem(item))
    .filter((item) => item && item.title && item.source);
  if (!items.length) {
    throw new Error('导入文件中没有可用条目');
  }
  return { kind: parsed.kind, items };
}

function normalizeImportedLibrarySnapshot(snapshot) {
  if (!snapshot || typeof snapshot !== 'object') {
    return null;
  }
  const kind = snapshot.kind === 'favorites' ? 'favorites' : snapshot.kind === 'history' ? 'history' : null;
  const name = String(snapshot.name || '').trim();
  if (!kind || !name || !Array.isArray(snapshot.items)) {
    return null;
  }
  const items = snapshot.items
    .map((item) => normalizeImportedLibraryItem(item))
    .filter((item) => item && item.title && item.source);
  if (!items.length) {
    return null;
  }
  return {
    name,
    kind,
    savedAt: typeof snapshot.savedAt === 'string' ? snapshot.savedAt : new Date().toISOString(),
    items,
  };
}

function parseLibrarySnapshotImportPayload(rawText) {
  const sharedPayload = decodeLibrarySnapshotSharePackage(rawText);
  let parsed;
  if (sharedPayload) {
    parsed = sharedPayload;
  } else {
    try {
      parsed = JSON.parse(rawText);
    } catch (_) {
      throw new Error('快照导入内容不是有效的 JSON 或快照分享包');
    }
  }
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.snapshots)) {
    throw new Error('快照导入文件缺少 snapshots 列表');
  }
  const snapshots = parsed.snapshots
    .map((snapshot) => normalizeImportedLibrarySnapshot(snapshot))
    .filter(Boolean);
  if (!snapshots.length) {
    throw new Error('快照导入文件中没有可用快照');
  }
  return {
    page: typeof parsed.page === 'string' ? parsed.page : '',
    snapshots,
  };
}

function loadPremiumLibrarySnapshots() {
  if (typeof window === 'undefined') {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PREMIUM_LIBRARY_SNAPSHOTS_STORAGE_KEY) || '[]');
    return Array.isArray(parsed) ? parsed.filter((snapshot) =>
      snapshot
      && typeof snapshot === 'object'
      && typeof snapshot.name === 'string'
      && (snapshot.kind === 'history' || snapshot.kind === 'favorites')
      && Array.isArray(snapshot.items)
    ) : [];
  } catch (_) {
    return [];
  }
}

function persistPremiumLibrarySnapshots(snapshots) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(PREMIUM_LIBRARY_SNAPSHOTS_STORAGE_KEY, JSON.stringify(snapshots));
}

function getPremiumLibrarySnapshots(kind) {
  return loadPremiumLibrarySnapshots().filter((snapshot) => snapshot.kind === kind);
}

function promptForPremiumSnapshotName(kind, actionLabel, existingSnapshots) {
  const existingNames = existingSnapshots.map((snapshot) => snapshot.name).join('、');
  const message = existingNames
    ? `${actionLabel}${getPremiumLibraryLabel(kind)}快照。\n已有快照：${existingNames}\n请输入快照名称`
    : `${actionLabel}${getPremiumLibraryLabel(kind)}快照。\n请输入快照名称`;
  return String(window.prompt(message, '') || '').trim();
}

function promptSelectPremiumSnapshot(kind, actionLabel, snapshots) {
  if (!snapshots.length) {
    throw new Error(`当前还没有保存的${getPremiumLibraryLabel(kind)}快照`);
  }
  const names = snapshots.map((snapshot, index) => `${index + 1}. ${snapshot.name}`).join('\n');
  const rawInput = String(window.prompt(`${actionLabel}${getPremiumLibraryLabel(kind)}快照：\n${names}\n请输入快照名称或序号`, '') || '').trim();
  if (!rawInput) {
    throw new Error('未选择快照');
  }
  const index = Number.parseInt(rawInput, 10);
  if (Number.isInteger(index) && index >= 1 && index <= snapshots.length) {
    return snapshots[index - 1];
  }
  const normalized = rawInput.toLowerCase();
  const matched = snapshots.find((snapshot) => snapshot.name.toLowerCase() === normalized || snapshot.name === rawInput);
  if (!matched) {
    throw new Error('没有匹配的快照');
  }
  return matched;
}

async function savePremiumLibrarySnapshot(kind) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可保存的${getPremiumLibraryLabel(kind)}`);
  }
  const snapshots = loadPremiumLibrarySnapshots();
  const name = promptForPremiumSnapshotName(kind, action.usingSelection ? '保存已选' : '保存当前', snapshots.filter((snapshot) => snapshot.kind === kind));
  if (!name) {
    throw new Error('未输入快照名称');
  }
  const nextSnapshot = {
    name,
    kind,
    savedAt: new Date().toISOString(),
    items: action.entries.map(({ item }) => item),
  };
  const nextSnapshots = snapshots.filter((snapshot) => !(snapshot.kind === kind && snapshot.name === name));
  nextSnapshots.unshift(nextSnapshot);
  persistPremiumLibrarySnapshots(nextSnapshots.slice(0, 20));
  renderPremiumLibraryCollections();
}

function renamePremiumLibrarySnapshot(kind) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  const snapshot = promptSelectPremiumSnapshot(kind, '重命名', snapshots);
  const name = promptForPremiumSnapshotName(
    kind,
    '重命名',
    snapshots.filter((item) => item.name !== snapshot.name)
  );
  if (!name) {
    throw new Error('未输入新的快照名称');
  }
  const nextSnapshots = loadPremiumLibrarySnapshots().map((item) => (
    item.kind === kind && item.name === snapshot.name
      ? { ...item, name }
      : item
  ));
  persistPremiumLibrarySnapshots(nextSnapshots);
  renderPremiumLibraryCollections();
  return { previousName: snapshot.name, name };
}

function duplicatePremiumLibrarySnapshot(kind) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  const snapshot = promptSelectPremiumSnapshot(kind, '克隆', snapshots);
  const name = promptForPremiumSnapshotName(kind, '克隆', snapshots);
  if (!name) {
    throw new Error('未输入新的快照名称');
  }
  const nextSnapshot = {
    ...snapshot,
    name,
    savedAt: new Date().toISOString(),
    items: snapshot.items.map((item) => ({ ...item })),
  };
  const nextSnapshots = loadPremiumLibrarySnapshots().filter((item) => !(item.kind === kind && item.name === name));
  nextSnapshots.unshift(nextSnapshot);
  persistPremiumLibrarySnapshots(nextSnapshots.slice(0, 20));
  renderPremiumLibraryCollections();
  return nextSnapshot;
}

async function restorePremiumLibrarySnapshot(kind, options = {}) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  const snapshot = promptSelectPremiumSnapshot(kind, options.merge ? '合并' : '恢复', snapshots);
  const previousItems = getPremiumLibraryItems(kind).slice();
  const nextItems = options.merge
    ? dedupeLibraryItems([...previousItems, ...snapshot.items])
    : snapshot.items.slice();
  setPremiumLibraryItems(kind, nextItems);
  pageState.libraryUndo = { kind, items: previousItems };
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
  return snapshot;
}

function deletePremiumLibrarySnapshot(kind) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  const snapshot = promptSelectPremiumSnapshot(kind, '删除', snapshots);
  if (!window.confirm(`确定删除快照「${snapshot.name}」？`)) {
    return null;
  }
  const nextSnapshots = loadPremiumLibrarySnapshots().filter((item) => !(item.kind === kind && item.name === snapshot.name));
  persistPremiumLibrarySnapshots(nextSnapshots);
  renderPremiumLibraryCollections();
  return snapshot;
}

function buildPremiumLibrarySnapshotExportPayload(kind) {
  return {
    format: 'kvideo-library-snapshots-export',
    version: 1,
    exportedAt: new Date().toISOString(),
    page: 'premium',
    kind,
    snapshots: getPremiumLibrarySnapshots(kind),
  };
}

function buildPremiumLibrarySnapshotSharePackage(kind) {
  const payload = buildPremiumLibrarySnapshotExportPayload(kind);
  return `kvideo://library-snapshots/${encodeLibrarySharePackage(payload)}`;
}

function exportPremiumLibrarySnapshots(kind) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可导出的${getPremiumLibraryLabel(kind)}快照`);
  }
  const payload = buildPremiumLibrarySnapshotExportPayload(kind);
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-premium-${kind}-snapshots-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

async function copyPremiumLibrarySnapshotSharePackage(kind) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可分享的${getPremiumLibraryLabel(kind)}快照`);
  }
  const text = buildPremiumLibrarySnapshotSharePackage(kind);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function copyPremiumLibrarySnapshotShareLink(kind, merge = false) {
  const snapshots = getPremiumLibrarySnapshots(kind);
  if (!snapshots.length) {
    throw new Error(`当前没有可分享的${getPremiumLibraryLabel(kind)}快照`);
  }
  const text = buildLibrarySnapshotShareUrl('/premium', kind, merge);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function importPremiumLibrarySnapshots(kind, file, options = {}) {
  const rawText = await file.text();
  return applyPremiumLibrarySnapshotImport(kind, rawText, options);
}

async function applyPremiumLibrarySnapshotImport(kind, rawText, options = {}) {
  const parsed = parseLibrarySnapshotImportPayload(rawText);
  const snapshots = parsed.snapshots
    .filter((snapshot) => snapshot.kind === kind)
    .map((snapshot) => ({ ...snapshot, items: snapshot.items.map((item) => ({ ...item, premium: true })) }));
  if (!snapshots.length) {
    throw new Error(`导入文件中没有${getPremiumLibraryLabel(kind)}快照`);
  }
  const existingSnapshots = loadPremiumLibrarySnapshots();
  const retainedSnapshots = existingSnapshots.filter((snapshot) => snapshot.kind !== kind);
  const nextKindSnapshots = options.merge
    ? [
        ...snapshots,
        ...existingSnapshots.filter((snapshot) =>
          snapshot.kind === kind && !snapshots.some((item) => item.name === snapshot.name)
        ),
      ]
    : snapshots;
  persistPremiumLibrarySnapshots([...retainedSnapshots, ...nextKindSnapshots].slice(0, 40));
  renderPremiumLibraryCollections();
}

async function applyPremiumLibrarySnapshotShareFromUrl() {
  const shareParams = readLibrarySnapshotShareParams();
  if (!shareParams) {
    return;
  }
  const parsed = parseLibrarySnapshotImportPayload(shareParams.rawValue);
  const targetKind = parsed.snapshots[0]?.kind === 'favorites' ? 'favorites' : 'history';
  await applyPremiumLibrarySnapshotImport(targetKind, shareParams.rawValue, {
    merge: shareParams.merge,
  });
  clearLibrarySnapshotShareParams();
  setStatus('success', shareParams.merge
    ? `已从分享链接合并导入${getPremiumLibraryLabel(targetKind)}快照`
    : `已从分享链接导入${getPremiumLibraryLabel(targetKind)}快照`);
}

async function persistPremiumLibraryItems(kind) {
  const items = getPremiumLibraryItems(kind);
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(getPremiumLibraryStorageKey(kind), JSON.stringify(items));
  }
  const response = await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: getPremiumLibraryDataKey(kind), value: items }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error || `保存 Premium ${kind === 'favorites' ? '收藏' : '历史'}失败`);
  }
}

function buildPremiumLibraryExportPayload(kind, entries = getPremiumLibraryActionEntries(kind).entries) {
  return {
    format: 'kvideo-library-export',
    version: 1,
    exportedAt: new Date().toISOString(),
    page: 'premium',
    kind,
    items: entries.map(({ item }) => ({
      raw: item,
      videoId: item?.videoId ?? item?.vod_id ?? '',
      title: item?.title || '未知视频',
      source: item?.source || '',
      sourceName: item?.sourceName || item?.source || '',
      premium: true,
      episodeIndex: Number.isInteger(Number(item?.episodeIndex)) ? Number(item.episodeIndex) : null,
      playerUrl: buildPremiumLibraryPlayerUrl(item),
      detailUrl: buildPremiumLibraryDetailUrl(item),
    })),
  };
}

function buildPremiumLibraryCopyText(kind, entries = getPremiumLibraryActionEntries(kind).entries) {
  const lines = [`# ${getPremiumLibraryLabel(kind)}清单`, ''];
  entries.forEach(({ item }, index) => {
    const title = item?.title || '未知视频';
    const source = item?.sourceName || item?.source || '未知来源';
    lines.push(`${index + 1}. ${title} [${source}]`);
    lines.push(`   播放: ${window.location.origin}${buildPremiumLibraryPlayerUrl(item)}`);
    lines.push(`   详情: ${window.location.origin}${buildPremiumLibraryDetailUrl(item)}`);
  });
  return lines.join('\n');
}

function buildPremiumLibrarySharePackage(kind) {
  const payload = buildPremiumLibraryExportPayload(kind);
  return `kvideo://library/${encodeLibrarySharePackage(payload)}`;
}

function decodeLibrarySnapshotSharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://library-snapshots/',
    'kvideo://library-snapshots-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('快照分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  const json = decodeURIComponent(percentEncoded);
  return JSON.parse(json);
}

function buildLibrarySnapshotShareUrl(pathname, kind, merge = false) {
  const sharePackage = pathname === '/premium'
    ? buildPremiumLibrarySnapshotSharePackage(kind)
    : buildHomeLibrarySnapshotSharePackage(kind);
  const params = new URLSearchParams();
  params.set('librarySnapshotsShare', sharePackage);
  if (merge) {
    params.set('librarySnapshotsShareMode', 'merge');
  }
  return `${window.location.origin}${pathname}?${params.toString()}`;
}

function readLibrarySnapshotShareParams() {
  const params = new URLSearchParams(window.location.search);
  const rawValue = String(params.get('librarySnapshotsShare') || '').trim();
  if (!rawValue) {
    return null;
  }
  return {
    rawValue,
    merge: params.get('librarySnapshotsShareMode') === 'merge',
  };
}

function clearLibrarySnapshotShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('librarySnapshotsShare');
  params.delete('librarySnapshotsShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

async function copyPremiumLibraryShareLink(kind, merge = false) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getPremiumLibraryLabel(kind)}`);
  }
  const sharePackage = `kvideo://library/${encodeLibrarySharePackage(buildPremiumLibraryExportPayload(kind, action.entries))}`;
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  const text = `${window.location.origin}/premium?${params.toString()}`;
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function sharePremiumLibraryLink(kind, merge = false) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getPremiumLibraryLabel(kind)}`);
  }
  const sharePackage = `kvideo://library/${encodeLibrarySharePackage(buildPremiumLibraryExportPayload(kind, action.entries))}`;
  const params = new URLSearchParams();
  params.set('libraryShare', sharePackage);
  if (merge) {
    params.set('libraryShareMode', 'merge');
  }
  const url = `${window.location.origin}/premium?${params.toString()}`;
  if (navigator.share) {
    await navigator.share({
      title: `${getPremiumLibraryLabel(kind)}分享`,
      text: merge
        ? `分享当前${getPremiumLibraryLabel(kind)}列表，打开后会合并导入`
        : `分享当前${getPremiumLibraryLabel(kind)}列表`,
      url,
    });
    return;
  }
  await copyPremiumLibraryShareLink(kind, merge);
}

async function copyPremiumLibraryItems(kind) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可复制的${getPremiumLibraryLabel(kind)}`);
  }
  const text = buildPremiumLibraryCopyText(kind, action.entries);
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

async function copyPremiumLibrarySharePackage(kind) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可分享的${getPremiumLibraryLabel(kind)}`);
  }
  const text = `kvideo://library/${encodeLibrarySharePackage(buildPremiumLibraryExportPayload(kind, action.entries))}`;
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('当前浏览器不支持剪贴板写入');
}

function exportPremiumLibraryItems(kind) {
  const action = getPremiumLibraryActionEntries(kind);
  if (!action.entries.length) {
    throw new Error(`当前没有可导出的${getPremiumLibraryLabel(kind)}`);
  }
  const payload = buildPremiumLibraryExportPayload(kind, action.entries);
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-premium-${kind}-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

async function importPremiumLibraryItems(kind, file, options = {}) {
  const rawText = await file.text();
  return applyPremiumLibraryImport(kind, rawText, options);
}

async function applyPremiumLibraryImport(kind, rawText, options = {}) {
  const parsed = parseLibraryImportPayload(rawText);
  const targetKind = options.targetKind || (parsed.kind === 'favorites' ? 'favorites' : kind);
  const { items } = parsed;
  const previousItems = getPremiumLibraryItems(targetKind).slice();
  const normalizedItems = items.map((item) => ({
    ...item,
    premium: true,
  }));
  const nextItems = options.merge
    ? dedupeLibraryItems([...previousItems, ...normalizedItems])
    : normalizedItems;
  setPremiumLibraryItems(targetKind, nextItems);
  pageState.libraryUndo = { kind: targetKind, items: previousItems };
  await persistPremiumLibraryItems(targetKind);
  renderPremiumLibraryCollections();
}

async function applyPremiumLibraryShareFromUrl() {
  const shareParams = readLibraryShareParams();
  if (!shareParams) {
    return;
  }
  const parsed = parseLibraryImportPayload(shareParams.rawValue);
  const targetKind = parsed.kind === 'favorites' ? 'favorites' : 'history';
  await applyPremiumLibraryImport(targetKind, shareParams.rawValue, {
    merge: shareParams.merge,
    targetKind,
  });
  clearLibraryShareParams();
  setStatus('success', shareParams.merge
    ? `已从分享链接合并导入${getPremiumLibraryLabel(targetKind)}列表，可撤销`
    : `已从分享链接导入${getPremiumLibraryLabel(targetKind)}列表，可撤销`);
}

async function dedupePremiumLibraryItems(kind) {
  const previousItems = getPremiumLibraryItems(kind).slice();
  const nextItems = dedupeLibraryItems(previousItems);
  if (nextItems.length === previousItems.length) {
    throw new Error(`当前${getPremiumLibraryLabel(kind)}没有重复项`);
  }
  setPremiumLibraryItems(kind, nextItems);
  pageState.libraryUndo = { kind, items: previousItems };
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
}

function renderPremiumLibrarySummary() {
  if (premiumLibrarySummaryEl instanceof HTMLElement) {
    const selectedCount = getPremiumSelectedSet(pageState.libraryTab).size;
    const visibleCount = getSortedPremiumLibraryEntries(pageState.libraryTab).length;
    const totalCount = getPremiumLibraryItems(pageState.libraryTab).length;
    const snapshotCount = getPremiumLibrarySnapshots(pageState.libraryTab).length;
    const parts = [`收藏 ${pageState.favoriteItems.length}`, `历史 ${pageState.historyItems.length}`];
    if (visibleCount !== totalCount) {
      parts.push(`当前显示 ${visibleCount}/${totalCount}`);
    }
    parts.push(`排序 ${getLibrarySortLabel(getPremiumLibrarySort(pageState.libraryTab))}`);
    parts.push(`快照 ${snapshotCount}`);
    if (selectedCount > 0) {
      parts.push(`已选 ${selectedCount}`);
    }
    if (pageState.libraryUndo && Array.isArray(pageState.libraryUndo.items)) {
      parts.push(`可撤销 ${pageState.libraryUndo.items.length}`);
    }
    premiumLibrarySummaryEl.textContent = parts.join(' / ');
  }
}

function renderPremiumLibraryCollection(kind) {
  const items = getSortedPremiumLibraryEntries(kind);
  const selectedItems = getPremiumSelectedSet(kind);
  const panels = [
    ...Array.from(premiumLibraryPanelEls),
    ...Array.from(premiumLibraryMainPanelEls),
  ].filter((panel) =>
    panel instanceof HTMLElement && (
      panel.dataset.premiumLibraryPanel === kind || panel.dataset.premiumLibraryMainPanel === kind
    )
  );
  const emptyMessage = kind === 'favorites'
    ? '当前没有已同步的 Premium 收藏。'
    : '当前没有可快速访问的 Premium 历史。';

  panels.forEach((panel) => {
    if (!(panel instanceof HTMLElement)) {
      return;
    }
    if (!items.length) {
      panel.classList.add('empty-state');
      panel.innerHTML = getPremiumLibraryFilter(kind)
        ? `没有匹配“${escapeHtml(getPremiumLibraryFilter(kind))}”的${kind === 'favorites' ? 'Premium 收藏' : 'Premium 历史'}。`
        : emptyMessage;
      return;
    }

    panel.classList.remove('empty-state');
    panel.innerHTML = items.slice(0, 12).map(({ item, index }) => `
      <article class="saved-item library-entry ${pageState.librarySelectionMode && selectedItems.has(index) ? 'selected' : ''}">
        <div class="stack compact-card-body">
          <strong>${escapeHtml(item?.title || '未知视频')}</strong>
          <div class="source-item-meta">
            <span>${escapeHtml(item?.sourceName || item?.source || '未知来源')}</span>
            <span>${kind === 'favorites'
              ? escapeHtml(item?.type || item?.remarks || 'Premium 收藏')
              : escapeHtml(item?.episodes?.[item?.episodeIndex]?.name || `第${Number(item?.episodeIndex ?? 0) + 1}集`)}</span>
          </div>
        </div>
        <div class="row wrap gap-sm library-actions">
          ${pageState.librarySelectionMode ? `
            <button
              class="button button-small ${selectedItems.has(index) ? 'primary' : ''}"
              type="button"
              data-premium-library-select-item="${kind}:${index}"
              aria-pressed="${selectedItems.has(index) ? 'true' : 'false'}"
            >${selectedItems.has(index) ? '取消选择' : '选择'}</button>
          ` : ''}
          <a class="button button-small" href="${escapeHtml(buildPremiumLibraryPlayerUrl(item))}">播放</a>
          <a class="button button-small" href="${escapeHtml(buildPremiumLibraryDetailUrl(item))}">详情</a>
          <button class="button button-small" type="button" data-premium-library-query="${escapeHtml(item?.title || '')}">搜同名</button>
          <button class="button button-small danger" type="button" data-premium-library-remove="${kind}:${index}">移除</button>
        </div>
      </article>
    `).join('');
  });
}

function renderPremiumLibraryCollections() {
  renderPremiumLibrarySummary();
  renderPremiumLibraryCollection('history');
  renderPremiumLibraryCollection('favorites');
  updatePremiumLibraryFilterInputs();
  updatePremiumLibrarySortInputs();
  const currentItems = getPremiumLibraryItems(pageState.libraryTab);
  const selectedCount = getPremiumSelectedSet(pageState.libraryTab).size;
  clearPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = pageState.libraryTab === 'favorites' ? '清空收藏' : '清空历史';
    }
  });
  togglePremiumLibrarySelectionButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = pageState.librarySelectionMode ? '取消批量' : '批量选择';
      button.setAttribute('aria-pressed', pageState.librarySelectionMode ? 'true' : 'false');
    }
  });
  selectAllPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0 || !pageState.librarySelectionMode;
      button.textContent = selectedCount === currentItems.length && currentItems.length > 0
        ? '已全选'
        : '全选当前列表';
    }
  });
  removeSelectedPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = selectedCount === 0 || !pageState.librarySelectionMode;
      button.textContent = selectedCount > 0 ? `移除已选 (${selectedCount})` : '移除已选';
    }
  });
  undoPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      const undoCount = Array.isArray(pageState.libraryUndo?.items) ? pageState.libraryUndo.items.length : 0;
      button.disabled = undoCount === 0;
      button.textContent = undoCount > 0 ? `撤销上次移除 (${undoCount})` : '撤销上次移除';
    }
  });
  copyPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选 (${selectedCount})`
        : `复制当前${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
  exportPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `导出已选 (${selectedCount})`
        : `导出当前${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
  sharePremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选分享包 (${selectedCount})`
        : `复制${getPremiumLibraryLabel(pageState.libraryTab)}分享包`;
    }
  });
  shareLinkPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选分享链接 (${selectedCount})`
        : `复制${getPremiumLibraryLabel(pageState.libraryTab)}分享链接`;
    }
  });
  shareLinkMergePremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `复制已选合并分享链接 (${selectedCount})`
        : `复制${getPremiumLibraryLabel(pageState.libraryTab)}合并分享链接`;
    }
  });
  nativeSharePremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = navigator.share
        ? (selectedCount > 0
          ? `分享已选 (${selectedCount})`
          : `分享当前${getPremiumLibraryLabel(pageState.libraryTab)}`)
        : (selectedCount > 0
          ? `复制已选分享链接 (${selectedCount})`
          : `复制${getPremiumLibraryLabel(pageState.libraryTab)}分享链接`);
    }
  });
  savePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length === 0;
      button.textContent = selectedCount > 0
        ? `保存已选快照 (${selectedCount})`
        : `保存${getPremiumLibraryLabel(pageState.libraryTab)}快照`;
    }
  });
  const snapshotCount = getPremiumLibrarySnapshots(pageState.libraryTab).length;
  renamePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `重命名快照 (${snapshotCount})`
        : '重命名快照';
    }
  });
  duplicatePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `克隆快照 (${snapshotCount})`
        : '克隆快照';
    }
  });
  restorePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `恢复快照 (${snapshotCount})`
        : '恢复快照';
    }
  });
  mergePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `合并快照 (${snapshotCount})`
        : '合并快照';
    }
  });
  deletePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `删除快照 (${snapshotCount})`
        : '删除快照';
    }
  });
  exportPremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `导出快照 (${snapshotCount})`
        : '导出快照';
    }
  });
  sharePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照分享包 (${snapshotCount})`
        : '复制快照分享包';
    }
  });
  shareLinkPremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照分享链接 (${snapshotCount})`
        : '复制快照分享链接';
    }
  });
  shareLinkMergePremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = snapshotCount === 0;
      button.textContent = snapshotCount > 0
        ? `复制快照合并链接 (${snapshotCount})`
        : '复制快照合并链接';
    }
  });
  importPremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = '导入快照';
    }
  });
  mergeImportPremiumLibrarySnapshotButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = '合并导入快照';
    }
  });
  mergeImportPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `合并导入${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
  clipboardImportPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `剪贴板导入${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
  clipboardMergeImportPremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.textContent = `剪贴板合并${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
  dedupePremiumLibraryButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.disabled = currentItems.length < 2;
      button.textContent = `去重当前${getPremiumLibraryLabel(pageState.libraryTab)}`;
    }
  });
}

async function clearPremiumLibraryItems(kind) {
  const previousItems = getPremiumLibraryItems(kind).slice();
  if (!window.confirm(`确定清空当前 Premium ${kind === 'favorites' ? '收藏' : '历史'}列表？`)) {
    return;
  }
  setPremiumLibraryItems(kind, []);
  pageState.libraryUndo = { kind, items: previousItems };
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
  setStatus('success', `已清空 Premium ${kind === 'favorites' ? '收藏' : '历史'}，可撤销`);
}

async function removePremiumLibraryItem(kind, index) {
  const items = getPremiumLibraryItems(kind).slice();
  const item = items[index];
  if (!item) {
    return;
  }
  if (!window.confirm(`确定移除「${item.title || '该条目'}」？`)) {
    return;
  }
  const previousItems = items.slice();
  items.splice(index, 1);
  setPremiumLibraryItems(kind, items);
  pageState.libraryUndo = { kind, items: previousItems };
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
  setStatus('success', `已移除「${item.title || '该条目'}」，可撤销`);
}

function setPremiumLibrarySelectionMode(enabled) {
  pageState.librarySelectionMode = Boolean(enabled);
  if (!pageState.librarySelectionMode) {
    getPremiumSelectedSet('history').clear();
    getPremiumSelectedSet('favorites').clear();
  }
  renderPremiumLibraryCollections();
}

function updatePremiumLibraryFilterValue(value) {
  setPremiumLibraryFilter(pageState.libraryTab, String(value || '').trim());
  persistPremiumLibraryFilters();
  renderPremiumLibraryCollections();
}

function updatePremiumLibrarySortValue(value) {
  setPremiumLibrarySort(pageState.libraryTab, value);
  persistPremiumLibrarySorts();
  renderPremiumLibraryCollections();
}

function togglePremiumLibraryItemSelection(kind, index) {
  const items = getPremiumLibraryItems(kind);
  if (!items[index]) {
    return;
  }
  const selectedItems = getPremiumSelectedSet(kind);
  if (selectedItems.has(index)) {
    selectedItems.delete(index);
  } else {
    selectedItems.add(index);
  }
  renderPremiumLibraryCollections();
}

function selectAllPremiumLibraryItems(kind) {
  const items = getPremiumLibraryItems(kind);
  const selectedItems = getPremiumSelectedSet(kind);
  selectedItems.clear();
  items.forEach((_, index) => selectedItems.add(index));
  renderPremiumLibraryCollections();
}

async function removeSelectedPremiumLibraryItems(kind) {
  const selectedItems = getPremiumSelectedSet(kind);
  if (!selectedItems.size) {
    return;
  }
  const selectedCount = selectedItems.size;
  const previousItems = getPremiumLibraryItems(kind).slice();
  if (!window.confirm(`确定移除当前 Premium ${kind === 'favorites' ? '收藏' : '历史'}列表中的 ${selectedItems.size} 项？`)) {
    return;
  }
  const items = getPremiumLibraryItems(kind).filter((_, index) => !selectedItems.has(index));
  setPremiumLibraryItems(kind, items);
  pageState.libraryUndo = { kind, items: previousItems };
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
  setStatus('success', `已批量移除 ${selectedCount} 项，可撤销`);
}

async function undoPremiumLibraryRemoval() {
  const undoState = pageState.libraryUndo;
  if (!undoState || !Array.isArray(undoState.items)) {
    return;
  }
  const kind = undoState.kind === 'favorites' ? 'favorites' : 'history';
  setPremiumLibraryItems(kind, undoState.items.slice());
  pageState.libraryUndo = null;
  await persistPremiumLibraryItems(kind);
  renderPremiumLibraryCollections();
  setStatus('success', '已撤销上次移除');
}

function currentCategoryValue() {
  const tag = pageState.tags.find((item) => item.id === pageState.selectedTag);
  return tag ? (tag.value || '') : '';
}

function premiumDetailUrl(video) {
  const params = new URLSearchParams({
    id: String(video.vod_id ?? ''),
    source: String(video.source ?? ''),
    title: String(video.vod_name ?? '未知视频'),
    premium: '1',
  });
  return `/detail?${params.toString()}`;
}

function premiumSearchUrl(video) {
  const params = new URLSearchParams({
    id: String(video.vod_id ?? ''),
    source: String(video.source ?? ''),
    title: String(video.vod_name ?? '未知视频'),
    premium: '1',
  });
  return `/player?${params.toString()}`;
}

function groupedPremiumDetailUrl(group) {
  const representative = group?.representative || {};
  const params = new URLSearchParams({
    id: String(representative.vod_id ?? ''),
    source: String(representative.source ?? ''),
    title: String(representative.vod_name ?? '未知视频'),
    premium: '1',
  });
  const groupedSources = Array.isArray(group?.videos) ? group.videos.map((video) => ({
    id: video?.vod_id ?? '',
    source: video?.source ?? '',
    sourceName: video?.sourceDisplayName || video?.sourceName || video?.source || '',
    latency: typeof getPremiumLatencyValue(video) === 'number' ? getPremiumLatencyValue(video) : undefined,
    pic: video?.vod_pic || undefined,
  })) : [];
  if (groupedSources.length > 1) {
    params.set('groupedSources', JSON.stringify(groupedSources));
  }
  return `/player?${params.toString()}`;
}

function getPremiumLatencyValue(video) {
  const liveLatency = pageState.liveLatencies[String(video?.source || '')];
  if (typeof liveLatency === 'number') {
    return liveLatency;
  }
  return typeof video?.latency === 'number' ? video.latency : null;
}

function isPremiumRealtimeLatencyEnabled() {
  return premiumState.realtimeLatency === true;
}

function stopPremiumLatencyPolling() {
  if (pageState.latencyTimer) {
    window.clearInterval(pageState.latencyTimer);
    pageState.latencyTimer = 0;
  }
  pageState.latencyRequestId += 1;
}

async function pingPremiumSourceLatency(baseUrl) {
  const response = await fetch('/api/ping', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url: baseUrl }),
  });

  if (!response.ok) {
    throw new Error(`测速失败: ${response.status}`);
  }

  const data = await response.json().catch(() => ({}));
  return typeof data.latency === 'number' ? data.latency : null;
}

async function refreshPremiumRealtimeLatencies() {
  if (!isPremiumRealtimeLatencyEnabled() || !pageState.searchResults.length) {
    return;
  }

  const requestId = ++pageState.latencyRequestId;
  const sourceEntries = Array.from(new Set(
    pageState.searchResults
      .map((video) => String(video?.source || '').trim())
      .filter(Boolean)
  ))
    .map((sourceId) => [sourceId, pageState.sourceBaseUrls[sourceId]])
    .filter(([, baseUrl]) => typeof baseUrl === 'string' && baseUrl.trim());

  if (!sourceEntries.length) {
    return;
  }

  const results = await Promise.all(
    sourceEntries.map(async ([sourceId, baseUrl]) => {
      try {
        const latency = await pingPremiumSourceLatency(baseUrl);
        return { sourceId, latency };
      } catch (_) {
        return { sourceId, latency: null };
      }
    })
  );

  if (requestId !== pageState.latencyRequestId) {
    return;
  }

  let changed = false;
  results.forEach(({ sourceId, latency }) => {
    if (typeof latency !== 'number') {
      return;
    }
    if (pageState.liveLatencies[sourceId] !== latency) {
      pageState.liveLatencies[sourceId] = latency;
      changed = true;
    }
  });

  if (changed) {
    renderPremiumSearchResults();
  }
}

function ensurePremiumLatencyPolling() {
  if (!isPremiumRealtimeLatencyEnabled() || !pageState.searchResults.length) {
    stopPremiumLatencyPolling();
    return;
  }

  if (!pageState.latencyTimer) {
    void refreshPremiumRealtimeLatencies();
    pageState.latencyTimer = window.setInterval(() => {
      void refreshPremiumRealtimeLatencies();
    }, PREMIUM_SEARCH_LATENCY_INTERVAL_MS);
  }
}

function replacePremiumSearchUrl(query) {
  const params = new URLSearchParams(window.location.search);
  if (query) {
    params.set('q', query);
  } else {
    params.delete('q');
  }
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function getPremiumSearchHistoryEnabled() {
  return premiumState.searchHistory !== false;
}

function loadPremiumSearchHistory() {
  if (typeof window === 'undefined' || !getPremiumSearchHistoryEnabled()) {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PREMIUM_SEARCH_HISTORY_KEY) || '[]');
    return Array.isArray(parsed) ? parsed.filter((item) =>
      item
      && typeof item === 'object'
      && typeof item.query === 'string'
      && item.query.trim()
    ) : [];
  } catch (_) {
    return [];
  }
}

function savePremiumSearchHistory(items) {
  if (typeof window === 'undefined' || !getPremiumSearchHistoryEnabled()) {
    return;
  }
  window.localStorage.setItem(PREMIUM_SEARCH_HISTORY_KEY, JSON.stringify(items.slice(0, 20)));
}

function filterPremiumSearchHistoryItems(query) {
  const normalized = String(query || '').trim().toLowerCase();
  const items = loadPremiumSearchHistory();
  if (!normalized) {
    return items.slice(0, 10);
  }
  return items
    .filter((item) => String(item.query || '').trim().toLowerCase().includes(normalized))
    .slice(0, 10);
}

function renderPremiumSearchHistory() {
  if (!(premiumHistoryListEl instanceof HTMLElement) || !(premiumHistoryCountEl instanceof HTMLElement)) {
    return;
  }

  const history = loadPremiumSearchHistory();
  premiumHistoryCountEl.textContent = `历史 ${history.length}`;

  if (premiumHistoryClearButton instanceof HTMLButtonElement) {
    premiumHistoryClearButton.disabled = !getPremiumSearchHistoryEnabled() || history.length === 0;
  }

  if (!getPremiumSearchHistoryEnabled()) {
    premiumHistoryListEl.className = 'saved-list empty-state';
    premiumHistoryListEl.textContent = '当前已关闭 Premium 搜索历史。';
    return;
  }

  if (!history.length) {
    premiumHistoryListEl.className = 'saved-list empty-state';
    premiumHistoryListEl.textContent = '当前还没有 Premium 搜索历史。';
    return;
  }

  premiumHistoryListEl.className = 'saved-list';
  premiumHistoryListEl.innerHTML = history.slice(0, 10).map((item, index) => `
    <article class="saved-item library-item">
      <div class="row space-between wrap gap-sm">
        <button class="button result-card-button" type="button" data-premium-history-query="${escapeHtml(item.query)}">
          <div class="library-item-main">
            <strong>${escapeHtml(item.query)}</strong>
            <span class="muted tiny">
              ${typeof item.resultCount === 'number' ? `上次结果 ${item.resultCount} 条` : '点击再次搜索'}
              · 第 ${index + 1} 条
            </span>
          </div>
        </button>
        <button class="button danger button-small" type="button" data-premium-history-remove="${escapeHtml(item.query)}">移除</button>
      </div>
    </article>
  `).join('');

  renderPremiumHistoryDropdown();
}

function updatePremiumSearchHistory(query, resultCount) {
  const trimmedQuery = String(query || '').trim();
  if (!trimmedQuery) {
    return;
  }

  const normalized = trimmedQuery.toLowerCase();
  const nextItem = {
    query: trimmedQuery,
    timestamp: Date.now(),
    resultCount: Number.isFinite(resultCount) ? resultCount : undefined,
  };
  const deduped = loadPremiumSearchHistory().filter((item) =>
    String(item.query || '').trim().toLowerCase() !== normalized
  );
  deduped.unshift(nextItem);
  savePremiumSearchHistory(deduped);
  renderPremiumSearchHistory();
}

function removePremiumSearchHistory(query) {
  const normalized = String(query || '').trim().toLowerCase();
  const nextHistory = loadPremiumSearchHistory().filter((item) =>
    String(item.query || '').trim().toLowerCase() !== normalized
  );
  savePremiumSearchHistory(nextHistory);
  renderPremiumSearchHistory();
}

function clearPremiumSearchHistory() {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.removeItem(PREMIUM_SEARCH_HISTORY_KEY);
  renderPremiumSearchHistory();
}

function setPremiumHistoryDropdown(open) {
  pageState.historyDropdownOpen = open;
  if (premiumSearchInput instanceof HTMLInputElement) {
    premiumSearchInput.setAttribute('aria-expanded', open ? 'true' : 'false');
    if (!open) {
      premiumSearchInput.removeAttribute('aria-activedescendant');
    }
  }
  if (!(premiumSearchHistoryDropdownEl instanceof HTMLElement)) {
    return;
  }
  premiumSearchHistoryDropdownEl.classList.toggle('hidden', !open);
  premiumSearchHistoryDropdownEl.hidden = !open;
  premiumSearchHistoryDropdownEl.setAttribute('aria-hidden', open ? 'false' : 'true');
}

function renderPremiumHistoryDropdown() {
  if (!(premiumSearchHistoryDropdownEl instanceof HTMLElement)) {
    return;
  }

  const allHistory = loadPremiumSearchHistory().slice(0, 10);
  const history = filterPremiumSearchHistoryItems(premiumSearchInput?.value || '');
  if (!pageState.historyDropdownOpen) {
    premiumSearchHistoryDropdownEl.classList.add('hidden');
    premiumSearchHistoryDropdownEl.hidden = true;
    return;
  }

  premiumSearchHistoryDropdownEl.classList.remove('hidden');
  premiumSearchHistoryDropdownEl.hidden = false;

  if (!history.length) {
    if (premiumSearchInput instanceof HTMLInputElement) {
      premiumSearchInput.removeAttribute('aria-activedescendant');
    }
    premiumSearchHistoryDropdownEl.innerHTML = `
      <div class="search-history-dropdown-header">
        <strong>Premium 搜索历史</strong>
      </div>
      <div class="empty-state">${allHistory.length ? '没有匹配的 Premium 搜索历史。' : '当前还没有 Premium 搜索历史。'}</div>
    `;
    return;
  }

  premiumSearchHistoryDropdownEl.innerHTML = `
    <div class="search-history-dropdown-header">
      <strong>Premium 搜索历史</strong>
      <button class="button button-small" type="button" data-premium-history-clear-all>清空</button>
    </div>
    <div class="search-history-dropdown-list">
      ${history.map((item, index) => `
        <div id="premium-search-history-option-${index}" class="search-history-dropdown-item ${index === pageState.historyHighlightedIndex ? 'active' : ''}" role="option" aria-selected="${index === pageState.historyHighlightedIndex ? 'true' : 'false'}">
          <button class="button result-card-button" type="button" data-premium-history-query="${escapeHtml(item.query)}">
            <div class="search-history-dropdown-main">
              <strong>${escapeHtml(item.query)}</strong>
              <span class="muted tiny">${typeof item.resultCount === 'number' ? `上次结果 ${item.resultCount} 条` : '点击再次搜索'}</span>
            </div>
          </button>
          <button class="button danger button-small" type="button" data-premium-history-remove="${escapeHtml(item.query)}">移除</button>
        </div>
      `).join('')}
    </div>
  `;
  if (premiumSearchInput instanceof HTMLInputElement) {
    if (pageState.historyHighlightedIndex >= 0 && history[pageState.historyHighlightedIndex]) {
      premiumSearchInput.setAttribute('aria-activedescendant', `premium-search-history-option-${pageState.historyHighlightedIndex}`);
    } else {
      premiumSearchInput.removeAttribute('aria-activedescendant');
    }
  }
}

function openPremiumHistoryDropdown() {
  if (pageState.historySubmitLockUntil > Date.now()) {
    return;
  }
  if (pageState.historyBlurTimer) {
    window.clearTimeout(pageState.historyBlurTimer);
    pageState.historyBlurTimer = 0;
  }
  pageState.historyHighlightedIndex = -1;
  setPremiumHistoryDropdown(true);
  renderPremiumHistoryDropdown();
}

function closePremiumHistoryDropdown() {
  if (pageState.historyBlurTimer) {
    window.clearTimeout(pageState.historyBlurTimer);
  }
  pageState.historyBlurTimer = window.setTimeout(() => {
    pageState.historyHighlightedIndex = -1;
    setPremiumHistoryDropdown(false);
  }, 120);
}

function getPremiumFocusableElements(container) {
  if (!(container instanceof HTMLElement)) {
    return [];
  }
  return Array.from(container.querySelectorAll(
    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
  )).filter((element) =>
    element instanceof HTMLElement
    && !element.hasAttribute('disabled')
    && element.getAttribute('aria-hidden') !== 'true'
    && !element.classList.contains('hidden')
  );
}

function trapPremiumDrawerFocus(event, drawerEl) {
  if (event.key !== 'Tab' || !(drawerEl instanceof HTMLElement)) {
    return;
  }

  const focusableElements = getPremiumFocusableElements(drawerEl);
  if (!focusableElements.length) {
    event.preventDefault();
    drawerEl.focus();
    return;
  }

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];
  const activeElement = document.activeElement;
  const isInsideDrawer = activeElement instanceof HTMLElement && drawerEl.contains(activeElement);

  if (event.shiftKey) {
    if (activeElement === firstElement || activeElement === drawerEl || !isInsideDrawer) {
      event.preventDefault();
      lastElement.focus();
    }
    return;
  }

  if (activeElement === lastElement || !isInsideDrawer) {
    event.preventDefault();
    firstElement.focus();
  }
}

function movePremiumLibraryTabFocus(buttons, currentButton, direction) {
  const tabButtons = Array.from(buttons).filter((button) => button instanceof HTMLButtonElement);
  if (!tabButtons.length || !(currentButton instanceof HTMLButtonElement)) {
    return;
  }
  const currentIndex = tabButtons.indexOf(currentButton);
  const nextIndex = direction === 'next'
    ? (currentIndex + 1 + tabButtons.length) % tabButtons.length
    : (currentIndex - 1 + tabButtons.length) % tabButtons.length;
  tabButtons[nextIndex].focus();
  const nextTab = tabButtons[nextIndex].dataset.premiumLibraryTab || 'history';
  setPremiumLibraryTab(nextTab);
}

function setPremiumLibraryTab(tab) {
  const nextTab = tab === 'favorites' ? 'favorites' : 'history';
  pageState.libraryTab = nextTab;

  const buttons = premiumLibraryToggleEl?.querySelectorAll('[data-premium-library-tab]') || [];
  buttons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.premiumLibraryTab === nextTab);
    button.setAttribute('aria-selected', button.dataset.premiumLibraryTab === nextTab ? 'true' : 'false');
    button.tabIndex = button.dataset.premiumLibraryTab === nextTab ? 0 : -1;
  });

  premiumLibraryPanelEls.forEach((panel) => {
    if (!(panel instanceof HTMLElement)) {
      return;
    }
    const panelTab = panel.dataset.premiumLibraryPanel === 'favorites' ? 'favorites' : 'history';
    const hidden = panelTab !== nextTab;
    panel.classList.toggle('hidden', hidden);
    panel.hidden = hidden;
  });

  if (premiumLibraryStatusEl instanceof HTMLElement) {
    premiumLibraryStatusEl.textContent = nextTab === 'favorites' ? '收藏面板' : '历史面板';
  }
  renderPremiumLibraryCollections();
}

function togglePremiumLibraryDrawer(open) {
  if (!(premiumLibraryOverlayEl instanceof HTMLElement) || !(premiumLibraryDrawerEl instanceof HTMLElement)) {
    return;
  }
  premiumLibraryOverlayEl.classList.toggle('hidden', !open);
  premiumLibraryOverlayEl.setAttribute('aria-hidden', open ? 'false' : 'true');
  openPremiumLibraryDrawerButtons.forEach((button) => {
    if (button instanceof HTMLButtonElement) {
      button.setAttribute('aria-expanded', open ? 'true' : 'false');
    }
  });
  document.body.classList.toggle('drawer-open', open);

  if (open) {
    pageState.drawerLastFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (pageState.drawerTrapHandler) {
      premiumLibraryDrawerEl.removeEventListener('keydown', pageState.drawerTrapHandler);
    }
    pageState.drawerTrapHandler = (event) => trapPremiumDrawerFocus(event, premiumLibraryDrawerEl);
    premiumLibraryDrawerEl.addEventListener('keydown', pageState.drawerTrapHandler);
    window.requestAnimationFrame(() => {
      const focusableElements = getPremiumFocusableElements(premiumLibraryDrawerEl);
      const target = focusableElements[0] || premiumLibraryDrawerEl;
      target.focus();
    });
    return;
  }

  if (pageState.drawerTrapHandler) {
    premiumLibraryDrawerEl.removeEventListener('keydown', pageState.drawerTrapHandler);
    pageState.drawerTrapHandler = null;
  }
  const lastFocused = pageState.drawerLastFocused;
  pageState.drawerLastFocused = null;
  if (lastFocused instanceof HTMLElement) {
    window.requestAnimationFrame(() => {
      lastFocused.focus();
    });
  }
}

function triggerPremiumLibrarySearch(query) {
  const normalized = String(query || '').trim();
  if (!normalized) {
    return;
  }
  if (premiumSearchInput instanceof HTMLInputElement) {
    premiumSearchInput.value = normalized;
  }
  replacePremiumSearchUrl(normalized);
  togglePremiumLibraryDrawer(false);
  runPremiumSearch(normalized).catch((error) => {
    if (premiumSearchProgressEl instanceof HTMLElement) {
      premiumSearchProgressEl.textContent = '搜索失败';
    }
    setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
  });
}

function renderPremiumSearchState() {
  const showSearch = pageState.hasSearched;
  if (premiumSearchSection instanceof HTMLElement) {
    premiumSearchSection.classList.toggle('hidden', !showSearch);
  }
  if (premiumDiscoverySectionsEl instanceof HTMLElement) {
    premiumDiscoverySectionsEl.classList.toggle('hidden', showSearch);
  }
  if (premiumSearchClearButton instanceof HTMLButtonElement) {
    premiumSearchClearButton.disabled = !showSearch;
  }
  if (showSearch) {
    clearPremiumLoadMoreObserver();
  } else {
    setupPremiumLoadMoreObserver();
  }
}

function clearPremiumLoadMoreObserver() {
  if (pageState.loadMoreObserver && typeof pageState.loadMoreObserver.disconnect === 'function') {
    pageState.loadMoreObserver.disconnect();
  }
  pageState.loadMoreObserver = null;
}

function setupPremiumAutoLoadMore() {
  if (!pageState.hasMore || pageState.loading) {
    return;
  }
  pageState.page += 1;
  loadVideos({ append: true }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '自动加载更多失败');
  });
}

function setupPremiumLoadMoreObserver() {
  clearPremiumLoadMoreObserver();
  if (pageState.hasSearched || !(loadMoreButton instanceof HTMLElement) || typeof window.IntersectionObserver !== 'function') {
    return;
  }
  pageState.loadMoreObserver = new IntersectionObserver((entries) => {
    const visible = entries.some((entry) => entry.isIntersecting);
    if (!visible) {
      return;
    }
    setupPremiumAutoLoadMore();
  }, {
    rootMargin: '240px 0px 240px 0px',
    threshold: 0.1,
  });
  pageState.loadMoreObserver.observe(loadMoreButton);
}

function getPremiumSortLatencyValue(video) {
  const liveLatency = getPremiumLatencyValue(video);
  return typeof liveLatency === 'number' ? liveLatency : Number.MAX_SAFE_INTEGER;
}

function sortPremiumVideos(videos, sortBy) {
  const sorted = [...videos];

  switch (sortBy) {
    case 'relevance':
      return sorted.sort((a, b) => Number(b?.relevanceScore || 0) - Number(a?.relevanceScore || 0));
    case 'latency-asc':
      return sorted.sort((a, b) => getPremiumSortLatencyValue(a) - getPremiumSortLatencyValue(b));
    case 'date-desc':
      return sorted.sort((a, b) => Number.parseInt(String(b?.vod_year || '0'), 10) - Number.parseInt(String(a?.vod_year || '0'), 10));
    case 'date-asc':
      return sorted.sort((a, b) => Number.parseInt(String(a?.vod_year || '0'), 10) - Number.parseInt(String(b?.vod_year || '0'), 10));
    case 'rating-desc':
      return sorted.sort((a, b) => Number(b?.vod_score || 0) - Number(a?.vod_score || 0));
    case 'name-asc':
      return sorted.sort((a, b) => String(a?.vod_name || '').localeCompare(String(b?.vod_name || ''), 'zh-CN'));
    case 'name-desc':
      return sorted.sort((a, b) => String(b?.vod_name || '').localeCompare(String(a?.vod_name || ''), 'zh-CN'));
    case 'default':
    default:
      return sorted.sort((a, b) => {
        const scoreGap = Number(b?.relevanceScore || 0) - Number(a?.relevanceScore || 0);
        if (scoreGap !== 0) {
          return scoreGap;
        }
        return getPremiumSortLatencyValue(a) - getPremiumSortLatencyValue(b);
      });
  }
}

function groupPremiumVideos(videos) {
  const groups = new Map();

  videos.forEach((video) => {
    const key = normalizeTitle(video?.vod_name || '');
    if (!key) {
      return;
    }
    if (!groups.has(key)) {
      groups.set(key, []);
    }
    groups.get(key).push(video);
  });

  return Array.from(groups.values()).map((items) => {
    const sortedItems = [...items].sort((a, b) => getPremiumSortLatencyValue(a) - getPremiumSortLatencyValue(b));
    return {
      representative: sortedItems[0],
      videos: sortedItems,
      name: sortedItems[0]?.vod_name || '未知标题',
    };
  });
}

function renderPremiumDisplayToggle() {
  const buttons = premiumSearchDisplayToggleEl?.querySelectorAll('[data-premium-display-mode]') || [];
  buttons.forEach((button) => {
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    button.classList.toggle('active', button.dataset.premiumDisplayMode === pageState.displayMode);
  });
}

function buildPremiumSearchSourceBadges(videos) {
  const sourceMap = new Map();
  videos.forEach((video) => {
    const sourceId = String(video?.source || '').trim();
    if (!sourceId) {
      return;
    }
    const existing = sourceMap.get(sourceId) || {
      id: sourceId,
      name: String(video?.sourceDisplayName || video?.sourceName || sourceId).trim() || sourceId,
      count: 0,
    };
    existing.count += 1;
    sourceMap.set(sourceId, existing);
  });
  return Array.from(sourceMap.values()).sort((left, right) => {
    if (right.count !== left.count) {
      return right.count - left.count;
    }
    return String(left.name || '').localeCompare(String(right.name || ''), 'zh-CN');
  });
}

function buildPremiumSearchTypeBadges(videos) {
  const typeMap = new Map();
  videos.forEach((video) => {
    const typeName = String(video?.type_name || '').trim();
    if (!typeName) {
      return;
    }
    typeMap.set(typeName, (typeMap.get(typeName) || 0) + 1);
  });
  return Array.from(typeMap.entries())
    .map(([type, count]) => ({ type, count }))
    .sort((left, right) => {
      if (right.count !== left.count) {
        return right.count - left.count;
      }
      return String(left.type || '').localeCompare(String(right.type || ''), 'zh-CN');
    });
}

function prunePremiumSearchFilters(availableSources, availableTypes) {
  const availableSourceIds = new Set(availableSources.map((source) => source.id));
  pageState.selectedSources = new Set(
    Array.from(pageState.selectedSources).filter((sourceId) => availableSourceIds.has(sourceId))
  );

  const availableTypeNames = new Set(availableTypes.map((badge) => badge.type));
  pageState.selectedTypes = new Set(
    Array.from(pageState.selectedTypes).filter((type) => availableTypeNames.has(type))
  );
}

function filterPremiumSearchResults(videos) {
  return videos.filter((video) => {
    const sourceMatched = pageState.selectedSources.size === 0
      || (video?.source && pageState.selectedSources.has(video.source));
    const typeMatched = pageState.selectedTypes.size === 0
      || (video?.type_name && pageState.selectedTypes.has(String(video.type_name).trim()));
    return sourceMatched && typeMatched;
  });
}

function renderPremiumSearchFilters(videos) {
  const sourceBadges = buildPremiumSearchSourceBadges(videos);
  const typeBadges = buildPremiumSearchTypeBadges(videos);
  prunePremiumSearchFilters(sourceBadges, typeBadges);

  const hasFilterData = sourceBadges.length > 0 || typeBadges.length > 0;
  premiumSearchFilterToolbarEl?.classList.toggle('hidden', !hasFilterData);

  const sourceFilterCount = pageState.selectedSources.size;
  const typeFilterCount = pageState.selectedTypes.size;
  const filterSummary = [];
  if (sourceFilterCount > 0) {
    filterSummary.push(`来源 ${sourceFilterCount}`);
  }
  if (typeFilterCount > 0) {
    filterSummary.push(`类型 ${typeFilterCount}`);
  }

  if (premiumSearchFilterSummaryEl instanceof HTMLElement) {
    premiumSearchFilterSummaryEl.textContent = filterSummary.length
      ? `已筛选：${filterSummary.join('，')}`
      : '未启用筛选';
  }
  if (clearPremiumSearchFiltersButton instanceof HTMLButtonElement) {
    clearPremiumSearchFiltersButton.disabled = sourceFilterCount === 0 && typeFilterCount === 0;
  }

  if (premiumSearchSourceBadgesEl instanceof HTMLElement) {
    if (!sourceBadges.length) {
      premiumSearchSourceBadgesEl.innerHTML = '<span class="chip">当前结果没有来源标签</span>';
    } else {
      premiumSearchSourceBadgesEl.innerHTML = sourceBadges.map((source) => `
        <button
          class="button button-small ${pageState.selectedSources.has(source.id) ? 'primary' : ''}"
          type="button"
          data-premium-search-source-badge="${escapeHtml(source.id)}"
          aria-pressed="${pageState.selectedSources.has(source.id) ? 'true' : 'false'}"
        >
          ${escapeHtml(source.name)} (${source.count})
        </button>
      `).join('');
    }
  }

  if (premiumSearchTypeBadgesEl instanceof HTMLElement) {
    if (!typeBadges.length) {
      premiumSearchTypeBadgesEl.innerHTML = '<span class="chip">当前结果没有类型标签</span>';
    } else {
      premiumSearchTypeBadgesEl.innerHTML = typeBadges.map((badge) => `
        <button
          class="button button-small ${pageState.selectedTypes.has(badge.type) ? 'primary' : ''}"
          type="button"
          data-premium-search-type-badge="${escapeHtml(badge.type)}"
          aria-pressed="${pageState.selectedTypes.has(badge.type) ? 'true' : 'false'}"
        >
          ${escapeHtml(badge.type)} (${badge.count})
        </button>
      `).join('');
    }
  }
}

function renderPremiumSearchResults() {
  if (!(premiumSearchResultsEl instanceof HTMLElement) || !(premiumSearchSummaryEl instanceof HTMLElement)) {
    return;
  }

  const sortedResults = sortPremiumVideos(pageState.searchResults, pageState.sortBy);
  renderPremiumSearchFilters(sortedResults);
  const filteredResults = filterPremiumSearchResults(sortedResults);
  renderPremiumDisplayToggle();
  if (premiumSearchSortSelectEl instanceof HTMLSelectElement) {
    premiumSearchSortSelectEl.value = pageState.sortBy;
  }

  premiumSearchSummaryEl.textContent = filteredResults.length === pageState.searchResults.length
    ? `结果 ${filteredResults.length}`
    : `结果 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
  if (premiumSearchTotalEl instanceof HTMLElement) {
    premiumSearchTotalEl.textContent = filteredResults.length === pageState.searchResults.length
      ? `结果 ${filteredResults.length}`
      : `结果 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
  }

  if (!pageState.searchResults.length) {
    premiumSearchResultsEl.className = 'results-grid empty-state';
    premiumSearchResultsEl.textContent = pageState.hasSearched
      ? `没有找到「${pageState.searchQuery}」相关 Premium 结果`
      : '请输入关键词开始搜索。';
    return;
  }

  if (!filteredResults.length) {
    premiumSearchResultsEl.className = 'results-grid empty-state';
    premiumSearchResultsEl.textContent = '当前筛选条件下暂无 Premium 结果';
    return;
  }

  if (pageState.displayMode === 'grouped') {
    const grouped = groupPremiumVideos(filteredResults)
      .sort((left, right) => sortPremiumVideos([left.representative, right.representative], pageState.sortBy)[0] === left.representative ? -1 : 1);

    premiumSearchSummaryEl.textContent = `分组 ${grouped.length} / 条目 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
    if (premiumSearchTotalEl instanceof HTMLElement) {
      premiumSearchTotalEl.textContent = `分组 ${grouped.length} / 条目 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
    }
    premiumSearchResultsEl.className = 'results-grid';
    premiumSearchResultsEl.innerHTML = grouped.map((group) => {
      const representative = group.representative || {};
      const poster = representative.vod_pic
        ? `<img class="result-poster" src="${escapeHtml(representative.vod_pic)}" alt="${escapeHtml(group.name || '未知视频')}" referrerpolicy="no-referrer" />`
        : '<div class="result-poster placeholder">🎬</div>';
      const bestLatencyValue = getPremiumLatencyValue(representative);
      const bestLatency = typeof bestLatencyValue === 'number' ? `<span class="chip">${bestLatencyValue} ms</span>` : '';
      const sourceNames = group.videos
        .slice(0, 3)
        .map((video) => escapeHtml(video.sourceDisplayName || video.sourceName || video.source || '未知来源'))
        .join(' / ');
      premiumSearchSummaryEl.textContent = `分组 ${grouped.length} / 条目 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
      if (premiumSearchTotalEl instanceof HTMLElement) {
        premiumSearchTotalEl.textContent = `分组 ${grouped.length} / 条目 ${filteredResults.length} / 总计 ${pageState.searchResults.length}`;
      }
      return `
        <a class="result-card" href="${groupedPremiumDetailUrl(group)}">
          ${poster}
          <div class="stack compact-card-body">
            <strong>${escapeHtml(group.name || '未知标题')}</strong>
            <div class="row wrap gap-sm">
              <span class="chip">${group.videos.length} 个源</span>
              ${bestLatency}
              ${representative.vod_remarks ? `<span class="chip">${escapeHtml(representative.vod_remarks)}</span>` : ''}
            </div>
            <p class="muted">${escapeHtml(sourceNames || '暂无来源信息')}</p>
          </div>
        </a>`;
    }).join('');
    return;
  }

  premiumSearchResultsEl.className = 'results-grid';
  premiumSearchResultsEl.innerHTML = filteredResults.map((video) => {
    const poster = video.vod_pic
      ? `<img class="result-poster" src="${escapeHtml(video.vod_pic)}" alt="${escapeHtml(video.vod_name || '未知视频')}" referrerpolicy="no-referrer" />`
      : '<div class="result-poster placeholder">🎬</div>';
    const latencyValue = getPremiumLatencyValue(video);
    const latency = typeof latencyValue === 'number' ? `<span class="chip">${latencyValue} ms</span>` : '';
    return `
      <a class="result-card" href="${premiumSearchUrl(video)}">
        ${poster}
        <div class="stack compact-card-body">
          <strong>${escapeHtml(video.vod_name || '未知标题')}</strong>
          <div class="row wrap gap-sm">
            <span class="chip">${escapeHtml(video.sourceDisplayName || video.sourceName || video.source || '未知来源')}</span>
            ${latency}
            ${video.vod_remarks ? `<span class="chip">${escapeHtml(video.vod_remarks)}</span>` : ''}
          </div>
          <p class="muted">${escapeHtml(video.type_name || video.vod_year || '暂无分类信息')}</p>
        </div>
      </a>`;
  }).join('');
}

function renderTags() {
  tagCountEl.textContent = `标签 ${pageState.tags.length}`;
  if (premiumTagManageToggleButton instanceof HTMLButtonElement) {
    premiumTagManageToggleButton.textContent = pageState.managingTags ? '完成管理' : '管理标签';
    premiumTagManageToggleButton.setAttribute('aria-pressed', pageState.managingTags ? 'true' : 'false');
  }
  if (restorePremiumTagsButton instanceof HTMLButtonElement) {
    restorePremiumTagsButton.disabled = pageState.tags.length === 0;
  }
  if (exportPremiumTagsButton instanceof HTMLButtonElement) {
    exportPremiumTagsButton.disabled = pageState.tags.length === 0;
    exportPremiumTagsButton.textContent = pageState.tags.length > 0 ? `导出标签 (${pageState.tags.length})` : '导出标签';
  }
  if (sharePremiumTagsButton instanceof HTMLButtonElement) {
    sharePremiumTagsButton.disabled = pageState.tags.length === 0;
    sharePremiumTagsButton.textContent = pageState.tags.length > 0 ? `复制标签分享包 (${pageState.tags.length})` : '复制标签分享包';
  }
  if (sharePremiumTagsLinkButton instanceof HTMLButtonElement) {
    sharePremiumTagsLinkButton.disabled = pageState.tags.length === 0;
    sharePremiumTagsLinkButton.textContent = pageState.tags.length > 0 ? `复制标签分享链接 (${pageState.tags.length})` : '复制标签分享链接';
  }
  if (sharePremiumTagsLinkMergeButton instanceof HTMLButtonElement) {
    sharePremiumTagsLinkMergeButton.disabled = pageState.tags.length === 0;
    sharePremiumTagsLinkMergeButton.textContent = pageState.tags.length > 0 ? `复制标签合并链接 (${pageState.tags.length})` : '复制标签合并链接';
  }

  if (!pageState.tags.length) {
    tagsEl.className = 'tag-cloud empty-state';
    tagsEl.textContent = '当前没有可用标签';
    return;
  }

  if (pageState.managingTags) {
    tagsEl.className = 'saved-list';
    tagsEl.innerHTML = pageState.tags.map((tag, index) => `
      <article class="saved-item source-item">
        <div class="source-item-header">
          <div class="stack compact-card-body">
            <strong>${escapeHtml(tag.label || tag.id)}</strong>
            <div class="source-item-meta">
              <span>ID: ${escapeHtml(tag.id)}</span>
              <span>${tag.id === pageState.selectedTag ? '当前选中标签' : '点击切换顺序或移除'}</span>
            </div>
          </div>
          <span class="chip">第 ${index + 1} 个</span>
        </div>
        <div class="source-actions">
          <button class="button button-small" type="button" data-tag-move="up" data-tag-id="${escapeHtml(tag.id)}" ${index === 0 ? 'disabled' : ''}>上移</button>
          <button class="button button-small" type="button" data-tag-move="down" data-tag-id="${escapeHtml(tag.id)}" ${index === pageState.tags.length - 1 ? 'disabled' : ''}>下移</button>
          <button class="button button-small danger" type="button" data-tag-remove="${escapeHtml(tag.id)}">隐藏</button>
        </div>
      </article>`).join('');
    return;
  }

  tagsEl.className = 'tag-cloud';
  tagsEl.innerHTML = pageState.tags.map((tag) => `
    <button
      class="tag-chip ${tag.id === pageState.selectedTag ? 'active' : ''}"
      type="button"
      data-tag-id="${escapeHtml(tag.id)}"
    >
      ${escapeHtml(tag.label || tag.id)}
    </button>`).join('');
}

function loadSavedPremiumTags() {
  if (typeof window === 'undefined') {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(PREMIUM_TAGS_STORAGE_KEY) || '[]');
    return Array.isArray(parsed)
      ? parsed.filter((item) => item && typeof item === 'object' && typeof item.id === 'string')
      : [];
  } catch (_) {
    return [];
  }
}

function savePremiumTags(tags) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(PREMIUM_TAGS_STORAGE_KEY, JSON.stringify(Array.isArray(tags) ? tags : []));
}

function normalizePremiumImportedTag(tag) {
  if (!tag || typeof tag !== 'object' || typeof tag.id !== 'string') {
    return null;
  }
  const id = String(tag.id || '').trim();
  if (!id) {
    return null;
  }
  return {
    id,
    label: String(tag.label || id).trim() || id,
    value: typeof tag.value === 'string' ? tag.value : '',
  };
}

function buildPremiumTagsExportPayload() {
  return {
    format: 'kvideo-premium-tags-export',
    version: 1,
    exportedAt: new Date().toISOString(),
    selectedTag: pageState.selectedTag,
    tags: pageState.tags.map((tag) => ({
      id: tag.id,
      label: tag.label || tag.id,
      value: typeof tag.value === 'string' ? tag.value : '',
    })),
  };
}

function encodePremiumTagsSharePackage(payload) {
  return `kvideo://premium-tags/${encodeLibrarySharePackage(payload)}`;
}

function decodePremiumTagsSharePackage(rawValue) {
  const normalized = String(rawValue || '').trim();
  const prefixes = [
    'kvideo://premium-tags/',
    'kvideo://premium-tags-share/',
  ];
  const prefix = prefixes.find((item) => normalized.startsWith(item));
  if (!prefix) {
    return null;
  }
  const encoded = normalized.slice(prefix.length);
  if (!encoded) {
    throw new Error('Premium 标签分享包内容为空');
  }
  const padded = encoded.replaceAll('-', '+').replaceAll('_', '/').padEnd(Math.ceil(encoded.length / 4) * 4, '=');
  const binary = atob(padded);
  const percentEncoded = Array.from(binary).map((char) =>
    `%${char.charCodeAt(0).toString(16).padStart(2, '0')}`
  ).join('');
  return JSON.parse(decodeURIComponent(percentEncoded));
}

function parsePremiumTagsImportPayload(rawText) {
  const normalized = String(rawText || '').trim();
  if (!normalized) {
    throw new Error('标签导入内容为空');
  }
  let parsed = null;
  if (normalized.startsWith('kvideo://premium-tags/')) {
    parsed = decodePremiumTagsSharePackage(normalized);
  } else {
    parsed = JSON.parse(normalized);
  }
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.tags)) {
    throw new Error('标签导入内容缺少 tags 列表');
  }
  const tags = parsed.tags
    .map((tag) => normalizePremiumImportedTag(tag))
    .filter(Boolean);
  if (!tags.length) {
    throw new Error('标签导入内容里没有有效标签');
  }
  return {
    selectedTag: typeof parsed.selectedTag === 'string' ? parsed.selectedTag : '',
    tags,
  };
}

function buildPremiumTagsShareUrl(merge = false) {
  const params = new URLSearchParams();
  params.set('premiumTagsShare', encodePremiumTagsSharePackage(buildPremiumTagsExportPayload()));
  if (merge) {
    params.set('premiumTagsShareMode', 'merge');
  }
  return `${window.location.origin}/premium?${params.toString()}`;
}

function readPremiumTagShareParams() {
  const params = new URLSearchParams(window.location.search);
  const rawValue = String(params.get('premiumTagsShare') || '').trim();
  if (!rawValue) {
    return null;
  }
  return {
    rawValue,
    merge: params.get('premiumTagsShareMode') === 'merge',
  };
}

function clearPremiumTagShareParams() {
  const params = new URLSearchParams(window.location.search);
  params.delete('premiumTagsShare');
  params.delete('premiumTagsShareMode');
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
}

function mergePremiumImportedTags(existingTags, importedTags) {
  const nextTags = existingTags.map((tag) => ({ ...tag }));
  const existingIds = new Set(nextTags.map((tag) => tag.id));
  importedTags.forEach((tag) => {
    if (!existingIds.has(tag.id)) {
      nextTags.push({ ...tag });
      existingIds.add(tag.id);
    }
  });
  return nextTags;
}

async function applyPremiumTagsImport(rawText, options = {}) {
  const parsed = parsePremiumTagsImportPayload(rawText);
  const previousSelectedTag = pageState.selectedTag;
  pageState.tags = options.merge
    ? mergePremiumImportedTags(pageState.tags, parsed.tags)
    : parsed.tags.map((tag) => ({ ...tag }));
  const nextSelectedTag = pageState.tags.find((tag) => tag.id === previousSelectedTag)
    ? previousSelectedTag
    : (pageState.tags.find((tag) => tag.id === parsed.selectedTag)?.id || pageState.tags[0]?.id || 'recommend');
  pageState.selectedTag = nextSelectedTag;
  savePremiumTags(pageState.tags);
  renderTags();
  if (previousSelectedTag !== nextSelectedTag) {
    pageState.page = 1;
    pageState.videos = [];
    pageState.hasMore = true;
    await loadVideos({ append: false });
  }
  return parsed;
}

async function importPremiumTags(file, options = {}) {
  const rawText = await file.text();
  return applyPremiumTagsImport(rawText, options);
}

function exportPremiumTags() {
  if (!pageState.tags.length) {
    throw new Error('当前没有可导出的 Premium 标签');
  }
  const payload = buildPremiumTagsExportPayload();
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-premium-tags-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

async function copyPremiumTagsSharePackage() {
  if (!pageState.tags.length) {
    throw new Error('当前没有可分享的 Premium 标签');
  }
  await copyText(encodePremiumTagsSharePackage(buildPremiumTagsExportPayload()));
}

async function copyPremiumTagsShareLink(merge = false) {
  if (!pageState.tags.length) {
    throw new Error('当前没有可分享的 Premium 标签');
  }
  await copyText(buildPremiumTagsShareUrl(merge));
}

async function applyPremiumTagShareFromUrl() {
  const shareParams = readPremiumTagShareParams();
  if (!shareParams) {
    return;
  }
  try {
    await applyPremiumTagsImport(shareParams.rawValue, { merge: shareParams.merge });
    setStatus('success', shareParams.merge ? '已从分享链接合并导入 Premium 标签' : '已从分享链接导入 Premium 标签');
  } finally {
    clearPremiumTagShareParams();
  }
}

function mergePremiumTags(apiTags) {
  const savedTags = loadSavedPremiumTags();
  if (!savedTags.length) {
    return Array.isArray(apiTags) ? apiTags : [];
  }

  const apiTagMap = new Map();
  (Array.isArray(apiTags) ? apiTags : []).forEach((tag) => {
    if (tag && typeof tag.id === 'string') {
      apiTagMap.set(tag.id, tag);
    }
  });

  const mergedTags = [];
  const processedIds = new Set();

  savedTags.forEach((savedTag) => {
    if (apiTagMap.has(savedTag.id)) {
      mergedTags.push(apiTagMap.get(savedTag.id));
      processedIds.add(savedTag.id);
    }
  });

  apiTagMap.forEach((tag, id) => {
    if (!processedIds.has(id)) {
      mergedTags.push(tag);
    }
  });

  return mergedTags;
}

function movePremiumTag(tagId, direction) {
  const currentIndex = pageState.tags.findIndex((tag) => tag.id === tagId);
  if (currentIndex < 0) {
    return;
  }
  const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
  if (targetIndex < 0 || targetIndex >= pageState.tags.length) {
    return;
  }
  const nextTags = pageState.tags.slice();
  [nextTags[currentIndex], nextTags[targetIndex]] = [nextTags[targetIndex], nextTags[currentIndex]];
  pageState.tags = nextTags;
  savePremiumTags(pageState.tags);
  renderTags();
  setStatus('success', `已更新标签顺序：${pageState.tags[targetIndex]?.label || tagId}`);
}

function removePremiumTag(tagId) {
  const nextTags = pageState.tags.filter((tag) => tag.id !== tagId);
  if (nextTags.length === pageState.tags.length) {
    return;
  }
  pageState.tags = nextTags;
  if (!pageState.tags.find((tag) => tag.id === pageState.selectedTag)) {
    pageState.selectedTag = pageState.tags[0]?.id || 'recommend';
    void loadVideos({ append: false }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '刷新 Premium 内容失败');
    });
  }
  savePremiumTags(pageState.tags);
  renderTags();
  setStatus('success', `已隐藏标签 ${tagId}`);
}

async function restorePremiumTags() {
  if (typeof window !== 'undefined') {
    window.localStorage.removeItem(PREMIUM_TAGS_STORAGE_KEY);
  }
  pageState.selectedTag = 'recommend';
  await loadTags();
  pageState.page = 1;
  pageState.videos = [];
  pageState.hasMore = true;
  await loadVideos({ append: false });
  setStatus('success', '已恢复默认 Premium 标签');
}

function renderVideos() {
  pageIndicatorEl.textContent = `第 ${pageState.page} 页`;
  if (loadMoreButton instanceof HTMLButtonElement) {
    loadMoreButton.disabled = pageState.loading || !pageState.hasMore;
    loadMoreButton.textContent = pageState.hasMore ? '加载更多' : '已全部加载';
  }

  if (!pageState.videos.length) {
    resultsEl.className = 'results-grid empty-state';
    resultsEl.textContent = '暂无 Premium 内容';
    setupPremiumLoadMoreObserver();
    return;
  }

  resultsEl.className = 'results-grid';
  resultsEl.innerHTML = pageState.videos.map((video) => {
    const poster = video.vod_pic
      ? `<img class="result-poster" src="${escapeHtml(video.vod_pic)}" alt="${escapeHtml(video.vod_name || '未知视频')}" referrerpolicy="no-referrer" />`
      : '<div class="result-poster placeholder">🎬</div>';
    return `
      <a class="result-card" href="${premiumDetailUrl(video)}">
        ${poster}
        <div class="stack compact-card-body">
          <strong>${escapeHtml(video.vod_name || '未知标题')}</strong>
          <div class="row wrap gap-sm">
            <span class="chip">${escapeHtml(video.source || '未知来源')}</span>
            ${video.vod_remarks ? `<span class="chip">${escapeHtml(video.vod_remarks)}</span>` : ''}
          </div>
          <p class="muted">${escapeHtml(video.type_name || '暂无分类信息')}</p>
        </div>
      </a>`;
  }).join('');
  setupPremiumLoadMoreObserver();
}

async function loadTags() {
  const sources = getEnabledPremiumSources();
  if (!sources.length) {
    pageState.tags = [];
    renderTags();
    setStatus('error', '当前没有启用的 Premium 源');
    return;
  }

  const response = await fetch('/api/premium/types', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sources }),
  });
  const data = await response.json().catch(() => ({}));

  if (!response.ok) {
    setStatus('error', data.error || '加载 Premium 标签失败');
    pageState.tags = [];
    renderTags();
    return;
  }

  pageState.tags = mergePremiumTags(Array.isArray(data.tags) ? data.tags : []);
  savePremiumTags(pageState.tags);
  if (!pageState.tags.find((tag) => tag.id === pageState.selectedTag)) {
    pageState.selectedTag = pageState.tags[0]?.id || 'recommend';
  }
  renderTags();
}

async function runPremiumSearch(query) {
  const normalizedQuery = String(query || '').trim();
  const sources = getEnabledPremiumSources();

  stopPremiumLatencyPolling();
  pageState.liveLatencies = {};
  pageState.selectedSources = new Set();
  pageState.selectedTypes = new Set();
  pageState.sourceBaseUrls = Object.fromEntries(
    sources
      .filter((source) => typeof source?.id === 'string' && typeof source?.baseUrl === 'string')
      .map((source) => [source.id, source.baseUrl])
  );
  pageState.searchQuery = normalizedQuery;
  pageState.hasSearched = Boolean(normalizedQuery);
  pageState.searchResults = [];
  renderPremiumSearchState();
  renderPremiumSearchResults();

  if (premiumSearchProgressEl instanceof HTMLElement) {
    premiumSearchProgressEl.textContent = pageState.hasSearched ? '搜索中...' : '等待搜索';
  }

  if (!normalizedQuery) {
    stopPremiumLatencyPolling();
    setStatus('muted', '已返回 Premium 内容流');
    return;
  }

  if (!sources.length) {
    if (premiumSearchProgressEl instanceof HTMLElement) {
      premiumSearchProgressEl.textContent = '搜索失败';
    }
    if (premiumSearchResultsEl instanceof HTMLElement) {
      premiumSearchResultsEl.className = 'results-grid empty-state';
      premiumSearchResultsEl.textContent = '当前没有启用的 Premium 搜索源';
    }
    stopPremiumLatencyPolling();
    setStatus('error', '当前没有启用的 Premium 搜索源');
    return;
  }

  if (premiumSearchTotalEl instanceof HTMLElement) {
    premiumSearchTotalEl.textContent = '结果 0';
  }
  if (premiumSearchResultsEl instanceof HTMLElement) {
    premiumSearchResultsEl.className = 'results-grid empty-state';
    premiumSearchResultsEl.textContent = '正在等待 Premium 搜索结果...';
  }
  setStatus('muted', `正在搜索「${normalizedQuery}」...`);

  const response = await fetch('/api/search-parallel', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query: normalizedQuery, sources, page: 1 }),
  });

  if (!response.ok || !response.body) {
    const data = await response.json().catch(() => ({}));
    if (premiumSearchProgressEl instanceof HTMLElement) {
      premiumSearchProgressEl.textContent = '搜索失败';
    }
    setStatus('error', data.error || 'Premium 搜索失败');
    return;
  }

  const decoder = new TextDecoder();
  const reader = response.body.getReader();
  let buffer = '';
  let totalSources = 0;

  while (true) {
    const { value, done } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (!line.startsWith('data: ')) {
        continue;
      }

      try {
        const payload = JSON.parse(line.slice(6));
        if (payload.type === 'start') {
          totalSources = payload.totalSources || 0;
          if (premiumSearchProgressEl instanceof HTMLElement) {
            premiumSearchProgressEl.textContent = `已启动 ${totalSources} 条线路`;
          }
        } else if (payload.type === 'videos') {
          pageState.searchResults.push(...(payload.videos || []));
          renderPremiumSearchResults();
          ensurePremiumLatencyPolling();
        } else if (payload.type === 'progress') {
          if (premiumSearchProgressEl instanceof HTMLElement) {
            premiumSearchProgressEl.textContent = `进度 ${payload.completedSources || 0}/${totalSources || payload.totalSources || 0}`;
          }
          setStatus('muted', `已收到 ${pageState.searchResults.length} 条 Premium 结果`);
        } else if (payload.type === 'complete') {
          if (premiumSearchProgressEl instanceof HTMLElement) {
            premiumSearchProgressEl.textContent = '搜索完成';
          }
          ensurePremiumLatencyPolling();
          setStatus('success', `Premium 搜索完成，共 ${pageState.searchResults.length} 条结果`);
        } else if (payload.type === 'error') {
          if (premiumSearchProgressEl instanceof HTMLElement) {
            premiumSearchProgressEl.textContent = '搜索失败';
          }
          stopPremiumLatencyPolling();
          setStatus('error', payload.message || 'Premium 搜索失败');
        }
      } catch (error) {
        if (premiumSearchProgressEl instanceof HTMLElement) {
          premiumSearchProgressEl.textContent = '搜索失败';
        }
        stopPremiumLatencyPolling();
        setStatus('error', error instanceof Error ? error.message : '解析 Premium 搜索结果失败');
      }
    }
  }

  renderPremiumSearchResults();
  if (!pageState.searchResults.length) {
    stopPremiumLatencyPolling();
  }
  updatePremiumSearchHistory(normalizedQuery, pageState.searchResults.length);
}

async function loadVideos({ append = false } = {}) {
  const sources = getEnabledPremiumSources();
  if (!sources.length) {
    pageState.videos = [];
    pageState.hasMore = false;
    renderVideos();
    setStatus('error', '当前没有启用的 Premium 源');
    return;
  }

  pageState.loading = true;
  loadMoreButton.disabled = true;
  setStatus('muted', `正在加载「${pageState.selectedTag}」内容...`);

  const response = await fetch('/api/premium/category', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      sources,
      category: currentCategoryValue(),
      page: String(pageState.page),
      limit: '20',
    }),
  });
  const data = await response.json().catch(() => ({}));
  pageState.loading = false;
  loadMoreButton.disabled = false;

  if (!response.ok) {
    setStatus('error', data.error || '加载 Premium 内容失败');
    if (!append) {
      pageState.videos = [];
      renderVideos();
    }
    return;
  }

  const videos = Array.isArray(data.videos) ? data.videos : [];
  pageState.videos = append ? [...pageState.videos, ...videos] : videos;
  pageState.hasMore = videos.length === 20;
  renderVideos();
  setStatus('success', `已加载 ${pageState.videos.length} 条 Premium 内容`);
}

async function refreshPage() {
  pageState.page = 1;
  pageState.videos = [];
  pageState.hasMore = true;
  await loadTags();
  await loadVideos({ append: false });
}

tagsEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const moveButton = target.closest('[data-tag-move]');
  if (moveButton instanceof HTMLButtonElement) {
    movePremiumTag(
      String(moveButton.dataset.tagId || ''),
      moveButton.dataset.tagMove === 'up' ? 'up' : 'down'
    );
    return;
  }
  const removeButton = target.closest('[data-tag-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    removePremiumTag(String(removeButton.dataset.tagRemove || ''));
    return;
  }
  const button = target.closest('[data-tag-id]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const tagId = button.dataset.tagId;
  if (!tagId || tagId === pageState.selectedTag) {
    return;
  }
  pageState.selectedTag = tagId;
  pageState.page = 1;
  pageState.videos = [];
  pageState.hasMore = true;
  renderTags();
  loadVideos({ append: false }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '加载 Premium 内容失败');
  });
});

premiumTagManageToggleButton?.addEventListener('click', () => {
  pageState.managingTags = !pageState.managingTags;
  renderTags();
});

restorePremiumTagsButton?.addEventListener('click', () => {
  restorePremiumTags().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '恢复默认 Premium 标签失败');
  });
});

exportPremiumTagsButton?.addEventListener('click', () => {
  try {
    exportPremiumTags();
    setStatus('success', '已导出 Premium 标签配置');
  } catch (error) {
    setStatus('error', error instanceof Error ? error.message : '导出 Premium 标签失败');
  }
});

importPremiumTagsTriggerButton?.addEventListener('click', () => {
  if (!(premiumTagsImportFileInput instanceof HTMLInputElement)) {
    setStatus('error', '当前页面不可用 Premium 标签导入功能');
    return;
  }
  premiumTagsImportFileInput.dataset.importMode = 'replace';
  premiumTagsImportFileInput.value = '';
  premiumTagsImportFileInput.click();
});

importPremiumTagsMergeTriggerButton?.addEventListener('click', () => {
  if (!(premiumTagsImportFileInput instanceof HTMLInputElement)) {
    setStatus('error', '当前页面不可用 Premium 标签合并导入功能');
    return;
  }
  premiumTagsImportFileInput.dataset.importMode = 'merge';
  premiumTagsImportFileInput.value = '';
  premiumTagsImportFileInput.click();
});

sharePremiumTagsButton?.addEventListener('click', () => {
  copyPremiumTagsSharePackage().then(() => {
    setStatus('success', '已复制 Premium 标签分享包');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制 Premium 标签分享包失败');
  });
});

sharePremiumTagsLinkButton?.addEventListener('click', () => {
  copyPremiumTagsShareLink(false).then(() => {
    setStatus('success', '已复制 Premium 标签分享链接');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制 Premium 标签分享链接失败');
  });
});

sharePremiumTagsLinkMergeButton?.addEventListener('click', () => {
  copyPremiumTagsShareLink(true).then(() => {
    setStatus('success', '已复制 Premium 标签合并链接');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '复制 Premium 标签合并链接失败');
  });
});

refreshButton?.addEventListener('click', () => {
  if (pageState.hasSearched && pageState.searchQuery) {
    runPremiumSearch(pageState.searchQuery).catch((error) => {
      if (premiumSearchProgressEl instanceof HTMLElement) {
        premiumSearchProgressEl.textContent = '搜索失败';
      }
      setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
    });
    return;
  }
  refreshPage().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '刷新 Premium 内容失败');
  });
});

premiumSearchDisplayToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-premium-display-mode]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const nextMode = button.dataset.premiumDisplayMode;
  if (nextMode !== 'normal' && nextMode !== 'grouped') {
    return;
  }
  pageState.displayMode = nextMode;
  renderPremiumSearchResults();
});

premiumSearchSortSelectEl?.addEventListener('change', () => {
  if (!(premiumSearchSortSelectEl instanceof HTMLSelectElement)) {
    return;
  }
  pageState.sortBy = premiumSearchSortSelectEl.value || 'default';
  renderPremiumSearchResults();
});

loadMoreButton?.addEventListener('click', () => {
  if (!pageState.hasMore || pageState.loading) {
    return;
  }
  pageState.page += 1;
  loadVideos({ append: true }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '加载更多失败');
  });
});

premiumSearchForm?.addEventListener('submit', (event) => {
  event.preventDefault();
  if (!(premiumSearchInput instanceof HTMLInputElement)) {
    return;
  }
  const query = premiumSearchInput.value.trim();
  pageState.historySubmitLockUntil = Date.now() + 800;
  setPremiumHistoryDropdown(false);
  premiumSearchInput.blur();
  const params = new URLSearchParams(window.location.search);
  if (query) {
    params.set('q', query);
  } else {
    params.delete('q');
  }
  const nextUrl = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', nextUrl);
  runPremiumSearch(query).catch((error) => {
    if (premiumSearchProgressEl instanceof HTMLElement) {
      premiumSearchProgressEl.textContent = '搜索失败';
    }
    setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
  });
});

premiumSearchClearButton?.addEventListener('click', () => {
  stopPremiumLatencyPolling();
  pageState.liveLatencies = {};
  pageState.searchQuery = '';
  pageState.hasSearched = false;
  pageState.searchResults = [];
  pageState.selectedSources = new Set();
  pageState.selectedTypes = new Set();
  if (premiumSearchInput instanceof HTMLInputElement) {
    premiumSearchInput.value = '';
  }
  if (premiumSearchProgressEl instanceof HTMLElement) {
    premiumSearchProgressEl.textContent = '等待搜索';
  }
  if (premiumSearchTotalEl instanceof HTMLElement) {
    premiumSearchTotalEl.textContent = '结果 0';
  }
  renderPremiumSearchState();
  renderPremiumSearchResults();
  window.history.replaceState({}, '', window.location.pathname);
  setStatus('muted', '已返回 Premium 内容流');
});

clearPremiumSearchFiltersButton?.addEventListener('click', () => {
  pageState.selectedSources = new Set();
  pageState.selectedTypes = new Set();
  renderPremiumSearchResults();
});

premiumSearchSourceBadgesEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-premium-search-source-badge]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const sourceId = String(button.dataset.premiumSearchSourceBadge || '').trim();
  if (!sourceId) {
    return;
  }
  if (pageState.selectedSources.has(sourceId)) {
    pageState.selectedSources.delete(sourceId);
  } else {
    pageState.selectedSources.add(sourceId);
  }
  renderPremiumSearchResults();
});

premiumSearchTypeBadgesEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-premium-search-type-badge]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  const type = String(button.dataset.premiumSearchTypeBadge || '').trim();
  if (!type) {
    return;
  }
  if (pageState.selectedTypes.has(type)) {
    pageState.selectedTypes.delete(type);
  } else {
    pageState.selectedTypes.add(type);
  }
  renderPremiumSearchResults();
});

premiumHistoryClearButton?.addEventListener('click', () => {
  clearPremiumSearchHistory();
  setStatus('success', '已清空 Premium 搜索历史');
});

premiumHistoryListEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const removeButton = target.closest('[data-premium-history-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    removePremiumSearchHistory(String(removeButton.dataset.premiumHistoryRemove || ''));
    setStatus('success', '已移除 Premium 搜索历史');
    return;
  }

  const queryButton = target.closest('[data-premium-history-query]');
  if (queryButton instanceof HTMLButtonElement) {
    const query = String(queryButton.dataset.premiumHistoryQuery || '').trim();
    if (!query) {
      return;
    }
    if (premiumSearchInput instanceof HTMLInputElement) {
      premiumSearchInput.value = query;
    }
    const params = new URLSearchParams(window.location.search);
    params.set('q', query);
    window.history.replaceState({}, '', `${window.location.pathname}?${params.toString()}`);
    togglePremiumLibraryDrawer(false);
    runPremiumSearch(query).catch((error) => {
      if (premiumSearchProgressEl instanceof HTMLElement) {
        premiumSearchProgressEl.textContent = '搜索失败';
      }
      setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
    });
  }
});

premiumSearchInput?.addEventListener('focus', () => {
  if (!getPremiumSearchHistoryEnabled()) {
    return;
  }
  openPremiumHistoryDropdown();
});

premiumSearchInput?.addEventListener('input', () => {
  pageState.historyHighlightedIndex = -1;
  if (pageState.historyDropdownOpen) {
    renderPremiumHistoryDropdown();
  }
});

premiumSearchInput?.addEventListener('blur', () => {
  closePremiumHistoryDropdown();
});

premiumSearchInput?.addEventListener('keydown', (event) => {
  const history = filterPremiumSearchHistoryItems(premiumSearchInput?.value || '');
  if (!history.length || !pageState.historyDropdownOpen) {
    return;
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    pageState.historyHighlightedIndex = pageState.historyHighlightedIndex < history.length - 1
      ? pageState.historyHighlightedIndex + 1
      : 0;
    renderPremiumHistoryDropdown();
    return;
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault();
    pageState.historyHighlightedIndex = pageState.historyHighlightedIndex > 0
      ? pageState.historyHighlightedIndex - 1
      : history.length - 1;
    renderPremiumHistoryDropdown();
    return;
  }

  if (event.key === 'Enter' && pageState.historyHighlightedIndex >= 0) {
    const item = history[pageState.historyHighlightedIndex];
    if (item?.query) {
      event.preventDefault();
      premiumSearchInput.value = item.query;
      replacePremiumSearchUrl(item.query);
      pageState.historySubmitLockUntil = Date.now() + 800;
      setPremiumHistoryDropdown(false);
      premiumSearchInput.blur();
      runPremiumSearch(item.query).catch((error) => {
        if (premiumSearchProgressEl instanceof HTMLElement) {
          premiumSearchProgressEl.textContent = '搜索失败';
        }
        setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
      });
    }
    return;
  }

  if (event.key === 'Escape') {
    setPremiumHistoryDropdown(false);
    premiumSearchInput.blur();
  }
});

premiumSearchHistoryDropdownEl?.addEventListener('mousedown', (event) => {
  event.preventDefault();
});

premiumSearchHistoryDropdownEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const clearAllButton = target.closest('[data-premium-history-clear-all]');
  if (clearAllButton instanceof HTMLButtonElement) {
    clearPremiumSearchHistory();
    setStatus('success', '已清空 Premium 搜索历史');
    return;
  }

  const removeButton = target.closest('[data-premium-history-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    removePremiumSearchHistory(String(removeButton.dataset.premiumHistoryRemove || ''));
    setStatus('success', '已移除 Premium 搜索历史');
    return;
  }

  const queryButton = target.closest('[data-premium-history-query]');
  if (queryButton instanceof HTMLButtonElement) {
    const query = String(queryButton.dataset.premiumHistoryQuery || '').trim();
    if (!query) {
      return;
    }
    premiumSearchInput.value = query;
    replacePremiumSearchUrl(query);
    pageState.historySubmitLockUntil = Date.now() + 800;
    setPremiumHistoryDropdown(false);
    premiumSearchInput.blur();
    runPremiumSearch(query).catch((error) => {
      if (premiumSearchProgressEl instanceof HTMLElement) {
        premiumSearchProgressEl.textContent = '搜索失败';
      }
      setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
    });
  }
});

premiumLibraryToggleEl?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const button = target.closest('[data-premium-library-tab]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }
  setPremiumLibraryTab(button.dataset.premiumLibraryTab || 'history');
});

premiumLibraryToggleEl?.addEventListener('keydown', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLButtonElement) || target.dataset.premiumLibraryTab == null) {
    return;
  }
  const buttons = premiumLibraryToggleEl.querySelectorAll('[data-premium-library-tab]');
  if (event.key === 'ArrowRight') {
    event.preventDefault();
    movePremiumLibraryTabFocus(buttons, target, 'next');
    return;
  }
  if (event.key === 'ArrowLeft') {
    event.preventDefault();
    movePremiumLibraryTabFocus(buttons, target, 'prev');
    return;
  }
  if (event.key === 'Home') {
    event.preventDefault();
    const firstButton = buttons[0];
    if (firstButton instanceof HTMLButtonElement) {
      firstButton.focus();
      setPremiumLibraryTab(firstButton.dataset.premiumLibraryTab || 'history');
    }
    return;
  }
  if (event.key === 'End') {
    event.preventDefault();
    const lastButton = buttons[buttons.length - 1];
    if (lastButton instanceof HTMLButtonElement) {
      lastButton.focus();
      setPremiumLibraryTab(lastButton.dataset.premiumLibraryTab || 'favorites');
    }
  }
});

openPremiumLibraryDrawerButtons.forEach((button) => {
  button.addEventListener('click', () => {
    togglePremiumLibraryDrawer(true);
  });
});

closePremiumLibraryDrawerButton?.addEventListener('click', () => {
  togglePremiumLibraryDrawer(false);
});

premiumLibraryOverlayEl?.addEventListener('click', (event) => {
  if (event.target === premiumLibraryOverlayEl) {
    togglePremiumLibraryDrawer(false);
  }
});

premiumLibraryFilterInputs.forEach((input) => {
  input.addEventListener('input', () => {
    if (input instanceof HTMLInputElement) {
      updatePremiumLibraryFilterValue(input.value);
    }
  });
});

premiumLibrarySortInputs.forEach((input) => {
  input.addEventListener('change', () => {
    if (input instanceof HTMLSelectElement) {
      updatePremiumLibrarySortValue(input.value);
    }
  });
});

clearPremiumLibraryFilterButtons.forEach((button) => {
  button.addEventListener('click', () => {
    updatePremiumLibraryFilterValue('');
  });
});

document.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const clearButton = target.closest('[data-premium-library-clear]');
  if (clearButton instanceof HTMLButtonElement) {
    clearPremiumLibraryItems(pageState.libraryTab).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '清空 Premium 列表失败');
    });
    return;
  }

  const toggleSelectionButton = target.closest('[data-premium-library-selection-toggle]');
  if (toggleSelectionButton instanceof HTMLButtonElement) {
    setPremiumLibrarySelectionMode(!pageState.librarySelectionMode);
    return;
  }

  const selectAllButton = target.closest('[data-premium-library-select-all]');
  if (selectAllButton instanceof HTMLButtonElement) {
    selectAllPremiumLibraryItems(pageState.libraryTab);
    return;
  }

  const removeSelectedButton = target.closest('[data-premium-library-remove-selected]');
  if (removeSelectedButton instanceof HTMLButtonElement) {
    removeSelectedPremiumLibraryItems(pageState.libraryTab).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '批量移除 Premium 条目失败');
    });
    return;
  }

  const undoButton = target.closest('[data-premium-library-undo]');
  if (undoButton instanceof HTMLButtonElement) {
    undoPremiumLibraryRemoval().catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '撤销 Premium 移除失败');
    });
    return;
  }

  const copyButton = target.closest('[data-premium-library-copy]');
  if (copyButton instanceof HTMLButtonElement) {
    copyPremiumLibraryItems(pageState.libraryTab).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}列表`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 列表失败');
    });
    return;
  }

  const shareButton = target.closest('[data-premium-library-share]');
  if (shareButton instanceof HTMLButtonElement) {
    copyPremiumLibrarySharePackage(pageState.libraryTab).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}分享包`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 分享包失败');
    });
    return;
  }

  const shareLinkButton = target.closest('[data-premium-library-share-link]');
  if (shareLinkButton instanceof HTMLButtonElement) {
    copyPremiumLibraryShareLink(pageState.libraryTab).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}分享链接`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 分享链接失败');
    });
    return;
  }

  const shareLinkMergeButton = target.closest('[data-premium-library-share-link-merge]');
  if (shareLinkMergeButton instanceof HTMLButtonElement) {
    copyPremiumLibraryShareLink(pageState.libraryTab, true).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}合并分享链接`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 合并分享链接失败');
    });
    return;
  }

  const nativeShareButton = target.closest('[data-premium-library-share-native]');
  if (nativeShareButton instanceof HTMLButtonElement) {
    sharePremiumLibraryLink(pageState.libraryTab).then(() => {
      setStatus('success', navigator.share
        ? `已发起当前${getPremiumLibraryLabel(pageState.libraryTab)}系统分享`
        : `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}分享链接`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '分享 Premium 列表失败');
    });
    return;
  }

  const saveSnapshotButton = target.closest('[data-premium-library-snapshot-save]');
  if (saveSnapshotButton instanceof HTMLButtonElement) {
    savePremiumLibrarySnapshot(pageState.libraryTab).then(() => {
      setStatus('success', `已保存${getPremiumLibraryLabel(pageState.libraryTab)}快照`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '保存 Premium 快照失败');
    });
    return;
  }

  const renameSnapshotButton = target.closest('[data-premium-library-snapshot-rename]');
  if (renameSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = renamePremiumLibrarySnapshot(pageState.libraryTab);
      setStatus('success', `已将快照「${snapshot.previousName}」重命名为「${snapshot.name}」`);
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '重命名 Premium 快照失败');
    }
    return;
  }

  const duplicateSnapshotButton = target.closest('[data-premium-library-snapshot-duplicate]');
  if (duplicateSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = duplicatePremiumLibrarySnapshot(pageState.libraryTab);
      setStatus('success', `已克隆快照「${snapshot.name}」`);
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '克隆 Premium 快照失败');
    }
    return;
  }

  const restoreSnapshotButton = target.closest('[data-premium-library-snapshot-restore]');
  if (restoreSnapshotButton instanceof HTMLButtonElement) {
    restorePremiumLibrarySnapshot(pageState.libraryTab).then((snapshot) => {
      setStatus('success', `已恢复快照「${snapshot.name}」，可撤销`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '恢复 Premium 快照失败');
    });
    return;
  }

  const mergeSnapshotButton = target.closest('[data-premium-library-snapshot-merge]');
  if (mergeSnapshotButton instanceof HTMLButtonElement) {
    restorePremiumLibrarySnapshot(pageState.libraryTab, { merge: true }).then((snapshot) => {
      setStatus('success', `已合并快照「${snapshot.name}」，可撤销`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '合并 Premium 快照失败');
    });
    return;
  }

  const deleteSnapshotButton = target.closest('[data-premium-library-snapshot-delete]');
  if (deleteSnapshotButton instanceof HTMLButtonElement) {
    try {
      const snapshot = deletePremiumLibrarySnapshot(pageState.libraryTab);
      if (snapshot) {
        setStatus('success', `已删除快照「${snapshot.name}」`);
      }
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '删除 Premium 快照失败');
    }
    return;
  }

  const exportSnapshotButton = target.closest('[data-premium-library-snapshot-export]');
  if (exportSnapshotButton instanceof HTMLButtonElement) {
    try {
      exportPremiumLibrarySnapshots(pageState.libraryTab);
      setStatus('success', `已导出${getPremiumLibraryLabel(pageState.libraryTab)}快照`);
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '导出 Premium 快照失败');
    }
    return;
  }

  const shareSnapshotButton = target.closest('[data-premium-library-snapshot-share]');
  if (shareSnapshotButton instanceof HTMLButtonElement) {
    copyPremiumLibrarySnapshotSharePackage(pageState.libraryTab).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}快照分享包`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 快照分享包失败');
    });
    return;
  }

  const shareSnapshotLinkButton = target.closest('[data-premium-library-snapshot-share-link]');
  if (shareSnapshotLinkButton instanceof HTMLButtonElement) {
    copyPremiumLibrarySnapshotShareLink(pageState.libraryTab).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}快照分享链接`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 快照分享链接失败');
    });
    return;
  }

  const shareSnapshotMergeLinkButton = target.closest('[data-premium-library-snapshot-share-link-merge]');
  if (shareSnapshotMergeLinkButton instanceof HTMLButtonElement) {
    copyPremiumLibrarySnapshotShareLink(pageState.libraryTab, true).then(() => {
      setStatus('success', `已复制当前${getPremiumLibraryLabel(pageState.libraryTab)}快照合并链接`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '复制 Premium 快照合并链接失败');
    });
    return;
  }

  const importSnapshotButton = target.closest('[data-premium-library-snapshot-import]');
  if (importSnapshotButton instanceof HTMLButtonElement) {
    if (premiumLibrarySnapshotImportFileInput instanceof HTMLInputElement) {
      premiumLibrarySnapshotImportFileInput.dataset.importMode = 'replace';
      premiumLibrarySnapshotImportFileInput.value = '';
      premiumLibrarySnapshotImportFileInput.click();
    } else {
      setStatus('error', '当前页面不可用 Premium 快照导入功能');
    }
    return;
  }

  const mergeImportSnapshotButton = target.closest('[data-premium-library-snapshot-import-merge]');
  if (mergeImportSnapshotButton instanceof HTMLButtonElement) {
    if (premiumLibrarySnapshotImportFileInput instanceof HTMLInputElement) {
      premiumLibrarySnapshotImportFileInput.dataset.importMode = 'merge';
      premiumLibrarySnapshotImportFileInput.value = '';
      premiumLibrarySnapshotImportFileInput.click();
    } else {
      setStatus('error', '当前页面不可用 Premium 快照合并导入功能');
    }
    return;
  }

  const exportButton = target.closest('[data-premium-library-export]');
  if (exportButton instanceof HTMLButtonElement) {
    try {
      exportPremiumLibraryItems(pageState.libraryTab);
      setStatus('success', `已导出当前${getPremiumLibraryLabel(pageState.libraryTab)}列表`);
    } catch (error) {
      setStatus('error', error instanceof Error ? error.message : '导出 Premium 列表失败');
    }
    return;
  }

  const importButton = target.closest('[data-premium-library-import]');
  if (importButton instanceof HTMLButtonElement) {
    if (premiumLibraryImportFileInput instanceof HTMLInputElement) {
      premiumLibraryImportFileInput.dataset.importMode = 'replace';
      premiumLibraryImportFileInput.value = '';
      premiumLibraryImportFileInput.click();
    } else {
      setStatus('error', '当前页面不可用导入功能');
    }
    return;
  }

  const mergeImportButton = target.closest('[data-premium-library-import-merge]');
  if (mergeImportButton instanceof HTMLButtonElement) {
    if (premiumLibraryImportFileInput instanceof HTMLInputElement) {
      premiumLibraryImportFileInput.dataset.importMode = 'merge';
      premiumLibraryImportFileInput.value = '';
      premiumLibraryImportFileInput.click();
    } else {
      setStatus('error', '当前页面不可用合并导入功能');
    }
    return;
  }

  const clipboardImportButton = target.closest('[data-premium-library-import-clipboard]');
  if (clipboardImportButton instanceof HTMLButtonElement) {
    readClipboardImportText(getPremiumLibraryLabel(pageState.libraryTab)).then((rawText) =>
      applyPremiumLibraryImport(pageState.libraryTab, rawText, { merge: false })
    ).then(() => {
      setStatus('success', `已从剪贴板导入当前${getPremiumLibraryLabel(pageState.libraryTab)}列表，可撤销`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '剪贴板导入 Premium 列表失败');
    });
    return;
  }

  const clipboardMergeButton = target.closest('[data-premium-library-import-clipboard-merge]');
  if (clipboardMergeButton instanceof HTMLButtonElement) {
    readClipboardImportText(getPremiumLibraryLabel(pageState.libraryTab)).then((rawText) =>
      applyPremiumLibraryImport(pageState.libraryTab, rawText, { merge: true })
    ).then(() => {
      setStatus('success', `已从剪贴板合并导入当前${getPremiumLibraryLabel(pageState.libraryTab)}列表，可撤销`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '剪贴板合并导入 Premium 列表失败');
    });
    return;
  }

  const dedupeButton = target.closest('[data-premium-library-dedupe]');
  if (dedupeButton instanceof HTMLButtonElement) {
    dedupePremiumLibraryItems(pageState.libraryTab).then(() => {
      setStatus('success', `已完成当前${getPremiumLibraryLabel(pageState.libraryTab)}去重，可撤销`);
    }).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : 'Premium 列表去重失败');
    });
    return;
  }

  const selectItemButton = target.closest('[data-premium-library-select-item]');
  if (selectItemButton instanceof HTMLButtonElement) {
    const [kind, indexText] = String(selectItemButton.dataset.premiumLibrarySelectItem || 'history:-1').split(':');
    togglePremiumLibraryItemSelection(
      kind === 'favorites' ? 'favorites' : 'history',
      Number.parseInt(indexText || '-1', 10)
    );
    return;
  }

  const removeButton = target.closest('[data-premium-library-remove]');
  if (removeButton instanceof HTMLButtonElement) {
    const [kind, indexText] = String(removeButton.dataset.premiumLibraryRemove || 'history:-1').split(':');
    removePremiumLibraryItem(
      kind === 'favorites' ? 'favorites' : 'history',
      Number.parseInt(indexText || '-1', 10)
    ).catch((error) => {
      setStatus('error', error instanceof Error ? error.message : '移除 Premium 条目失败');
    });
    return;
  }

  const queryButton = target.closest('[data-premium-library-query]');
  if (queryButton instanceof HTMLButtonElement) {
    triggerPremiumLibrarySearch(String(queryButton.dataset.premiumLibraryQuery || ''));
  }
});

premiumLibraryImportFileInput?.addEventListener('change', () => {
  if (!(premiumLibraryImportFileInput instanceof HTMLInputElement)) {
    return;
  }
  const [file] = Array.from(premiumLibraryImportFileInput.files || []);
  if (!file) {
    return;
  }
  const merge = premiumLibraryImportFileInput.dataset.importMode === 'merge';
  importPremiumLibraryItems(pageState.libraryTab, file, { merge }).then(() => {
    setStatus('success', merge
      ? `已合并导入当前${getPremiumLibraryLabel(pageState.libraryTab)}列表，可撤销`
      : `已导入当前${getPremiumLibraryLabel(pageState.libraryTab)}列表，可撤销`);
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '导入 Premium 列表失败');
  }).finally(() => {
    delete premiumLibraryImportFileInput.dataset.importMode;
    premiumLibraryImportFileInput.value = '';
  });
});

premiumLibrarySnapshotImportFileInput?.addEventListener('change', () => {
  if (!(premiumLibrarySnapshotImportFileInput instanceof HTMLInputElement)) {
    return;
  }
  const [file] = Array.from(premiumLibrarySnapshotImportFileInput.files || []);
  if (!file) {
    return;
  }
  const merge = premiumLibrarySnapshotImportFileInput.dataset.importMode === 'merge';
  importPremiumLibrarySnapshots(pageState.libraryTab, file, { merge }).then(() => {
    setStatus('success', merge
      ? `已合并导入${getPremiumLibraryLabel(pageState.libraryTab)}快照`
      : `已导入${getPremiumLibraryLabel(pageState.libraryTab)}快照`);
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '导入 Premium 快照失败');
  }).finally(() => {
    delete premiumLibrarySnapshotImportFileInput.dataset.importMode;
    premiumLibrarySnapshotImportFileInput.value = '';
  });
});

premiumTagsImportFileInput?.addEventListener('change', () => {
  if (!(premiumTagsImportFileInput instanceof HTMLInputElement)) {
    return;
  }
  const [file] = Array.from(premiumTagsImportFileInput.files || []);
  if (!file) {
    return;
  }
  const merge = premiumTagsImportFileInput.dataset.importMode === 'merge';
  importPremiumTags(file, { merge }).then(() => {
    setStatus('success', merge ? '已合并导入 Premium 标签配置' : '已导入 Premium 标签配置');
  }).catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '导入 Premium 标签失败');
  }).finally(() => {
    delete premiumTagsImportFileInput.dataset.importMode;
    premiumTagsImportFileInput.value = '';
  });
});

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    togglePremiumLibraryDrawer(false);
  }
});

renderPremiumSearchState();
renderPremiumSearchResults();
renderPremiumSearchHistory();
setPremiumLibraryTab('history');
pageState.settingsSignature = getPremiumSettingsSignature(premiumState);
syncPremiumSettings(readStoredPremiumSettings(), { reload: false });

window.addEventListener('kvideo:bootstrap-ready', () => {
  syncPremiumSettings(readStoredPremiumSettings());
});

window.addEventListener('kvideo:storage-updated', (event) => {
  const detail = event && typeof event === 'object' && 'detail' in event ? event.detail : null;
  if (!detail || detail.key !== 'kvideo-settings') {
    return;
  }
  syncPremiumSettings(readStoredPremiumSettings());
});

applyPremiumLibraryShareFromUrl().catch((error) => {
  setStatus('error', error instanceof Error ? error.message : '处理 Premium 分享链接失败');
});

applyPremiumLibrarySnapshotShareFromUrl().catch((error) => {
  setStatus('error', error instanceof Error ? error.message : '处理 Premium 快照分享链接失败');
});

applyPremiumTagShareFromUrl().catch((error) => {
  setStatus('error', error instanceof Error ? error.message : '处理 Premium 标签分享链接失败');
});

window.addEventListener('beforeunload', () => {
  stopPremiumLatencyPolling();
});

if (pageState.hasSearched && pageState.searchQuery) {
  runPremiumSearch(pageState.searchQuery).catch((error) => {
    if (premiumSearchProgressEl instanceof HTMLElement) {
      premiumSearchProgressEl.textContent = '搜索失败';
    }
    setStatus('error', error instanceof Error ? error.message : 'Premium 搜索失败');
  });
} else {
  refreshPage().catch((error) => {
    setStatus('error', error instanceof Error ? error.message : '初始化 Premium 页面失败');
    tagsEl.className = 'tag-cloud empty-state';
    tagsEl.textContent = '无法加载标签';
    resultsEl.className = 'results-grid empty-state';
    resultsEl.textContent = '无法加载 Premium 内容';
  });
}
"#;
