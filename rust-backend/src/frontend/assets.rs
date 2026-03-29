pub(super) const LOGIN_SCRIPT: &str = r#"
const nextPath = JSON.parse(document.getElementById('next-path').textContent || '"/settings"');
const form = document.getElementById('login-form');
const statusEl = document.getElementById('login-status');

form?.addEventListener('submit', async (event) => {
  event.preventDefault();
  statusEl.textContent = '登录中...';

  const payload = {
    username: document.getElementById('username').value.trim(),
    password: document.getElementById('password').value,
  };

  const response = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });

  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    statusEl.textContent = data.error || '登录失败';
    statusEl.className = 'status error';
    return;
  }

  statusEl.textContent = '登录成功，正在跳转...';
  statusEl.className = 'status success';
  window.location.href = nextPath;
});
"#;

pub(super) const SHELL_BOOTSTRAP_SCRIPT: &str = r#"
(() => {
  if (typeof window === 'undefined') {
    return;
  }

  const payloadEl = document.getElementById('bootstrap-payload');
  const overlayEl = document.getElementById('bootstrap-overlay');
  const messageEl = document.getElementById('bootstrap-message');
  const initialPayload = JSON.parse(payloadEl?.textContent || '{}');
  const storageKeys = [
    { key: 'settings', storageKey: 'kvideo-settings' },
    { key: 'history', storageKey: 'kvideo-history-store' },
    { key: 'favorites', storageKey: 'kvideo-favorites-store' },
    { key: 'search-history', storageKey: 'kvideo-search-history' },
    { key: 'premium-search-history', storageKey: 'kvideo-premium-search-history' },
    { key: 'premium-history', storageKey: 'kvideo-premium-history-store' },
    { key: 'premium-favorites', storageKey: 'kvideo-premium-favorites-store' },
    { key: 'premium-tags', storageKey: 'kvideo_premium_custom_tags' },
  ];
  const storageMap = Object.fromEntries(storageKeys.map((item) => [item.storageKey, item.key]));
  const storageDefaults = {
    settings: {},
    history: [],
    favorites: [],
    'search-history': [],
    'premium-search-history': [],
    'premium-history': [],
    'premium-favorites': [],
    'premium-tags': [],
  };
  const syncTimers = {};
  const SUBSCRIPTION_SYNC_COOLDOWN_MS = 5 * 60 * 1000;

  const state = {
    ready: false,
    authenticated: Boolean(initialPayload.authenticated),
    user: initialPayload.user || null,
    disablePremium: Boolean(initialPayload.user?.disablePremium),
    config: null,
    error: null,
  };

  window.__KVIDEO_BOOTSTRAP_STATE__ = state;
  window.__KVIDEO_BOOTSTRAP_PROMISE__ = Promise.resolve(state);

  function setOverlay(message, visible) {
    if (messageEl) {
      messageEl.textContent = message;
    }
    if (overlayEl) {
      overlayEl.classList.toggle('hidden', !visible);
      overlayEl.setAttribute('aria-hidden', visible ? 'false' : 'true');
    }
  }

  function hasContent(value) {
    if (Array.isArray(value)) {
      return value.length > 0;
    }
    return Boolean(value) && typeof value === 'object' && Object.keys(value).length > 0;
  }

  function syncWindowUser(user) {
    if (!user || typeof user !== 'object') {
      delete window.__KVIDEO_USER__;
      window.__KVIDEO_DISABLE_PREMIUM__ = false;
      return;
    }

    window.__KVIDEO_USER__ = user;
    window.__KVIDEO_DISABLE_PREMIUM__ = Boolean(user.disablePremium);
  }

  function normalizeKeywordList(rawValue) {
    if (Array.isArray(rawValue)) {
      return rawValue
        .map((item) => String(item || '').trim())
        .filter(Boolean);
    }

    return String(rawValue || '')
      .split(/[\n,]/)
      .map((item) => item.trim())
      .filter(Boolean);
  }

  function mergeEnvAdKeywords(configData) {
    const envKeywords = normalizeKeywordList(configData?.adKeywords);
    if (!envKeywords.length) {
      return;
    }

    let settings = {};
    try {
      const parsed = JSON.parse(window.localStorage.getItem('kvideo-settings') || '{}');
      settings = parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
    } catch (_) {
    }

    const userKeywords = normalizeKeywordList(settings.adKeywords);
    const merged = [];

    for (const keyword of [...envKeywords, ...userKeywords]) {
      if (!merged.includes(keyword)) {
        merged.push(keyword);
      }
    }

    window.localStorage.setItem('kvideo-settings', JSON.stringify({
      ...settings,
      adKeywords: merged,
    }));
  }

  function readLocalSettings() {
    try {
      const parsed = JSON.parse(window.localStorage.getItem('kvideo-settings') || '{}');
      return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
    } catch (_) {
      return {};
    }
  }

  function normalizeSubscriptionEntry(rawValue, index) {
    if (!rawValue || typeof rawValue !== 'object') {
      return null;
    }

    const name = String(rawValue.name || '').trim();
    let url = String(rawValue.url || '').trim();
    if (!name || !url) {
      return null;
    }

    if (!/^https?:\/\//i.test(url)) {
      url = `https://${url}`;
    }

    return {
      id: String(rawValue.id || `env-sub-${index + 1}`),
      name,
      url,
      lastUpdated: Number(rawValue.lastUpdated || 0),
      autoRefresh: rawValue.autoRefresh !== false,
    };
  }

  function parseEnvSubscriptions(rawValue) {
    const trimmed = String(rawValue || '').trim();
    if (!trimmed) {
      return [];
    }

    try {
      const parsed = JSON.parse(trimmed);
      if (Array.isArray(parsed)) {
        return parsed
          .map((item, index) => normalizeSubscriptionEntry(item, index))
          .filter(Boolean);
      }
    } catch (_) {
    }

    return trimmed
      .split(/[\n,]/)
      .map((item) => item.trim())
      .filter(Boolean)
      .map((url, index) => normalizeSubscriptionEntry({
        id: `env-sub-${index + 1}`,
        name: `系统预设源 ${index + 1}`,
        url,
        autoRefresh: true,
      }, index))
      .filter(Boolean);
  }

  function mergeEnvSubscriptions(configData) {
    const envSubscriptions = parseEnvSubscriptions(configData?.subscriptionSources);
    if (!envSubscriptions.length) {
      return;
    }

    const settings = readLocalSettings();
    const currentSubscriptions = Array.isArray(settings.subscriptions)
      ? settings.subscriptions.filter((item) => item && typeof item === 'object')
      : [];
    const mergedSubscriptions = [...currentSubscriptions];
    let changed = false;

    envSubscriptions.forEach((subscription) => {
      const existingIndex = mergedSubscriptions.findIndex((item) => String(item?.url || '').trim() === subscription.url);
      if (existingIndex >= 0) {
        const current = mergedSubscriptions[existingIndex];
        if (current.name !== subscription.name || current.autoRefresh === false) {
          mergedSubscriptions[existingIndex] = {
            ...current,
            name: subscription.name,
            autoRefresh: true,
          };
          changed = true;
        }
        return;
      }

      mergedSubscriptions.push(subscription);
      changed = true;
    });

    if (!changed) {
      return;
    }

    window.localStorage.setItem('kvideo-settings', JSON.stringify({
      ...settings,
      subscriptions: mergedSubscriptions,
    }));
  }

  function parseImportedSource(rawSource, fallbackPriority) {
    if (!rawSource || typeof rawSource !== 'object') {
      return null;
    }
    if (typeof rawSource.id !== 'string' || typeof rawSource.name !== 'string' || typeof rawSource.baseUrl !== 'string') {
      return null;
    }

    return {
      id: rawSource.id.trim(),
      name: rawSource.name.trim(),
      baseUrl: rawSource.baseUrl.trim(),
      searchPath: typeof rawSource.searchPath === 'string' ? rawSource.searchPath.trim() : '',
      detailPath: typeof rawSource.detailPath === 'string' ? rawSource.detailPath.trim() : '',
      enabled: rawSource.enabled !== false,
      priority: Number.isFinite(Number(rawSource.priority)) ? Math.max(0, Math.floor(Number(rawSource.priority))) : fallbackPriority,
      group: rawSource.group === 'premium' ? 'premium' : 'normal',
    };
  }

  function parseImportedSources(rawValue) {
    const parsed = JSON.parse(String(rawValue || '').trim() || '[]');
    const sourceList = Array.isArray(parsed)
      ? parsed
      : Array.isArray(parsed.sources)
        ? parsed.sources
        : Array.isArray(parsed.list)
          ? parsed.list
          : null;

    if (!sourceList) {
      throw new Error('无法识别的线路 JSON 格式');
    }

    const normalSources = [];
    const premiumSources = [];

    sourceList.forEach((item, index) => {
      const normalized = parseImportedSource(item, index + 1);
      if (!normalized) {
        return;
      }
      if (normalized.group === 'premium') {
        premiumSources.push(normalized);
      } else {
        normalSources.push(normalized);
      }
    });

    return { normalSources, premiumSources };
  }

  function mergeSourceLists(existing, nextSources) {
    const merged = Array.isArray(existing) ? existing.slice() : [];
    const sourceIds = new Set(merged.map((source) => source?.id).filter(Boolean));

    nextSources.forEach((source) => {
      if (sourceIds.has(source.id)) {
        const index = merged.findIndex((item) => item?.id === source.id);
        if (index >= 0) {
          merged[index] = { ...merged[index], ...source };
        }
        return;
      }

      merged.push({
        ...source,
        priority: source.priority || merged.length + 1,
      });
      sourceIds.add(source.id);
    });

    return merged;
  }

  async function fetchSourcesFromSubscriptionUrl(url) {
    const isExternal = /^https?:\/\//i.test(url);
    const fetchUrl = isExternal ? `/api/proxy?url=${encodeURIComponent(url)}` : url;
    const response = await fetch(fetchUrl, {
      headers: {
        Accept: 'application/json',
      },
      credentials: 'same-origin',
    });

    if (!response.ok) {
      throw new Error(`获取订阅失败: ${response.status}`);
    }

    const text = await response.text();
    return parseImportedSources(text);
  }

  async function syncSubscriptionsFromSettings() {
    const settings = readLocalSettings();
    const subscriptions = Array.isArray(settings.subscriptions)
      ? settings.subscriptions.filter((item) => item && typeof item === 'object' && typeof item.url === 'string' && item.url.trim())
      : [];
    if (!subscriptions.length) {
      return;
    }

    const now = Date.now();
    const subscriptionsToSync = subscriptions.filter((subscription) =>
      subscription.autoRefresh !== false
      && !(Number(subscription.lastUpdated || 0) > 0 && now - Number(subscription.lastUpdated || 0) < SUBSCRIPTION_SYNC_COOLDOWN_MS)
    );

    if (!subscriptionsToSync.length) {
      return;
    }

    let nextSources = Array.isArray(settings.sources) ? settings.sources.slice() : [];
    let nextPremiumSources = Array.isArray(settings.premiumSources) ? settings.premiumSources.slice() : [];
    const nextSubscriptions = subscriptions.slice();
    let changed = false;

    const results = await Promise.allSettled(
      subscriptionsToSync.map((subscription) => fetchSourcesFromSubscriptionUrl(subscription.url))
    );

    results.forEach((result, index) => {
      const subscription = subscriptionsToSync[index];
      if (result.status !== 'fulfilled') {
        return;
      }

      const imported = result.value;
      if (Array.isArray(imported.normalSources) && imported.normalSources.length) {
        nextSources = mergeSourceLists(nextSources, imported.normalSources);
        changed = true;
      }
      if (Array.isArray(imported.premiumSources) && imported.premiumSources.length) {
        nextPremiumSources = mergeSourceLists(nextPremiumSources, imported.premiumSources);
        changed = true;
      }

      const subscriptionIndex = nextSubscriptions.findIndex((item) => item.id === subscription.id);
      if (subscriptionIndex >= 0) {
        nextSubscriptions[subscriptionIndex] = {
          ...nextSubscriptions[subscriptionIndex],
          lastUpdated: now,
        };
        changed = true;
      }
    });

    if (!changed) {
      return;
    }

    window.localStorage.setItem('kvideo-settings', JSON.stringify({
      ...settings,
      sources: nextSources,
      premiumSources: nextPremiumSources,
      subscriptions: nextSubscriptions,
    }));
  }

  function scheduleServerSync(storageKey, rawValue) {
    const dataKey = storageMap[storageKey];
    if (!dataKey || !state.authenticated) {
      return;
    }

    if (syncTimers[dataKey]) {
      window.clearTimeout(syncTimers[dataKey]);
    }

    syncTimers[dataKey] = window.setTimeout(async () => {
      try {
        const value = rawValue == null
          ? storageDefaults[dataKey]
          : JSON.parse(rawValue);

        await fetch('/api/user/data', {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'same-origin',
          body: JSON.stringify({ key: dataKey, value }),
        });
      } catch (_) {
      }
    }, 2000);
  }

  function installStorageSync() {
    if (window.__KVIDEO_STORAGE_SYNC_INSTALLED__) {
      return;
    }

    const originalSetItem = window.localStorage.setItem.bind(window.localStorage);
    const originalRemoveItem = window.localStorage.removeItem.bind(window.localStorage);

    window.localStorage.setItem = (key, value) => {
      originalSetItem(key, value);
      scheduleServerSync(key, value);
      window.dispatchEvent(new CustomEvent('kvideo:storage-updated', { detail: { key, value } }));
    };

    window.localStorage.removeItem = (key) => {
      originalRemoveItem(key);
      scheduleServerSync(key, null);
      window.dispatchEvent(new CustomEvent('kvideo:storage-updated', { detail: { key, value: null } }));
    };

    window.__KVIDEO_STORAGE_SYNC_INSTALLED__ = true;
  }

  async function readJson(response) {
    return response.json().catch(() => ({}));
  }

  async function syncUserData() {
    await Promise.all(storageKeys.map(async ({ key, storageKey }) => {
      try {
        const response = await fetch(`/api/user/data?key=${encodeURIComponent(key)}`, {
          credentials: 'same-origin',
        });
        if (!response.ok) {
          return;
        }

        const data = await readJson(response);
        if (hasContent(data.data)) {
          window.localStorage.setItem(storageKey, JSON.stringify(data.data));
        } else {
          window.localStorage.removeItem(storageKey);
        }
      } catch (_) {
      }
    }));
  }

  async function bootstrap() {
    syncWindowUser(state.user);

    if (!state.authenticated) {
      state.ready = true;
      window.dispatchEvent(new CustomEvent('kvideo:bootstrap-ready', { detail: state }));
      return state;
    }

    setOverlay('正在同步账户配置...', true);

    try {
      const configResponse = await fetch('/api/config', {
        credentials: 'same-origin',
      });
      const configData = await readJson(configResponse);
      if (configResponse.ok) {
        state.config = configData;
      }

      await syncUserData();
      state.disablePremium = Boolean(state.user?.disablePremium || state.config?.disablePremium);
      window.__KVIDEO_DISABLE_PREMIUM__ = state.disablePremium;
      installStorageSync();
      mergeEnvAdKeywords(state.config);
      mergeEnvSubscriptions(state.config);
      await syncSubscriptionsFromSettings();
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'bootstrap failed';
    } finally {
      state.ready = true;
      setOverlay(state.error ? '账户配置同步失败，已降级继续使用。' : '账户配置已同步', false);
      window.dispatchEvent(new CustomEvent('kvideo:bootstrap-ready', { detail: state }));
    }

    return state;
  }

  window.__KVIDEO_BOOTSTRAP_PROMISE__ = bootstrap();
})();
"#;

pub(super) const SHELL_APP_SCRIPT: &str = r#"
(() => {
  if (typeof window === 'undefined') {
    return;
  }

  const SETTINGS_KEY = 'kvideo-settings';
  const ACCESS_GATE_SESSION_KEY = 'kvideo-access-unlocked';
  let scrollSaveTimer = 0;
  const backToTopButton = document.getElementById('back-to-top');
  const accessGateOverlay = document.getElementById('access-gate-overlay');
  const accessGateForm = document.getElementById('access-gate-form');
  const accessGatePassword = document.getElementById('access-gate-password');
  const accessGateStatus = document.getElementById('access-gate-status');
  let accessGateLastFocused = null;

  function readSettings() {
    try {
      const parsed = JSON.parse(window.localStorage.getItem(SETTINGS_KEY) || '{}');
      return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
    } catch (_) {
      return {};
    }
  }

  function rememberScrollEnabled() {
    return readSettings().rememberScrollPosition !== false;
  }

  function normalizeAccessPasswords(rawValue) {
    if (!Array.isArray(rawValue)) {
      return [];
    }
    return rawValue
      .map((item) => String(item || '').trim())
      .filter(Boolean);
  }

  function readBootstrapConfig() {
    const config = window.__KVIDEO_BOOTSTRAP_STATE__?.config;
    return config && typeof config === 'object' ? config : {};
  }

  function accessGateFingerprint(settings) {
    return JSON.stringify(normalizeAccessPasswords(settings.accessPasswords));
  }

  function isLocalAccessGateEnabled(settings) {
    const passwords = normalizeAccessPasswords(settings.accessPasswords);
    return Boolean(settings.passwordAccess) && passwords.length > 0;
  }

  function isLocalAccessGateUnlocked(settings) {
    if (!isLocalAccessGateEnabled(settings)) {
      return true;
    }
    return window.sessionStorage.getItem(ACCESS_GATE_SESSION_KEY) === accessGateFingerprint(settings);
  }

  function hasEnvAccessGate() {
    return Boolean(readBootstrapConfig().hasEnvPassword);
  }

  function isEnvAccessGateUnlocked() {
    if (!hasEnvAccessGate()) {
      return true;
    }
    return Boolean(readBootstrapConfig().envPasswordUnlocked);
  }

  function isAccessGateEnabled(settings) {
    return isLocalAccessGateEnabled(settings) || hasEnvAccessGate();
  }

  function isAccessGateUnlocked(settings) {
    return isLocalAccessGateUnlocked(settings) || isEnvAccessGateUnlocked();
  }

  function setAccessGateStatus(kind, message) {
    if (!(accessGateStatus instanceof HTMLElement)) {
      return;
    }
    accessGateStatus.textContent = message;
    accessGateStatus.className = `status ${kind}`;
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

  function trapAccessGateFocus(event) {
    if (event.key !== 'Tab' || !(accessGateForm instanceof HTMLElement)) {
      return;
    }

    const focusableElements = getFocusableElements(accessGateForm);
    if (!focusableElements.length) {
      event.preventDefault();
      accessGateForm.focus();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];
    const activeElement = document.activeElement;
    const isInsideDialog = activeElement instanceof HTMLElement && accessGateForm.contains(activeElement);

    if (event.shiftKey) {
      if (activeElement === firstElement || activeElement === accessGateForm || !isInsideDialog) {
        event.preventDefault();
        lastElement.focus();
      }
      return;
    }

    if (activeElement === lastElement || !isInsideDialog) {
      event.preventDefault();
      firstElement.focus();
    }
  }

  function toggleAccessGate(locked) {
    if (!(accessGateOverlay instanceof HTMLElement)) {
      return;
    }
    accessGateOverlay.classList.toggle('hidden', !locked);
    accessGateOverlay.setAttribute('aria-hidden', locked ? 'false' : 'true');
    document.body.classList.toggle('app-locked', locked);
    if (locked) {
      accessGateLastFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      accessGateForm?.addEventListener('keydown', trapAccessGateFocus);
      window.requestAnimationFrame(() => {
        if (accessGatePassword instanceof HTMLInputElement) {
          accessGatePassword.focus();
          accessGatePassword.select();
        } else if (accessGateForm instanceof HTMLElement) {
          accessGateForm.focus();
        }
      });
      return;
    }

    accessGateForm?.removeEventListener('keydown', trapAccessGateFocus);
    if (accessGatePassword instanceof HTMLInputElement) {
      accessGatePassword.value = '';
    }
    if (accessGateLastFocused instanceof HTMLElement) {
      window.requestAnimationFrame(() => {
        accessGateLastFocused.focus();
      });
    }
  }

  function applyAccessGateState() {
    const settings = readSettings();
    const locked = !isAccessGateUnlocked(settings);
    if (!isAccessGateEnabled(settings)) {
      window.sessionStorage.removeItem(ACCESS_GATE_SESSION_KEY);
      setAccessGateStatus('muted', '请输入访问密码继续使用。');
      toggleAccessGate(false);
      return;
    }

    setAccessGateStatus('muted', locked ? '请输入访问密码继续使用。' : '当前会话已解锁。');
    toggleAccessGate(locked);
  }

  async function unlockEnvAccessGate(password) {
    const response = await fetch('/api/auth/access-unlock', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
      body: JSON.stringify({ password }),
    });
    const data = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(data.error || '访问密码错误，请重试。');
    }

    const config = window.__KVIDEO_BOOTSTRAP_STATE__?.config;
    if (config && typeof config === 'object') {
      config.hasEnvPassword = data.hasEnvPassword ?? true;
      config.persistPassword = data.persistPassword ?? config.persistPassword;
      config.envPasswordUnlocked = Boolean(data.envPasswordUnlocked);
    }
  }

  function getScrollKey() {
    return `scroll-pos:${window.location.pathname}${window.location.search || ''}`;
  }

  function restoreScrollPosition() {
    if (!rememberScrollEnabled()) {
      return;
    }

    const saved = window.sessionStorage.getItem(getScrollKey());
    if (!saved) {
      return;
    }

    const position = Number.parseInt(saved, 10);
    if (!Number.isFinite(position) || position <= 0) {
      return;
    }

    let attempts = 0;
    const maxAttempts = 10;

    const tryScroll = () => {
      window.scrollTo(0, position);
      attempts += 1;

      if (Math.abs(window.scrollY - position) >= 10 && attempts < maxAttempts) {
        window.setTimeout(tryScroll, 200);
      }
    };

    window.setTimeout(tryScroll, 100);
  }

  function saveScrollPosition() {
    if (!rememberScrollEnabled()) {
      return;
    }

    const key = getScrollKey();
    if (window.scrollY > 0) {
      window.sessionStorage.setItem(key, String(window.scrollY));
    } else {
      window.sessionStorage.removeItem(key);
    }
  }

  function installScrollPersistence() {
    window.addEventListener('scroll', () => {
      window.clearTimeout(scrollSaveTimer);
      scrollSaveTimer = window.setTimeout(saveScrollPosition, 500);
      if (backToTopButton instanceof HTMLButtonElement) {
        backToTopButton.classList.toggle('visible', window.scrollY > 300);
      }
    }, { passive: true });

    restoreScrollPosition();
    window.addEventListener('kvideo:bootstrap-ready', restoreScrollPosition);
  }

  function registerServiceWorker() {
    if (!('serviceWorker' in navigator)) {
      return;
    }

    window.addEventListener('load', () => {
      navigator.serviceWorker.register('/sw.js').catch(() => {});
    }, { once: true });
  }

  accessGateForm?.addEventListener('submit', async (event) => {
    event.preventDefault();
    const settings = readSettings();
    const nextPassword = accessGatePassword instanceof HTMLInputElement
      ? accessGatePassword.value.trim()
      : '';

    if (!isAccessGateEnabled(settings)) {
      applyAccessGateState();
      return;
    }

    if (!nextPassword) {
      setAccessGateStatus('error', '请输入访问密码。');
      return;
    }

    const passwords = normalizeAccessPasswords(settings.accessPasswords);
    const localEnabled = isLocalAccessGateEnabled(settings);
    const envEnabled = hasEnvAccessGate();

    if (localEnabled && passwords.includes(nextPassword)) {
      window.sessionStorage.setItem(ACCESS_GATE_SESSION_KEY, accessGateFingerprint(settings));
      setAccessGateStatus('success', '已解锁当前页面。');
      toggleAccessGate(false);
      return;
    }

    if (!envEnabled) {
      setAccessGateStatus('error', '访问密码错误，请重试。');
      if (accessGatePassword instanceof HTMLInputElement) {
        accessGatePassword.select();
      }
      return;
    }

    setAccessGateStatus('muted', '正在验证访问密码...');
    try {
      await unlockEnvAccessGate(nextPassword);
      setAccessGateStatus('success', '已解锁当前页面。');
      toggleAccessGate(false);
    } catch (error) {
      setAccessGateStatus('error', error instanceof Error ? error.message : '访问密码错误，请重试。');
      if (accessGatePassword instanceof HTMLInputElement) {
        accessGatePassword.select();
      }
    }
  });

  window.addEventListener('storage', (event) => {
    if (event.key === SETTINGS_KEY) {
      applyAccessGateState();
    }
  });
  window.addEventListener('kvideo:storage-updated', (event) => {
    if (event?.detail?.key === SETTINGS_KEY) {
      applyAccessGateState();
    }
  });

  if (backToTopButton instanceof HTMLButtonElement) {
    backToTopButton.addEventListener('click', () => {
      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
    backToTopButton.classList.toggle('visible', window.scrollY > 300);
  }

  applyAccessGateState();
  window.addEventListener('kvideo:bootstrap-ready', applyAccessGateState);
  installScrollPersistence();
  registerServiceWorker();
})();
"#;

pub(super) const SETTINGS_SCRIPT: &str = r#"
const settingsArea = document.getElementById('settings-json');
const settingsStatus = document.getElementById('settings-status');
const passwordStatus = document.getElementById('password-status');
const initialSettings = JSON.parse(document.getElementById('initial-settings').textContent || '{}');
const quickSearchDisplayMode = document.getElementById('quick-search-display-mode');
const quickSortBy = document.getElementById('quick-sort-by');
const quickProxyMode = document.getElementById('quick-proxy-mode');
const quickFullscreenType = document.getElementById('quick-fullscreen-type');
const quickSearchHistory = document.getElementById('quick-search-history');
const quickWatchHistory = document.getElementById('quick-watch-history');
const quickRealtimeLatency = document.getElementById('quick-realtime-latency');
const quickRememberScroll = document.getElementById('quick-remember-scroll');
const quickEpisodeReverseOrder = document.getElementById('quick-episode-reverse-order');
const quickAutoNextEpisode = document.getElementById('quick-auto-next-episode');
const quickShowModeIndicator = document.getElementById('quick-show-mode-indicator');
const quickAutoSkipIntro = document.getElementById('quick-auto-skip-intro');
const quickSkipIntroSeconds = document.getElementById('quick-skip-intro-seconds');
const quickAutoSkipOutro = document.getElementById('quick-auto-skip-outro');
const quickSkipOutroSeconds = document.getElementById('quick-skip-outro-seconds');
const quickAdFilterMode = document.getElementById('quick-ad-filter-mode');
const quickAdKeywords = document.getElementById('quick-ad-keywords');
const settingsSourceList = document.getElementById('settings-source-list');
const settingsSourceCount = document.getElementById('settings-source-count');
const settingsPremiumCount = document.getElementById('settings-premium-count');
const settingsSourceSearch = document.getElementById('settings-source-search');
const settingsSourceToggle = document.getElementById('toggle-settings-source-limit');
const subscriptionSummary = document.getElementById('subscription-summary');
const subscriptionSourcesEl = document.getElementById('subscription-sources');
const settingsSubscriptionCount = document.getElementById('settings-subscription-count');
const settingsSubscriptionList = document.getElementById('settings-subscription-list');
const settingsImportPayload = document.getElementById('settings-import-payload');
const accessControlEnabled = document.getElementById('access-control-enabled');
const accessPasswordForm = document.getElementById('access-password-form');
const accessPasswordInput = document.getElementById('access-password-input');
const accessPasswordList = document.getElementById('access-password-list');
const accessPasswordCount = document.getElementById('access-password-count');
const settingsSourceForm = document.getElementById('settings-source-form');
const settingsSourceId = document.getElementById('settings-source-id');
const settingsSourceName = document.getElementById('settings-source-name');
const settingsSourceBaseUrl = document.getElementById('settings-source-base-url');
const settingsSourcePriority = document.getElementById('settings-source-priority');
const settingsSourceSearchPath = document.getElementById('settings-source-search-path');
const settingsSourceDetailPath = document.getElementById('settings-source-detail-path');
const settingsSourceEnabled = document.getElementById('settings-source-enabled');
const settingsSourceSubmit = document.getElementById('settings-source-submit');
const settingsSourceCancel = document.getElementById('settings-source-cancel');
const backupIncludeSearchHistory = document.getElementById('backup-include-search-history');
const backupIncludeWatchHistory = document.getElementById('backup-include-watch-history');
const backupIncludePremiumData = document.getElementById('backup-include-premium-data');
const exportBackupButton = document.getElementById('export-backup');
const importBackupFileTrigger = document.getElementById('import-backup-file-trigger');
const importBackupFileInput = document.getElementById('import-backup-file');
let settingsEditingIndex = -1;
const defaultSources = [];
const SETTINGS_SOURCE_PREVIEW_LIMIT = 6;
let settingsSourceExpanded = false;
let settingsSourceSearchTerm = '';

function setStatus(target, kind, message) {
  target.textContent = message;
  target.className = `status ${kind}`;
}

function escapeHtml(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('\"', '&quot;')
    .replaceAll("'", '&#39;');
}

function normalizeSettings(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : {};
}

function parseSettingsJson() {
  return normalizeSettings(JSON.parse(settingsArea.value || '{}'));
}

function countEnabled(list) {
  return Array.isArray(list)
    ? list.filter((item) => item && typeof item === 'object' && item.id && item.enabled !== false).length
    : 0;
}

function parseNonNegativeNumber(value, fallback = 0) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return fallback;
  }
  return Math.floor(parsed);
}

function normalizeKeywordList(rawValue) {
  if (Array.isArray(rawValue)) {
    return rawValue.map((item) => String(item || '').trim()).filter(Boolean);
  }
  return String(rawValue || '')
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizeSourceDraft(rawSource, fallbackPriority) {
  return {
    id: String(rawSource?.id || '').trim(),
    name: String(rawSource?.name || '').trim(),
    baseUrl: String(rawSource?.baseUrl || '').trim(),
    searchPath: String(rawSource?.searchPath || '').trim(),
    detailPath: String(rawSource?.detailPath || '').trim(),
    priority: parseNonNegativeNumber(rawSource?.priority, fallbackPriority),
    enabled: rawSource?.enabled !== false,
  };
}

function normalizeAccessPasswords(rawValue) {
  if (!Array.isArray(rawValue)) {
    return [];
  }

  return rawValue
    .map((item) => String(item || '').trim())
    .filter(Boolean);
}

function countSubscriptionEntries(rawValue) {
  const trimmed = String(rawValue || '').trim();
  if (!trimmed) {
    return 0;
  }

  try {
    const parsed = JSON.parse(trimmed);
    if (Array.isArray(parsed)) {
      return parsed.filter((item) =>
        item
        && typeof item === 'object'
        && typeof item.name === 'string'
        && typeof item.url === 'string'
      ).length;
    }
  } catch (_) {
  }

  return trimmed
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean)
    .length;
}

function parseSubscriptionConfig(rawValue) {
  const trimmed = String(rawValue || '').trim();
  if (!trimmed) {
    return [];
  }

  try {
    const parsed = JSON.parse(trimmed);
    if (Array.isArray(parsed)) {
      return parsed
        .filter((item) =>
          item
          && typeof item === 'object'
          && typeof item.name === 'string'
          && typeof item.url === 'string'
        )
        .map((item, index) => ({
          id: String(item.id || `sub_${index + 1}`),
          name: item.name.trim(),
          url: item.url.trim(),
          lastUpdated: Number(item.lastUpdated || 0),
          autoRefresh: item.autoRefresh !== false,
        }));
    }
  } catch (_) {
  }

  return trimmed
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter((item) => /^https?:\/\//i.test(item))
    .map((url, index) => ({
      id: `sub_${index + 1}`,
      name: `系统预设源 ${index + 1}`,
      url,
      lastUpdated: 0,
      autoRefresh: true,
    }));
}

function normalizeImportedSource(rawSource, fallbackPriority) {
  if (!rawSource || typeof rawSource !== 'object') {
    return null;
  }
  if (typeof rawSource.id !== 'string' || typeof rawSource.name !== 'string' || typeof rawSource.baseUrl !== 'string') {
    return null;
  }

  return {
    id: rawSource.id.trim(),
    name: rawSource.name.trim(),
    baseUrl: rawSource.baseUrl.trim(),
    searchPath: typeof rawSource.searchPath === 'string' ? rawSource.searchPath.trim() : '',
    detailPath: typeof rawSource.detailPath === 'string' ? rawSource.detailPath.trim() : '',
    enabled: rawSource.enabled !== false,
    priority: parseNonNegativeNumber(rawSource.priority, fallbackPriority),
    group: rawSource.group === 'premium' ? 'premium' : 'normal',
  };
}

function parseImportedSources(rawValue) {
  const parsed = JSON.parse(String(rawValue || '').trim() || '[]');
  const sourceList = Array.isArray(parsed)
    ? parsed
    : Array.isArray(parsed.sources)
      ? parsed.sources
      : Array.isArray(parsed.list)
        ? parsed.list
        : null;

  if (!sourceList) {
    throw new Error('无法识别的线路 JSON 格式');
  }

  const normalSources = [];
  const premiumSources = [];

  sourceList.forEach((item, index) => {
    const normalized = normalizeImportedSource(item, index + 1);
    if (!normalized) {
      return;
    }
    if (normalized.group === 'premium') {
      premiumSources.push(normalized);
    } else {
      normalSources.push(normalized);
    }
  });

  return { normalSources, premiumSources };
}

function mergeSources(existing, nextSources) {
  const merged = [...existing];
  const sourceIds = new Set(existing.map((source) => source.id));

  nextSources.forEach((source) => {
    if (sourceIds.has(source.id)) {
      const index = merged.findIndex((item) => item.id === source.id);
      if (index >= 0) {
        merged[index] = { ...merged[index], ...source };
      }
      return;
    }

    merged.push({
      ...source,
      priority: source.priority || merged.length + 1,
    });
    sourceIds.add(source.id);
  });

  return merged;
}

function readSettingsSourceForm() {
  return normalizeSourceDraft({
    id: settingsSourceId.value,
    name: settingsSourceName.value,
    baseUrl: settingsSourceBaseUrl.value,
    searchPath: settingsSourceSearchPath.value,
    detailPath: settingsSourceDetailPath.value,
    priority: settingsSourcePriority.value,
    enabled: settingsSourceEnabled.checked,
  }, 1);
}

function fillSettingsSourceForm(source, index = -1) {
  const normalized = normalizeSourceDraft(source, index + 1 || 1);
  settingsEditingIndex = index;
  settingsSourceId.value = normalized.id;
  settingsSourceName.value = normalized.name;
  settingsSourceBaseUrl.value = normalized.baseUrl;
  settingsSourceSearchPath.value = normalized.searchPath;
  settingsSourceDetailPath.value = normalized.detailPath;
  settingsSourcePriority.value = String(normalized.priority || 1);
  settingsSourceEnabled.checked = normalized.enabled;
  settingsSourceSubmit.textContent = index >= 0 ? '保存线路' : '新增线路';
  settingsSourceCancel.disabled = index < 0;
}

function resetSettingsSourceForm() {
  fillSettingsSourceForm({}, -1);
  settingsSourcePriority.value = '1';
  settingsSourceEnabled.checked = true;
}

function syncQuickControls(settings) {
  quickSearchDisplayMode.value = settings.searchDisplayMode === 'grouped' ? 'grouped' : 'normal';
  quickSortBy.value = settings.sortBy || 'default';
  quickProxyMode.value = settings.proxyMode || 'retry';
  quickFullscreenType.value = settings.fullscreenType === 'window' ? 'window' : 'native';
  quickSearchHistory.checked = settings.searchHistory !== false;
  quickWatchHistory.checked = settings.watchHistory !== false;
  quickRealtimeLatency.checked = Boolean(settings.realtimeLatency);
  quickRememberScroll.checked = settings.rememberScrollPosition !== false;
  quickEpisodeReverseOrder.checked = Boolean(settings.episodeReverseOrder);
  quickAutoNextEpisode.checked = settings.autoNextEpisode !== false;
  quickShowModeIndicator.checked = Boolean(settings.showModeIndicator);
  quickAutoSkipIntro.checked = Boolean(settings.autoSkipIntro);
  quickSkipIntroSeconds.value = String(parseNonNegativeNumber(settings.skipIntroSeconds, 0));
  quickAutoSkipOutro.checked = Boolean(settings.autoSkipOutro);
  quickSkipOutroSeconds.value = String(parseNonNegativeNumber(settings.skipOutroSeconds, 0));
  quickAdFilterMode.value = ['off', 'keyword', 'heuristic', 'aggressive'].includes(settings.adFilterMode)
    ? settings.adFilterMode
    : (settings.adFilter ? 'heuristic' : 'off');
  quickAdKeywords.value = normalizeKeywordList(settings.adKeywords).join('\n');
}

function applyQuickControls(settings) {
  settings.searchDisplayMode = quickSearchDisplayMode.value === 'grouped' ? 'grouped' : 'normal';
  settings.sortBy = quickSortBy.value || 'default';
  settings.proxyMode = quickProxyMode.value || 'retry';
  settings.fullscreenType = quickFullscreenType.value === 'window' ? 'window' : 'native';
  settings.searchHistory = quickSearchHistory.checked;
  settings.watchHistory = quickWatchHistory.checked;
  settings.realtimeLatency = quickRealtimeLatency.checked;
  settings.rememberScrollPosition = quickRememberScroll.checked;
  settings.episodeReverseOrder = quickEpisodeReverseOrder.checked;
  settings.autoNextEpisode = quickAutoNextEpisode.checked;
  settings.showModeIndicator = quickShowModeIndicator.checked;
  settings.autoSkipIntro = quickAutoSkipIntro.checked;
  settings.skipIntroSeconds = parseNonNegativeNumber(quickSkipIntroSeconds.value, 0);
  settings.autoSkipOutro = quickAutoSkipOutro.checked;
  settings.skipOutroSeconds = parseNonNegativeNumber(quickSkipOutroSeconds.value, 0);
  settings.adFilterMode = ['keyword', 'heuristic', 'aggressive'].includes(quickAdFilterMode.value)
    ? quickAdFilterMode.value
    : 'off';
  settings.adFilter = settings.adFilterMode !== 'off';
  settings.adKeywords = normalizeKeywordList(quickAdKeywords.value);
  return settings;
}

function renderSettingsSources(settings) {
  const sources = Array.isArray(settings.sources) ? settings.sources : [];
  const premiumSources = Array.isArray(settings.premiumSources) ? settings.premiumSources : [];
  settingsSourceCount.textContent = `线路 ${countEnabled(sources)} / ${sources.length}`;
  settingsPremiumCount.textContent = `Premium ${countEnabled(premiumSources)} / ${premiumSources.length}`;

  const normalizedSearch = String(settingsSourceSearchTerm || '').trim().toLowerCase();
  const filteredSources = sources
    .map((source, index) => ({ source, index }))
    .filter(({ source }) => {
      if (!normalizedSearch) {
        return true;
      }
      return [
        source?.name,
        source?.id,
        source?.baseUrl,
      ].some((value) => String(value || '').toLowerCase().includes(normalizedSearch));
    });

  if (settingsSourceToggle instanceof HTMLElement) {
    const shouldShowToggle = !normalizedSearch && sources.length > SETTINGS_SOURCE_PREVIEW_LIMIT;
    settingsSourceToggle.classList.toggle('hidden', !shouldShowToggle);
    settingsSourceToggle.textContent = settingsSourceExpanded ? '收起预览' : `显示全部 (${sources.length})`;
  }

  if (!sources.length) {
    settingsSourceList.className = 'saved-list empty-state';
    settingsSourceList.innerHTML = '当前没有普通线路配置。';
    return;
  }

  if (!filteredSources.length) {
    settingsSourceList.className = 'saved-list empty-state';
    settingsSourceList.innerHTML = '没有匹配的线路。';
    return;
  }

  const visibleSources = !normalizedSearch && !settingsSourceExpanded
    ? filteredSources.slice(0, SETTINGS_SOURCE_PREVIEW_LIMIT)
    : filteredSources;

  settingsSourceList.className = 'saved-list';
  settingsSourceList.innerHTML = visibleSources.map(({ source, index }) => {
    const enabled = source && source.enabled !== false;
    const priority = typeof source?.priority === 'number' ? source.priority : index + 1;
    return `
      <article class="saved-item source-item">
        <div class="source-item-header">
          <div class="stack compact-card-body">
            <strong>${escapeHtml(source?.name || source?.id || '未命名线路')}</strong>
            <div class="source-item-meta">
              <span class="source-item-url">Base URL: ${escapeHtml(source?.baseUrl || '-')}</span>
              <span class="chip">ID: ${escapeHtml(source?.id || 'unknown')}</span>
              <span>优先级: ${escapeHtml(priority)}</span>
            </div>
          </div>
          <span class="chip">${enabled ? '已启用' : '已禁用'}</span>
        </div>
        <div class="source-actions">
          <button class="button button-small" type="button" data-settings-source-action="toggle" data-index="${index}">${enabled ? '禁用' : '启用'}</button>
          <button class="button button-small" type="button" data-settings-source-action="edit" data-index="${index}">编辑</button>
          <button class="button button-small" type="button" data-settings-source-action="up" data-index="${index}" ${index === 0 ? 'disabled' : ''}>上移</button>
          <button class="button button-small" type="button" data-settings-source-action="down" data-index="${index}" ${index === sources.length - 1 ? 'disabled' : ''}>下移</button>
          <button class="button button-small danger" type="button" data-settings-source-action="remove" data-index="${index}">删除</button>
        </div>
      </article>
    `;
  }).join('');
}

function renderSubscriptions(settings) {
  const subscriptions = Array.isArray(settings.subscriptions) ? settings.subscriptions : [];
  if (settingsSubscriptionCount) {
    settingsSubscriptionCount.textContent = `订阅配置 ${subscriptions.length}`;
  }

  if (!settingsSubscriptionList) {
    return;
  }

  if (!subscriptions.length) {
    settingsSubscriptionList.className = 'saved-list empty-state';
    settingsSubscriptionList.textContent = '当前还没有订阅配置。';
    return;
  }

  settingsSubscriptionList.className = 'saved-list';
  settingsSubscriptionList.innerHTML = subscriptions.map((subscription, index) => `
    <article class="saved-item source-item">
      <div class="source-item-header">
        <div class="stack compact-card-body">
          <strong>${escapeHtml(subscription?.name || `订阅 ${index + 1}`)}</strong>
          <div class="source-item-meta">
            <span>URL: ${escapeHtml(subscription?.url || '-')}</span>
            <span>自动刷新: ${subscription?.autoRefresh === false ? '关闭' : '开启'}</span>
          </div>
        </div>
        <span class="chip">${subscription?.lastUpdated ? '已同步' : '未同步'}</span>
      </div>
      <div class="source-actions">
        <button class="button button-small" type="button" data-subscription-action="sync" data-index="${index}">同步线路</button>
        <button class="button button-small danger" type="button" data-subscription-action="remove" data-index="${index}">移除</button>
      </div>
    </article>
  `).join('');
}

function renderAccessPasswords(settings) {
  const passwords = normalizeAccessPasswords(settings.accessPasswords);

  if (accessControlEnabled instanceof HTMLInputElement) {
    accessControlEnabled.checked = Boolean(settings.passwordAccess);
  }

  if (accessPasswordCount) {
    accessPasswordCount.textContent = `密码 ${passwords.length}`;
  }

  if (!(accessPasswordList instanceof HTMLElement)) {
    return;
  }

  if (!passwords.length) {
    accessPasswordList.className = 'saved-list empty-state';
    accessPasswordList.textContent = '当前还没有本地访问密码。';
    return;
  }

  accessPasswordList.className = 'saved-list';
  accessPasswordList.innerHTML = passwords.map((password, index) => `
    <article class="saved-item source-item">
      <div class="source-item-header">
        <div class="stack compact-card-body">
          <strong>本地密码 ${index + 1}</strong>
          <div class="source-item-meta">
            <span>内容：${'•'.repeat(Math.max(6, Math.min(password.length, 16)))}</span>
          </div>
        </div>
        <span class="chip">已配置</span>
      </div>
      <div class="source-actions">
        <button class="button button-small danger" type="button" data-access-password-remove="${escapeHtml(password)}">删除</button>
      </div>
    </article>
  `).join('');
}

function applySettingsSnapshot(settings, kind, message) {
  const normalized = normalizeSettings(settings);
  settingsArea.value = JSON.stringify(normalized, null, 2);
  syncQuickControls(normalized);
  renderSettingsSources(normalized);
  renderSubscriptions(normalized);
  renderAccessPasswords(normalized);
  if (subscriptionSummary) {
    subscriptionSummary.textContent = `订阅 ${countSubscriptionEntries(subscriptionSourcesEl?.textContent || '')}`;
  }
  if (message) {
    setStatus(settingsStatus, kind || 'success', message);
  }
}

function downloadText(filename, content) {
  const blob = new Blob([content], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function getBackupOptions() {
  return {
    searchHistory: backupIncludeSearchHistory instanceof HTMLInputElement ? backupIncludeSearchHistory.checked : true,
    watchHistory: backupIncludeWatchHistory instanceof HTMLInputElement ? backupIncludeWatchHistory.checked : true,
    premiumData: backupIncludePremiumData instanceof HTMLInputElement ? backupIncludePremiumData.checked : true,
  };
}

async function fetchUserDataValue(key) {
  const response = await fetch(`/api/user/data?key=${encodeURIComponent(key)}`);
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error || `读取 ${key} 失败`);
  }
  return data.data;
}

async function persistUserDataValue(key, value) {
  const response = await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key, value }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error || `保存 ${key} 失败`);
  }
}

function parseBackupPayload(rawValue) {
  const parsed = JSON.parse(String(rawValue || '').trim() || '{}');
  if (parsed && typeof parsed === 'object' && !Array.isArray(parsed) && parsed.settings && typeof parsed.settings === 'object') {
    return {
      settings: normalizeSettings(parsed.settings),
      userData: parsed.userData && typeof parsed.userData === 'object' && !Array.isArray(parsed.userData)
        ? parsed.userData
        : {},
    };
  }

  if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
    return {
      settings: normalizeSettings(parsed),
      userData: {},
    };
  }

  throw new Error('无法识别的备份文件格式');
}

async function exportFullBackup() {
  const settings = applyQuickControls(parseSettingsJson());
  const options = getBackupOptions();
  const keys = [];

  if (options.searchHistory) {
    keys.push('search-history');
  }
  if (options.watchHistory) {
    keys.push('history', 'favorites');
  }
  if (options.premiumData) {
    keys.push('premium-search-history', 'premium-history', 'premium-favorites', 'premium-tags');
  }

  const entries = await Promise.all(keys.map(async (key) => [key, await fetchUserDataValue(key)]));
  const backup = {
    format: 'kvideo-rust-backup',
    version: 1,
    exportedAt: new Date().toISOString(),
    options,
    settings,
    userData: Object.fromEntries(entries),
  };

  downloadText(`kvideo-rust-backup-${Date.now()}.json`, JSON.stringify(backup, null, 2));
}

async function importBackupPayload(rawValue) {
  const { settings, userData } = parseBackupPayload(rawValue);

  await persistUserDataValue('settings', settings);
  const localStorageWrites = [['kvideo-settings', settings]];

  for (const [key, value] of Object.entries(userData)) {
    await persistUserDataValue(key, value);
    if (key === 'search-history') {
      localStorageWrites.push(['kvideo-search-history', value]);
    } else if (key === 'history') {
      localStorageWrites.push(['kvideo-history-store', value]);
    } else if (key === 'favorites') {
      localStorageWrites.push(['kvideo-favorites-store', value]);
    } else if (key === 'premium-search-history') {
      localStorageWrites.push(['kvideo-premium-search-history', value]);
    } else if (key === 'premium-history') {
      localStorageWrites.push(['kvideo-premium-history-store', value]);
    } else if (key === 'premium-favorites') {
      localStorageWrites.push(['kvideo-premium-favorites-store', value]);
    } else if (key === 'premium-tags') {
      localStorageWrites.push(['kvideo_premium_custom_tags', value]);
    }
  }

  if (typeof window !== 'undefined') {
    localStorageWrites.forEach(([storageKey, value]) => {
      window.localStorage.setItem(storageKey, JSON.stringify(value));
    });
  }

  applySettingsSnapshot(settings, 'success', '备份已导入并恢复到当前账号');
}

function updateSources(mutator, successMessage) {
  let settings;
  try {
    settings = parseSettingsJson();
    const currentSources = Array.isArray(settings.sources) ? settings.sources.map((item) => ({ ...item })) : [];
    settings.sources = mutator(currentSources);
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
    return;
  }
  applySettingsSnapshot(settings, 'success', successMessage);
}

function updateSettings(mutator, successMessage) {
  let settings;
  try {
    settings = parseSettingsJson();
    settings = mutator(settings);
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
    return;
  }

  applySettingsSnapshot(settings, 'success', successMessage);
}

async function fetchSourcesFromSubscriptionUrl(url) {
  const isExternal = /^https?:\/\//i.test(url);
  const fetchUrl = isExternal ? `/api/proxy?url=${encodeURIComponent(url)}` : url;
  const response = await fetch(fetchUrl, {
    headers: {
      Accept: 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`获取订阅失败: ${response.status}`);
  }

  const text = await response.text();
  return parseImportedSources(text);
}

document.getElementById('apply-quick-settings')?.addEventListener('click', () => {
  try {
    const settings = applyQuickControls(parseSettingsJson());
    applySettingsSnapshot(settings, 'success', '已把常用设置同步到 JSON');
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
  }
});

document.getElementById('sync-quick-settings')?.addEventListener('click', () => {
  try {
    const settings = parseSettingsJson();
    syncQuickControls(settings);
    setStatus(settingsStatus, 'success', '已从 JSON 读取常用设置');
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
  }
});

document.getElementById('format-settings')?.addEventListener('click', () => {
  try {
    applySettingsSnapshot(parseSettingsJson(), 'success', '已格式化 JSON');
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
  }
});

document.getElementById('refresh-settings')?.addEventListener('click', async () => {
  const response = await fetch('/api/user/data?key=settings');
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setStatus(settingsStatus, 'error', data.error || '刷新失败');
    return;
  }

  applySettingsSnapshot(data.data || {}, 'success', '已从服务端刷新');
});

document.getElementById('export-settings')?.addEventListener('click', () => {
  downloadText(`kvideo-rust-settings-${Date.now()}.json`, settingsArea.value || '{}');
  setStatus(settingsStatus, 'success', '已导出当前设置 JSON');
});

exportBackupButton?.addEventListener('click', () => {
  exportFullBackup().then(() => {
    setStatus(settingsStatus, 'success', '已导出完整备份');
  }).catch((error) => {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : '导出完整备份失败');
  });
});

importBackupFileTrigger?.addEventListener('click', () => {
  if (importBackupFileInput instanceof HTMLInputElement) {
    importBackupFileInput.click();
  }
});

importBackupFileInput?.addEventListener('change', async () => {
  if (!(importBackupFileInput instanceof HTMLInputElement)) {
    return;
  }
  const file = importBackupFileInput.files && importBackupFileInput.files[0];
  if (!file) {
    return;
  }

  try {
    const text = await file.text();
    await importBackupPayload(text);
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : '导入备份文件失败');
  } finally {
    importBackupFileInput.value = '';
  }
});

document.getElementById('copy-subscription-sources')?.addEventListener('click', async () => {
  const text = subscriptionSourcesEl?.textContent || '';
  try {
    await navigator.clipboard.writeText(text);
    setStatus(settingsStatus, 'success', '已复制系统订阅源配置');
  } catch (_) {
    setStatus(settingsStatus, 'error', '复制失败，请手动复制订阅源内容');
  }
});

document.getElementById('import-source-json')?.addEventListener('click', () => {
  const payload = settingsImportPayload?.value || '';
  if (!payload.trim()) {
    setStatus(settingsStatus, 'error', '请先粘贴线路 JSON');
    return;
  }

  updateSettings((settings) => {
    const imported = parseImportedSources(payload);
    settings.sources = mergeSources(Array.isArray(settings.sources) ? settings.sources : [], imported.normalSources);
    settings.premiumSources = mergeSources(Array.isArray(settings.premiumSources) ? settings.premiumSources : [], imported.premiumSources);
    return settings;
  }, '线路 JSON 已导入到当前设置');
});

document.getElementById('import-subscriptions')?.addEventListener('click', () => {
  const payload = settingsImportPayload?.value || '';
  const importedSubscriptions = parseSubscriptionConfig(payload);
  if (!importedSubscriptions.length) {
    setStatus(settingsStatus, 'error', '请粘贴有效的订阅 URL 或订阅 JSON');
    return;
  }

  updateSettings((settings) => {
    const current = Array.isArray(settings.subscriptions) ? settings.subscriptions : [];
    const merged = [...current];
    importedSubscriptions.forEach((subscription) => {
      const index = merged.findIndex((item) => item.url === subscription.url);
      if (index >= 0) {
        merged[index] = { ...merged[index], ...subscription };
      } else {
        merged.push(subscription);
      }
    });
    settings.subscriptions = merged;
    return settings;
  }, `已导入 ${importedSubscriptions.length} 条订阅配置`);
});

document.getElementById('sync-system-subscriptions')?.addEventListener('click', () => {
  const importedSubscriptions = parseSubscriptionConfig(subscriptionSourcesEl?.textContent || '');
  if (!importedSubscriptions.length) {
    setStatus(settingsStatus, 'error', '当前没有可同步的系统订阅源');
    return;
  }

  updateSettings((settings) => {
    const current = Array.isArray(settings.subscriptions) ? settings.subscriptions : [];
    const merged = [...current];
    importedSubscriptions.forEach((subscription) => {
      const index = merged.findIndex((item) => item.url === subscription.url);
      if (index >= 0) {
        merged[index] = { ...merged[index], ...subscription };
      } else {
        merged.push(subscription);
      }
    });
    settings.subscriptions = merged;
    return settings;
  }, `已同步 ${importedSubscriptions.length} 条系统订阅配置`);
});

document.getElementById('save-settings')?.addEventListener('click', async () => {
  let parsed;
  try {
    parsed = applyQuickControls(parseSettingsJson());
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : 'JSON 格式错误');
    return;
  }

  const response = await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: 'settings', value: parsed }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setStatus(settingsStatus, 'error', data.error || '保存失败');
    return;
  }

  applySettingsSnapshot(parsed, 'success', '设置已保存');
});

document.getElementById('enable-all-sources')?.addEventListener('click', () => {
  updateSources(
    (sources) => sources.map((source) => ({ ...source, enabled: true })),
    '已启用全部普通线路'
  );
});

document.getElementById('disable-all-sources')?.addEventListener('click', () => {
  updateSources(
    (sources) => sources.map((source) => ({ ...source, enabled: false })),
    '已禁用全部普通线路'
  );
});

document.getElementById('restore-default-sources')?.addEventListener('click', () => {
  updateSources(
    () => defaultSources.map((source) => ({ ...source })),
    defaultSources.length ? '已恢复仓库默认普通线路' : '当前仓库未内置默认普通线路，已清空到默认状态'
  );
  resetSettingsSourceForm();
});

settingsSourceSearch?.addEventListener('input', (event) => {
  settingsSourceSearchTerm = event.target.value || '';
  try {
    renderSettingsSources(parseSettingsJson());
  } catch (_) {
  }
});

settingsSourceToggle?.addEventListener('click', () => {
  settingsSourceExpanded = !settingsSourceExpanded;
  try {
    renderSettingsSources(parseSettingsJson());
  } catch (_) {
  }
});

settingsSourceForm?.addEventListener('submit', (event) => {
  event.preventDefault();

  const draft = readSettingsSourceForm();
  if (!draft.id || !draft.name || !draft.baseUrl || !draft.searchPath || !draft.detailPath) {
    setStatus(settingsStatus, 'error', '请完整填写线路表单');
    return;
  }

  updateSources((sources) => {
    const duplicateIndex = sources.findIndex((source, index) =>
      source?.id === draft.id && index !== settingsEditingIndex
    );
    if (duplicateIndex >= 0) {
      throw new Error(`线路 ID「${draft.id}」已存在`);
    }

    if (settingsEditingIndex >= 0 && settingsEditingIndex < sources.length) {
      sources[settingsEditingIndex] = draft;
      return sources;
    }

    return [...sources, draft];
  }, settingsEditingIndex >= 0 ? `线路 ${draft.name} 已更新` : `线路 ${draft.name} 已新增`);

  resetSettingsSourceForm();
});

settingsSourceCancel?.addEventListener('click', () => {
  resetSettingsSourceForm();
  setStatus(settingsStatus, 'success', '已取消线路编辑');
});

settingsSourceList?.addEventListener('click', (event) => {
  const button = event.target.closest('[data-settings-source-action]');
  if (!button) {
    return;
  }

  const index = Number.parseInt(button.dataset.index || '-1', 10);
  if (!Number.isInteger(index) || index < 0) {
    return;
  }

  const action = button.dataset.settingsSourceAction;
  updateSources((sources) => {
    if (index >= sources.length) {
      return sources;
    }

    if (action === 'edit') {
      fillSettingsSourceForm(sources[index], index);
      return sources;
    }

    if (action === 'toggle') {
      sources[index] = { ...sources[index], enabled: sources[index].enabled === false };
      return sources;
    }

    if (action === 'up' && index > 0) {
      [sources[index - 1], sources[index]] = [sources[index], sources[index - 1]];
      return sources;
    }

    if (action === 'down' && index < sources.length - 1) {
      [sources[index], sources[index + 1]] = [sources[index + 1], sources[index]];
      return sources;
    }

    if (action === 'remove') {
      return sources.filter((_, itemIndex) => itemIndex !== index);
    }

    return sources;
  }, '线路配置已更新');

  if (action === 'edit') {
    setStatus(settingsStatus, 'success', '已载入线路表单');
  }
});

settingsSubscriptionList?.addEventListener('click', async (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest('[data-subscription-action]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  const index = Number.parseInt(button.dataset.index || '-1', 10);
  if (!Number.isInteger(index) || index < 0) {
    return;
  }

  const action = button.dataset.subscriptionAction;
  if (action === 'remove') {
    updateSettings((settings) => {
      const subscriptions = Array.isArray(settings.subscriptions) ? settings.subscriptions.slice() : [];
      subscriptions.splice(index, 1);
      settings.subscriptions = subscriptions;
      return settings;
    }, '订阅配置已移除');
    return;
  }

  if (action !== 'sync') {
    return;
  }

  const currentSettings = parseSettingsJson();
  const subscriptions = Array.isArray(currentSettings.subscriptions) ? currentSettings.subscriptions : [];
  const subscription = subscriptions[index];
  if (!subscription?.url) {
    setStatus(settingsStatus, 'error', '无效的订阅配置');
    return;
  }

  setStatus(settingsStatus, 'muted', `正在同步订阅「${subscription.name || subscription.url}」...`);

  try {
    const imported = await fetchSourcesFromSubscriptionUrl(subscription.url);
    updateSettings((settings) => {
      settings.sources = mergeSources(Array.isArray(settings.sources) ? settings.sources : [], imported.normalSources);
      settings.premiumSources = mergeSources(Array.isArray(settings.premiumSources) ? settings.premiumSources : [], imported.premiumSources);
      const nextSubscriptions = Array.isArray(settings.subscriptions) ? settings.subscriptions.slice() : [];
      if (nextSubscriptions[index]) {
        nextSubscriptions[index] = {
          ...nextSubscriptions[index],
          lastUpdated: Date.now(),
        };
      }
      settings.subscriptions = nextSubscriptions;
      return settings;
    }, `订阅「${subscription.name || subscription.url}」已同步`);
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : '同步订阅失败');
  }
});

accessControlEnabled?.addEventListener('change', () => {
  if (!(accessControlEnabled instanceof HTMLInputElement)) {
    return;
  }

  updateSettings((settings) => {
    settings.passwordAccess = accessControlEnabled.checked;
    settings.accessPasswords = normalizeAccessPasswords(settings.accessPasswords);
    return settings;
  }, accessControlEnabled.checked ? '已启用本地访问控制配置' : '已关闭本地访问控制配置');
});

accessPasswordForm?.addEventListener('submit', (event) => {
  event.preventDefault();
  if (!(accessPasswordInput instanceof HTMLInputElement)) {
    return;
  }

  const nextPassword = accessPasswordInput.value.trim();
  if (!nextPassword) {
    setStatus(settingsStatus, 'error', '请输入要添加的本地访问密码');
    return;
  }

  updateSettings((settings) => {
    const passwords = normalizeAccessPasswords(settings.accessPasswords);
    if (passwords.includes(nextPassword)) {
      throw new Error('该本地访问密码已存在');
    }
    settings.passwordAccess = true;
    settings.accessPasswords = [...passwords, nextPassword];
    return settings;
  }, '已添加本地访问密码');

  accessPasswordInput.value = '';
});

accessPasswordList?.addEventListener('click', (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest('[data-access-password-remove]');
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  const password = String(button.dataset.accessPasswordRemove || '').trim();
  if (!password) {
    return;
  }

  updateSettings((settings) => {
    settings.accessPasswords = normalizeAccessPasswords(settings.accessPasswords)
      .filter((item) => item !== password);
    return settings;
  }, '已移除本地访问密码');
});

document.getElementById('password-form')?.addEventListener('submit', async (event) => {
  event.preventDefault();
  const response = await fetch('/api/auth/password', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      currentPassword: document.getElementById('current-password').value,
      newPassword: document.getElementById('new-password').value,
    }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setStatus(passwordStatus, 'error', data.error || '修改密码失败');
    return;
  }

  document.getElementById('current-password').value = '';
  document.getElementById('new-password').value = '';
  setStatus(passwordStatus, 'success', data.message || '密码已更新');
});

document.getElementById('logout-button')?.addEventListener('click', async () => {
  await fetch('/api/auth/logout', { method: 'POST' });
  window.location.href = '/login';
});

document.getElementById('clear-synced-data')?.addEventListener('click', async () => {
  if (!window.confirm('确定清空当前账号已同步的历史、收藏和缓存数据？此操作不可撤销。')) {
    return;
  }

  const resetPayloads = [
    { key: 'history', value: [] },
    { key: 'favorites', value: [] },
    { key: 'search-history', value: [] },
    { key: 'premium-search-history', value: [] },
    { key: 'premium-history', value: [] },
    { key: 'premium-favorites', value: [] },
    { key: 'search-cache', value: {} },
    { key: 'premium-tags', value: [] },
  ];

  try {
    await Promise.all(resetPayloads.map(async ({ key, value }) => {
      const response = await fetch('/api/user/data', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, value }),
      });
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        throw new Error(data.error || `清空 ${key} 失败`);
      }
    }));

    if (typeof window !== 'undefined') {
      [
        'kvideo-search-history',
        'kvideo-premium-search-history',
        'kvideo-history-store',
        'kvideo-favorites-store',
        'kvideo-premium-history-store',
        'kvideo-premium-favorites-store',
        'kvideo_premium_custom_tags',
      ].forEach((storageKey) => window.localStorage.removeItem(storageKey));
    }

    setStatus(settingsStatus, 'success', '已清空当前账号已同步数据');
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : '清空已同步数据失败');
  }
});

document.getElementById('reset-all-data')?.addEventListener('click', async () => {
  if (!window.confirm('确定清除所有数据？这会清空设置、历史、收藏、本地缓存，并退出当前登录。')) {
    return;
  }

  const resetPayloads = [
    { key: 'settings', value: {} },
    { key: 'history', value: [] },
    { key: 'favorites', value: [] },
    { key: 'search-history', value: [] },
    { key: 'premium-search-history', value: [] },
    { key: 'premium-history', value: [] },
    { key: 'premium-favorites', value: [] },
    { key: 'search-cache', value: {} },
    { key: 'premium-tags', value: [] },
  ];

  try {
    await Promise.all(resetPayloads.map(async ({ key, value }) => {
      const response = await fetch('/api/user/data', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, value }),
      });
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        throw new Error(data.error || `重置 ${key} 失败`);
      }
    }));

    if (typeof window !== 'undefined') {
      window.localStorage.clear();
      window.sessionStorage.clear();

      if ('caches' in window) {
        const cacheKeys = await caches.keys();
        await Promise.all(cacheKeys.map((cacheKey) => caches.delete(cacheKey)));
      }
    }

    await fetch('/api/auth/logout', { method: 'POST' });
    window.location.href = '/login';
  } catch (error) {
    setStatus(settingsStatus, 'error', error instanceof Error ? error.message : '清除所有数据失败');
  }
});

applySettingsSnapshot(initialSettings);
resetSettingsSourceForm();
"#;

pub(super) const PREMIUM_SETTINGS_SCRIPT: &str = r#"
const premiumArea = document.getElementById('premium-json');
const premiumStatus = document.getElementById('premium-status');
const initialPremium = JSON.parse(document.getElementById('initial-premium-settings').textContent || '[]');
const premiumSourceList = document.getElementById('premium-source-list');
const premiumSourceCount = document.getElementById('premium-source-count');
const premiumSourceSearch = document.getElementById('premium-source-search');
const premiumSourceToggle = document.getElementById('toggle-premium-source-limit');
const premiumSourceForm = document.getElementById('premium-source-form');
const premiumSourceId = document.getElementById('premium-source-id');
const premiumSourceName = document.getElementById('premium-source-name');
const premiumSourceBaseUrl = document.getElementById('premium-source-base-url');
const premiumSourcePriority = document.getElementById('premium-source-priority');
const premiumSourceSearchPath = document.getElementById('premium-source-search-path');
const premiumSourceDetailPath = document.getElementById('premium-source-detail-path');
const premiumSourceEnabled = document.getElementById('premium-source-enabled');
const premiumSourceSubmit = document.getElementById('premium-source-submit');
const premiumSourceCancel = document.getElementById('premium-source-cancel');
let premiumEditingIndex = -1;
const defaultPremiumSources = [];
const PREMIUM_SOURCE_PREVIEW_LIMIT = 6;
let premiumSourceExpanded = false;
let premiumSourceSearchTerm = '';

function setPremiumStatus(kind, message) {
  premiumStatus.textContent = message;
  premiumStatus.className = `status ${kind}`;
}

function escapePremiumHtml(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('\"', '&quot;')
    .replaceAll("'", '&#39;');
}

function parsePremiumSources() {
  const parsed = JSON.parse(premiumArea.value || '[]');
  if (!Array.isArray(parsed)) {
    throw new Error('Premium Sources 必须是数组');
  }
  return parsed;
}

function countPremiumEnabled(list) {
  return list.filter((item) => item && typeof item === 'object' && item.id && item.enabled !== false).length;
}

function normalizePremiumDraft(rawSource, fallbackPriority) {
  return {
    id: String(rawSource?.id || '').trim(),
    name: String(rawSource?.name || '').trim(),
    baseUrl: String(rawSource?.baseUrl || '').trim(),
    searchPath: String(rawSource?.searchPath || '').trim(),
    detailPath: String(rawSource?.detailPath || '').trim(),
    priority: parseNonNegativeNumber(rawSource?.priority, fallbackPriority),
    enabled: rawSource?.enabled !== false,
  };
}

function readPremiumSourceForm() {
  return normalizePremiumDraft({
    id: premiumSourceId.value,
    name: premiumSourceName.value,
    baseUrl: premiumSourceBaseUrl.value,
    searchPath: premiumSourceSearchPath.value,
    detailPath: premiumSourceDetailPath.value,
    priority: premiumSourcePriority.value,
    enabled: premiumSourceEnabled.checked,
  }, 1);
}

function fillPremiumSourceForm(source, index = -1) {
  const normalized = normalizePremiumDraft(source, index + 1 || 1);
  premiumEditingIndex = index;
  premiumSourceId.value = normalized.id;
  premiumSourceName.value = normalized.name;
  premiumSourceBaseUrl.value = normalized.baseUrl;
  premiumSourceSearchPath.value = normalized.searchPath;
  premiumSourceDetailPath.value = normalized.detailPath;
  premiumSourcePriority.value = String(normalized.priority || 1);
  premiumSourceEnabled.checked = normalized.enabled;
  premiumSourceSubmit.textContent = index >= 0 ? '保存高级源' : '新增高级源';
  premiumSourceCancel.disabled = index < 0;
}

function resetPremiumSourceForm() {
  fillPremiumSourceForm({}, -1);
  premiumSourcePriority.value = '1';
  premiumSourceEnabled.checked = true;
}

function renderPremiumSources(list) {
  premiumSourceCount.textContent = `源 ${countPremiumEnabled(list)} / ${list.length}`;

  const normalizedSearch = String(premiumSourceSearchTerm || '').trim().toLowerCase();
  const filteredSources = list
    .map((source, index) => ({ source, index }))
    .filter(({ source }) => {
      if (!normalizedSearch) {
        return true;
      }
      return [
        source?.name,
        source?.id,
        source?.baseUrl,
      ].some((value) => String(value || '').toLowerCase().includes(normalizedSearch));
    });

  if (premiumSourceToggle instanceof HTMLElement) {
    const shouldShowToggle = !normalizedSearch && list.length > PREMIUM_SOURCE_PREVIEW_LIMIT;
    premiumSourceToggle.classList.toggle('hidden', !shouldShowToggle);
    premiumSourceToggle.textContent = premiumSourceExpanded ? '收起预览' : `显示全部 (${list.length})`;
  }

  if (!list.length) {
    premiumSourceList.className = 'saved-list empty-state';
    premiumSourceList.innerHTML = '当前没有 Premium 源配置。';
    return;
  }

  if (!filteredSources.length) {
    premiumSourceList.className = 'saved-list empty-state';
    premiumSourceList.innerHTML = '没有匹配的高级源。';
    return;
  }

  const visibleSources = !normalizedSearch && !premiumSourceExpanded
    ? filteredSources.slice(0, PREMIUM_SOURCE_PREVIEW_LIMIT)
    : filteredSources;

  premiumSourceList.className = 'saved-list';
  premiumSourceList.innerHTML = visibleSources.map(({ source, index }) => {
    const enabled = source && source.enabled !== false;
    const priority = typeof source?.priority === 'number' ? source.priority : index + 1;
    return `
      <article class="saved-item source-item">
        <div class="source-item-header">
          <div class="stack compact-card-body">
            <strong>${escapePremiumHtml(source?.name || source?.id || '未命名高级源')}</strong>
            <div class="source-item-meta">
              <span class="source-item-url">Base URL: ${escapePremiumHtml(source?.baseUrl || '-')}</span>
              <span>ID: ${escapePremiumHtml(source?.id || 'unknown')}</span>
              <span>优先级: ${escapePremiumHtml(priority)}</span>
            </div>
          </div>
          <span class="chip">${enabled ? '已启用' : '已禁用'}</span>
        </div>
        <div class="source-actions">
          <button class="button button-small" type="button" data-premium-source-action="toggle" data-index="${index}">${enabled ? '禁用' : '启用'}</button>
          <button class="button button-small" type="button" data-premium-source-action="edit" data-index="${index}">编辑</button>
          <button class="button button-small" type="button" data-premium-source-action="up" data-index="${index}" ${index === 0 ? 'disabled' : ''}>上移</button>
          <button class="button button-small" type="button" data-premium-source-action="down" data-index="${index}" ${index === list.length - 1 ? 'disabled' : ''}>下移</button>
          <button class="button button-small danger" type="button" data-premium-source-action="remove" data-index="${index}">删除</button>
        </div>
      </article>
    `;
  }).join('');
}

function applyPremiumSnapshot(list, kind, message) {
  premiumArea.value = JSON.stringify(list, null, 2);
  renderPremiumSources(list);
  if (message) {
    setPremiumStatus(kind || 'success', message);
  }
}

function downloadPremiumJson() {
  const blob = new Blob([premiumArea.value || '[]'], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `kvideo-rust-premium-sources-${Date.now()}.json`;
  link.click();
  URL.revokeObjectURL(url);
}

function updatePremiumSources(mutator, successMessage) {
  let nextSources;
  try {
    nextSources = mutator(parsePremiumSources().map((item) => ({ ...item })));
  } catch (error) {
    setPremiumStatus('error', error instanceof Error ? error.message : 'JSON 格式错误');
    return;
  }

  applyPremiumSnapshot(nextSources, 'success', successMessage);
}

document.getElementById('format-premium')?.addEventListener('click', () => {
  try {
    applyPremiumSnapshot(parsePremiumSources(), 'success', '已格式化 Premium Sources JSON');
  } catch (error) {
    setPremiumStatus('error', error instanceof Error ? error.message : 'JSON 格式错误');
  }
});

document.getElementById('download-premium')?.addEventListener('click', () => {
  downloadPremiumJson();
  setPremiumStatus('success', '已导出当前 Premium Sources');
});

document.getElementById('premium-enable-all')?.addEventListener('click', () => {
  updatePremiumSources(
    (sources) => sources.map((source) => ({ ...source, enabled: true })),
    '已启用全部高级源'
  );
});

document.getElementById('premium-disable-all')?.addEventListener('click', () => {
  updatePremiumSources(
    (sources) => sources.map((source) => ({ ...source, enabled: false })),
    '已禁用全部高级源'
  );
});

document.getElementById('restore-default-premium-sources')?.addEventListener('click', () => {
  updatePremiumSources(
    () => defaultPremiumSources.map((source) => ({ ...source })),
    defaultPremiumSources.length ? '已恢复仓库默认高级源' : '当前仓库未内置默认高级源，已清空到默认状态'
  );
  resetPremiumSourceForm();
});

premiumSourceToggle?.addEventListener('click', () => {
  premiumSourceExpanded = !premiumSourceExpanded;
  renderPremiumSources(parsePremiumSources());
});

premiumSourceSearch?.addEventListener('input', () => {
  premiumSourceSearchTerm = premiumSourceSearch.value || '';
  renderPremiumSources(parsePremiumSources());
});

premiumSourceForm?.addEventListener('submit', (event) => {
  event.preventDefault();

  const draft = readPremiumSourceForm();
  if (!draft.id || !draft.name || !draft.baseUrl || !draft.searchPath || !draft.detailPath) {
    setPremiumStatus('error', '请完整填写高级源表单');
    return;
  }

  updatePremiumSources((sources) => {
    const duplicateIndex = sources.findIndex((source, index) =>
      source?.id === draft.id && index !== premiumEditingIndex
    );
    if (duplicateIndex >= 0) {
      throw new Error(`高级源 ID「${draft.id}」已存在`);
    }

    if (premiumEditingIndex >= 0 && premiumEditingIndex < sources.length) {
      sources[premiumEditingIndex] = draft;
      return sources;
    }

    return [...sources, draft];
  }, premiumEditingIndex >= 0 ? `高级源 ${draft.name} 已更新` : `高级源 ${draft.name} 已新增`);

  resetPremiumSourceForm();
});

premiumSourceCancel?.addEventListener('click', () => {
  resetPremiumSourceForm();
  setPremiumStatus('success', '已取消高级源编辑');
});

premiumSourceList?.addEventListener('click', (event) => {
  const button = event.target.closest('[data-premium-source-action]');
  if (!button) {
    return;
  }

  const index = Number.parseInt(button.dataset.index || '-1', 10);
  if (!Number.isInteger(index) || index < 0) {
    return;
  }

  const action = button.dataset.premiumSourceAction;
  updatePremiumSources((sources) => {
    if (index >= sources.length) {
      return sources;
    }

    if (action === 'edit') {
      fillPremiumSourceForm(sources[index], index);
      return sources;
    }

    if (action === 'toggle') {
      sources[index] = { ...sources[index], enabled: sources[index].enabled === false };
      return sources;
    }

    if (action === 'up' && index > 0) {
      [sources[index - 1], sources[index]] = [sources[index], sources[index - 1]];
      return sources;
    }

    if (action === 'down' && index < sources.length - 1) {
      [sources[index], sources[index + 1]] = [sources[index + 1], sources[index]];
      return sources;
    }

    if (action === 'remove') {
      return sources.filter((_, itemIndex) => itemIndex !== index);
    }

    return sources;
  }, '高级源配置已更新');

  if (action === 'edit') {
    setPremiumStatus('success', '已载入高级源表单');
  }
});

document.getElementById('save-premium')?.addEventListener('click', async () => {
  let premiumSources;
  try {
    premiumSources = parsePremiumSources();
  } catch (error) {
    setPremiumStatus('error', error instanceof Error ? error.message : 'JSON 格式错误');
    return;
  }

  const latestResponse = await fetch('/api/user/data?key=settings');
  const latestData = await latestResponse.json().catch(() => ({}));
  if (!latestResponse.ok) {
    setPremiumStatus('error', latestData.error || '读取当前设置失败');
    return;
  }

  const nextSettings = latestData.data && typeof latestData.data === 'object' ? latestData.data : {};
  nextSettings.premiumSources = premiumSources;

  const response = await fetch('/api/user/data', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: 'settings', value: nextSettings }),
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    setPremiumStatus('error', data.error || '保存失败');
    return;
  }

  applyPremiumSnapshot(premiumSources, 'success', 'Premium Sources 已保存');
});

applyPremiumSnapshot(Array.isArray(initialPremium) ? initialPremium : []);
resetPremiumSourceForm();
"#;

pub(super) const PAGE_STYLE: &str = r#"
:root {
  color-scheme: dark;
  --bg: #08111f;
  --panel: rgba(12, 24, 43, 0.9);
  --panel-soft: rgba(17, 33, 59, 0.75);
  --border: rgba(148, 163, 184, 0.2);
  --text: #e5eefc;
  --muted: #9eb0cf;
  --primary: #4f8cff;
  --primary-soft: rgba(79, 140, 255, 0.14);
  --danger: #ef6b7b;
  --success: #4ade80;
  --shadow: 0 20px 50px rgba(3, 8, 20, 0.32);
  font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

* { box-sizing: border-box; }
body {
  margin: 0;
  min-height: 100vh;
  color: var(--text);
  background:
    radial-gradient(circle at top, rgba(79, 140, 255, 0.18), transparent 32%),
    linear-gradient(180deg, #08111f 0%, #09192d 100%);
}
body.app-locked {
  overflow: hidden;
}
a { color: inherit; text-decoration: none; }
button, input, textarea, select {
  font: inherit;
}
.shell {
  width: min(1180px, calc(100% - 32px));
  margin: 0 auto;
  padding: 18px 0 40px;
}
.back-to-top {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 40;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 44px;
  padding: 0 16px;
  border: 1px solid rgba(79, 140, 255, 0.38);
  border-radius: 999px;
  background: rgba(8, 17, 31, 0.88);
  color: var(--text);
  box-shadow: 0 16px 32px rgba(3, 8, 20, 0.32);
  cursor: pointer;
  opacity: 0;
  pointer-events: none;
  transform: translateY(12px);
  transition: opacity 180ms ease, transform 180ms ease, border-color 180ms ease;
}
.back-to-top.visible {
  opacity: 1;
  pointer-events: auto;
  transform: translateY(0);
}
.back-to-top:hover {
  border-color: rgba(79, 140, 255, 0.62);
}
.page-content, .stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.grid {
  display: grid;
  gap: 14px;
}
.two-col {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.home-hero-grid {
  grid-template-columns: minmax(0, 1.35fr) minmax(280px, 0.65fr);
  gap: 12px;
}
.home-hero-stack {
  gap: 12px;
}
.home-secondary-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.home-library-layout {
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(300px, 0.9fr);
}
.align-start { align-items: start; }
.card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  box-shadow: 0 14px 36px rgba(2, 6, 23, 0.18);
  backdrop-filter: blur(12px);
}
.collapsible-card {
  padding: 0;
  overflow: hidden;
}
.inset-card {
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.72);
}
.collapsible-summary {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px;
  cursor: pointer;
  list-style: none;
}
.collapsible-summary::-webkit-details-marker {
  display: none;
}
.collapsible-summary::after {
  content: '展开';
  flex-shrink: 0;
  align-self: center;
  min-height: 34px;
  padding: 0 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--panel-soft);
  color: var(--muted);
  display: inline-flex;
  align-items: center;
}
.collapsible-card[open] .collapsible-summary {
  border-bottom: 1px solid var(--border);
}
.collapsible-card[open] .collapsible-summary::after {
  content: '收起';
}
.collapsible-content {
  padding: 16px 20px 20px;
}
.compact-card { padding: 16px 18px; }
.hero-card {
  gap: 12px;
  padding: 18px 20px;
}
.player-page-header,
.premium-overview-card,
.premium-search-shell {
  gap: 10px;
}
.player-header-row,
.player-info-row {
  align-items: center;
  gap: 4px;
}
.player-title-stack {
  flex: 1;
  min-width: 0;
  gap: 1px;
}
.player-title-heading {
  margin: 0;
  font-size: clamp(15px, 1.25vw, 18px);
  line-height: 1.12;
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  text-overflow: ellipsis;
}
.player-inline-meta {
  font-size: 10px;
  line-height: 1.1;
}
.player-title-actions {
  justify-content: flex-end;
  gap: 6px;
}
.player-info-row {
  padding-top: 2px;
}
.player-secondary-column {
  gap: 14px;
}
.player-layout > .card:first-child {
  padding: 8px 10px;
}
.player-back-link {
  min-height: 28px;
  padding: 0 9px;
  font-size: 11px;
}
.player-layout > .card:first-child .player-control-row {
  gap: 6px;
}
.player-layout > .card:first-child .player-control-row .button {
  min-height: 32px;
  padding: 0 9px;
  font-size: 11px;
}
.player-layout > .card:first-child .player-control-row .chip {
  min-height: 24px;
  padding: 0 8px;
  font-size: 10px;
}
.player-layout > .card:first-child .inline-control {
  min-height: 32px;
}
.player-tools-card {
  margin-top: 2px;
}
.player-secondary-controls .button {
  min-height: 32px;
  padding: 0 10px;
}
.player-navigation-card .collapsible-summary,
.player-metadata-card .collapsible-summary,
.player-library-card .collapsible-summary,
.player-tools-card .collapsible-summary {
  padding: 16px 18px;
}
.player-navigation-card .collapsible-content,
.player-metadata-card .collapsible-content,
.player-library-card .collapsible-content,
.player-tools-card .collapsible-content {
  padding: 16px 18px 18px;
}
.premium-source-card .collapsible-summary,
.premium-tags-card .collapsible-summary,
.premium-history-card .collapsible-summary,
.premium-library-preview-card .collapsible-summary {
  padding: 18px 20px;
}
.premium-source-card .collapsible-content,
.premium-tags-card .collapsible-content,
.premium-history-card .collapsible-content,
.premium-library-preview-card .collapsible-content {
  padding: 16px 20px 20px;
}
.premium-source-card .code-block {
  max-height: 220px;
}
.library-preview-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.home-search-card {
  gap: 8px;
}
.compact-hero-card {
  gap: 8px;
  padding: 12px 14px;
}
.compact-hero-bar {
  align-items: center;
}
.compact-hero-actions {
  justify-content: flex-end;
}
.settings-overview-card .compact-card-body {
  gap: 2px;
}
.settings-overview-card .chip-list {
  gap: 8px;
}
.settings-overview-card .button {
  min-height: 30px;
  padding: 0 10px;
  font-size: 12px;
}
.settings-overview-card .collapsible-summary {
  padding: 14px 16px;
}
.settings-overview-card .collapsible-content {
  padding: 14px 16px 16px;
}
.settings-overview-card .code-block {
  max-height: 140px;
}
.premium-sync-card {
  padding: 12px 14px;
}
.compact-search-form {
  gap: 8px;
}
.compact-search-field {
  gap: 0;
}
.compact-search-field > span {
  display: none;
}
.compact-search-actions {
  gap: 6px;
}
.compact-status-row {
  align-items: center;
}
.compact-status-row .status {
  flex: 1;
  min-width: 0;
  font-size: 13px;
}
.premium-search-status-row {
  align-items: flex-start;
  padding-top: 2px;
  border-top: 1px solid var(--border);
}
.premium-search-status-row .compact-chip-list {
  flex: 0 1 auto;
}
.premium-search-status-row .compact-hero-actions {
  flex: 1 1 360px;
  justify-content: flex-end;
}
.premium-search-status-row .status {
  flex: 1 1 180px;
  min-width: 0;
  font-size: 12px;
}
.compact-meta-chips {
  gap: 6px;
  justify-content: flex-end;
}
.compact-meta-chips .chip {
  min-height: 26px;
  padding: 0 10px;
  font-size: 11px;
}
.results-section-heading {
  display: flex;
  align-items: center;
  min-height: 32px;
}
.results-section-heading h2 {
  font-size: 18px;
}
.search-results-shell,
.premium-search-shell,
.premium-overview-card {
  gap: 8px;
  padding: 12px 14px;
}
.premium-overview-card .compact-chip-list {
  flex: 1;
  min-width: 0;
}
.premium-overview-card .compact-chip-list .chip {
  max-width: 100%;
}
.hero-card .chip {
  min-height: 30px;
  padding: 0 12px;
  font-size: 12px;
}
.search-actions {
  gap: 10px;
}
.search-actions .button {
  min-height: 36px;
  padding: 0 12px;
  font-size: 13px;
}
.hero-heading-row {
  align-items: center;
}
.hero-subtitle {
  margin: 0;
}
.compact-inline-panel {
  gap: 10px;
  padding-top: 6px;
  border-top: 1px solid var(--border);
}
.home-tools-card .collapsible-summary {
  padding: 18px 20px;
}
.home-tools-card .collapsible-content {
  padding: 16px 20px 20px;
}
.home-tools-card .inset-card {
  border-color: rgba(148, 163, 184, 0.16);
  background: rgba(15, 23, 42, 0.52);
  box-shadow: none;
}
.home-tools-card .inset-card .result-card,
.home-tools-card .inset-card .saved-item {
  background: rgba(255, 255, 255, 0.02);
}
.gap-xs {
  gap: 6px;
}
.hero-card h1, .brand-title, h1, h2 {
  margin: 0;
}
.topbar {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
  margin-bottom: 10px;
}
.shell-topbar {
  align-items: center;
  padding: 12px 16px;
  gap: 12px;
}
.shell-topbar-media {
  padding: 6px 10px;
  gap: 8px;
}
.shell-topbar-minimal {
  justify-content: center;
  padding: 0;
  margin-bottom: 4px;
  background: transparent;
  border: 0;
  box-shadow: none;
}
.shell-login {
  max-width: 720px;
}
.shell-brand {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 34px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid rgba(79, 140, 255, 0.22);
  background: rgba(79, 140, 255, 0.08);
  color: var(--text);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.16em;
  text-decoration: none;
}
.shell-nav-links {
  flex: 1;
  min-width: 0;
  justify-content: flex-end;
  flex-wrap: nowrap;
  overflow-x: auto;
  scrollbar-width: none;
}
.shell-nav-links::-webkit-scrollbar {
  display: none;
}
.shell-nav-links .button {
  min-height: 32px;
  padding: 0 11px;
  font-size: 12px;
  white-space: nowrap;
}
.shell-topbar-media .shell-brand {
  min-height: 28px;
  padding: 0 9px;
  font-size: 10px;
}
.shell-topbar-media .shell-nav-links .button {
  min-height: 28px;
  padding: 0 9px;
  font-size: 10px;
}
.shell-topbar-media .shell-nav-links {
  gap: 6px;
}
.premium-search-shell .results-section-heading h2 {
  font-size: 17px;
}
.premium-search-shell .search-input-shell input {
  min-height: 44px;
}
.premium-search-shell .compact-search-actions .button {
  min-height: 36px;
}
.login-page-content {
  max-width: 420px;
  margin: 0 auto;
  width: 100%;
}
.shell-user {
  flex: 0 0 auto;
  white-space: nowrap;
  font-size: 12px;
}
.preview-card {
  border-style: dashed;
  background: rgba(255, 255, 255, 0.02);
}
.preview-card .eyebrow {
  font-size: 10px;
}
.preview-card p {
  margin: 0;
}
.eyebrow {
  display: inline-flex;
  font-size: 12px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: #8fb5ff;
}
.muted {
  color: var(--muted);
  margin: 0;
}
.tiny { font-size: 12px; }
.row {
  display: flex;
  align-items: center;
}
.align-controls {
  align-items: center;
}
.wrap { flex-wrap: wrap; }
.gap-sm { gap: 12px; }
.space-between { justify-content: space-between; }
.chip-list {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(184px, 1fr));
  gap: 12px;
}
.compact-results-grid {
  grid-template-columns: repeat(auto-fit, minmax(156px, 1fr));
}
.discovery-results-grid {
  grid-template-columns: repeat(auto-fit, minmax(148px, 1fr));
}
.premium-results-grid {
  grid-template-columns: repeat(auto-fit, minmax(144px, 1fr));
}
.player-layout {
  grid-template-columns: minmax(0, 1.3fr) minmax(320px, 0.9fr);
}
.player-control-row {
  gap: 8px;
}
.player-control-row .button {
  min-height: 36px;
  padding: 0 12px;
  font-size: 13px;
}
.player-control-row .inline-control {
  min-height: 36px;
  padding: 0 2px;
}
.player-control-row select {
  min-width: 92px;
}
.detail-layout {
  grid-template-columns: minmax(0, 1.3fr) minmax(320px, 0.9fr);
}
.saved-list {
  display: grid;
  gap: 12px;
}
#settings-source-list,
#premium-source-list,
#settings-subscription-list {
  max-height: 46vh;
  overflow: auto;
  padding-right: 4px;
  scrollbar-gutter: stable;
}
.source-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.source-item-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 10px;
  color: var(--muted);
  font-size: 12px;
  word-break: break-all;
}
.source-item-url {
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  max-width: min(100%, 420px);
}
.compact-chip-list {
  gap: 8px;
}
.compact-chip-list .chip {
  min-height: 26px;
  padding: 0 10px;
  font-size: 12px;
}
.source-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.source-item .button.button-small {
  min-height: 30px;
  padding: 0 9px;
  font-size: 11px;
}
.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.segmented-control {
  display: inline-flex;
  align-items: center;
  padding: 4px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.03);
  gap: 6px;
}
.segment {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 36px;
  padding: 0 14px;
  border-radius: 999px;
  border: 0;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  transition: 160ms ease;
}
.segment.active {
  background: var(--primary-soft);
  color: var(--text);
}
.tag-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 40px;
  padding: 0 16px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.03);
  color: var(--text);
  cursor: pointer;
  transition: 160ms ease;
}
.tag-chip.active {
  background: var(--primary-soft);
  border-color: rgba(79, 140, 255, 0.45);
}
.tag-chip:hover {
  transform: translateY(-1px);
  border-color: rgba(148, 163, 184, 0.36);
}
.user-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}
.user-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.03);
}
.saved-item, .result-card, .episode-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.025);
  transition: 160ms ease;
}
.library-item {
  gap: 12px;
}
.library-item-main {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}
.home-library-card {
  position: sticky;
  top: 20px;
}
.drawer-open {
  overflow: hidden;
}
.side-drawer-overlay {
  position: fixed;
  inset: 0;
  z-index: 1100;
  display: flex;
  justify-content: flex-end;
  padding: 20px;
  background: rgba(2, 6, 23, 0.58);
  backdrop-filter: blur(8px);
}
.side-drawer {
  width: min(420px, 100%);
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: 20px;
  background: rgba(12, 24, 43, 0.96);
  box-shadow: 0 24px 80px rgba(2, 6, 23, 0.45);
}
.side-dock {
  position: fixed;
  bottom: 88px;
  z-index: 60;
  display: flex;
}
.side-dock-left {
  left: 20px;
}
.side-dock-right {
  right: 20px;
}
.side-dock-button {
  min-height: 42px;
  padding: 0 16px;
  border-radius: 999px;
  border: 1px solid rgba(79, 140, 255, 0.32);
  background: rgba(8, 17, 31, 0.88);
  color: var(--text);
  box-shadow: 0 16px 32px rgba(3, 8, 20, 0.32);
  cursor: pointer;
}
.side-dock-button:hover {
  border-color: rgba(79, 140, 255, 0.62);
}
.library-entry {
  gap: 14px;
}
.library-entry.selected {
  border-color: rgba(79, 140, 255, 0.62);
  background: rgba(79, 140, 255, 0.14);
}
.library-actions {
  align-items: center;
}
.active-saved-item {
  border-color: rgba(79, 140, 255, 0.55);
  background: rgba(79, 140, 255, 0.12);
}
.result-card-button {
  width: 100%;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.result-card strong,
.result-card-button strong {
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-height: 1.3;
  font-size: 12px;
}
.result-card p,
.result-card-button p {
  margin: 0;
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-height: 1.35;
  font-size: 10.5px;
}
.result-card .chip,
.result-card-button .chip {
  min-height: 22px;
  padding: 0 7px;
  font-size: 10px;
}
.result-card .row,
.result-card-button .row {
  gap: 6px;
}
.video-shell {
  position: relative;
  overflow: hidden;
  border-radius: 20px;
  border: 1px solid var(--border);
  background: rgba(3, 8, 20, 0.5);
}
.video-shell.is-web-fullscreen {
  position: fixed;
  inset: 0;
  z-index: 1200;
  border-radius: 0;
  border: none;
  background: #000;
}
.player-web-fullscreen {
  overflow: hidden;
}
.player-video {
  width: 100%;
  min-height: 280px;
  background: #000;
  display: block;
}
.video-shell.is-web-fullscreen .player-video {
  width: 100vw;
  height: 100vh;
  min-height: 100vh;
}
.video-shell.is-web-fullscreen.force-landscape {
  width: 100vh;
  width: 100dvh;
  height: 100vw;
  top: 50%;
  left: 50%;
  right: auto;
  bottom: auto;
  transform: translate(-50%, -50%) rotate(90deg);
}
.video-shell.is-web-fullscreen.force-landscape .player-video {
  width: 100%;
  height: 100%;
  min-height: 100%;
}
.skip-indicator {
  position: absolute;
  top: 50%;
  z-index: 3;
  min-width: 96px;
  padding: 14px 18px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(5, 10, 20, 0.72);
  color: #fff;
  font-size: 24px;
  font-weight: 700;
  text-align: center;
  transform: translateY(-50%) scale(0.92);
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.34);
  backdrop-filter: blur(14px);
  opacity: 0;
  pointer-events: none;
  transition: opacity 180ms ease, transform 180ms ease;
}
.skip-indicator.visible {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}
.skip-indicator-backward {
  left: 24px;
}
.skip-indicator-forward {
  right: 24px;
}
.mode-badge {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 12px;
  color: white;
}
.mode-badge.direct {
  background: rgba(34, 197, 94, 0.85);
}
.mode-badge.proxy {
  background: rgba(249, 115, 22, 0.88);
}
.hidden {
  display: none;
}
.bootstrap-overlay {
  position: fixed;
  inset: 0;
  z-index: 999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(2, 6, 23, 0.68);
  backdrop-filter: blur(10px);
}
.bootstrap-card {
  display: grid;
  gap: 12px;
  min-width: min(320px, 100%);
  max-width: 420px;
  padding: 24px;
  border-radius: 20px;
  border: 1px solid var(--border);
  background: rgba(15, 23, 42, 0.92);
  box-shadow: 0 24px 80px rgba(2, 6, 23, 0.45);
  text-align: center;
}
.access-gate-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(2, 6, 23, 0.78);
  backdrop-filter: blur(14px);
}
.bootstrap-overlay.hidden,
.access-gate-overlay.hidden {
  display: none;
}
.access-gate-card {
  display: grid;
  gap: 14px;
  width: min(420px, 100%);
  padding: 24px;
  border-radius: 22px;
  border: 1px solid var(--border);
  background: rgba(15, 23, 42, 0.94);
  box-shadow: 0 30px 90px rgba(2, 6, 23, 0.5);
}
.bootstrap-spinner {
  width: 36px;
  height: 36px;
  margin: 0 auto;
  border-radius: 999px;
  border: 3px solid rgba(148, 163, 184, 0.28);
  border-top-color: var(--primary);
  animation: bootstrap-spin 0.8s linear infinite;
}
@keyframes bootstrap-spin {
  to {
    transform: rotate(360deg);
  }
}
.result-card:hover, .saved-item:hover, .episode-item:hover {
  border-color: rgba(148, 163, 184, 0.36);
  background: rgba(255, 255, 255, 0.035);
}
.result-poster, .detail-poster {
  width: 100%;
  object-fit: cover;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.05);
}
.result-poster {
  aspect-ratio: 2 / 3;
}
.detail-poster {
  aspect-ratio: 2 / 3;
  max-width: 240px;
}
.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 48px;
}
.compact-card-body {
  gap: 6px;
}
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  padding: 18px;
  border-radius: 18px;
  border: 1px dashed var(--border);
  background: rgba(255, 255, 255, 0.02);
  color: var(--muted);
  text-align: center;
}
.episodes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}
.episode-item {
  appearance: none;
  color: var(--text);
  text-align: left;
  cursor: pointer;
}
.episode-item.active {
  border-color: rgba(79, 140, 255, 0.55);
  background: rgba(79, 140, 255, 0.16);
}
.detail-summary, .detail-summary-grid {
  display: grid;
  gap: 20px;
}
.detail-summary-grid {
  grid-template-columns: minmax(180px, 240px) minmax(0, 1fr);
  align-items: start;
}
.metadata-list {
  gap: 10px;
}
.chip {
  display: inline-flex;
  align-items: center;
  min-height: 34px;
  padding: 0 14px;
  border-radius: 999px;
  background: var(--panel-soft);
  border: 1px solid var(--border);
  color: var(--text);
}
.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 42px;
  padding: 0 16px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.02);
  color: var(--text);
  cursor: pointer;
  transition: 160ms ease;
}
.button:hover {
  transform: translateY(-1px);
  border-color: rgba(148, 163, 184, 0.36);
}
.button-small {
  min-height: 34px;
  padding: 0 12px;
  font-size: 13px;
}
.button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  transform: none;
}
.button.primary {
  background: var(--primary);
  border-color: transparent;
  color: white;
}
.button.active {
  background: var(--primary-soft);
  border-color: rgba(79, 140, 255, 0.45);
}
.button.danger {
  color: #ffd9de;
  border-color: rgba(239, 107, 123, 0.35);
  background: rgba(239, 107, 123, 0.1);
}
.form-grid {
  gap: 14px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.search-input-shell {
  position: relative;
}
.search-history-dropdown {
  position: absolute;
  top: calc(100% + 10px);
  left: 0;
  right: 0;
  z-index: 80;
  display: grid;
  gap: 8px;
  padding: 14px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: rgba(12, 24, 43, 0.96);
  box-shadow: 0 24px 80px rgba(2, 6, 23, 0.45);
  backdrop-filter: blur(18px);
}
.search-history-dropdown.hidden {
  display: none !important;
  pointer-events: none;
}
.search-history-dropdown-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.search-history-dropdown-list {
  display: grid;
  gap: 8px;
  max-height: 320px;
  overflow: auto;
}
.search-history-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.03);
}
.search-history-dropdown-item.active {
  border-color: rgba(79, 140, 255, 0.45);
  background: rgba(79, 140, 255, 0.12);
}
.search-history-dropdown-main {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
  text-align: left;
}
.inline-control {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--muted);
}
.inline-control select {
  width: auto;
  min-width: 148px;
}
.field span {
  color: var(--muted);
  font-size: 14px;
}
.checkbox-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text);
}
.checkbox-row input {
  width: 18px;
  height: 18px;
}
.checkbox-row.is-disabled {
  opacity: 0.6;
}
input, textarea, select {
  width: 100%;
  padding: 12px 14px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: rgba(3, 8, 20, 0.28);
  color: var(--text);
}
textarea {
  resize: vertical;
  min-height: 180px;
}
.code-input-medium {
  min-height: 140px;
  max-height: 32vh;
}
.code-input-large {
  min-height: 260px;
  max-height: 56vh;
}
.code-input, .code-block {
  font-family: 'SFMono-Regular', ui-monospace, Menlo, Consolas, monospace;
  font-size: 13px;
  line-height: 1.6;
}
.compact-code-input {
  min-height: 120px;
}
.code-block {
  margin: 0;
  padding: 14px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: rgba(3, 8, 20, 0.28);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 180px;
  overflow: auto;
}
.status {
  min-height: 20px;
  margin: 0;
}
.status.success { color: var(--success); }
.status.error { color: #ff93a0; }

@media (max-width: 860px) {
  .two-col { grid-template-columns: 1fr; }
  .home-hero-grid { grid-template-columns: 1fr; }
  .home-secondary-grid { grid-template-columns: 1fr; }
  .home-hero-grid > .compact-card { display: none; }
  .home-tools-card .collapsible-summary,
  .home-tools-card .collapsible-content {
    padding-left: 16px;
    padding-right: 16px;
  }
  .home-library-layout { grid-template-columns: 1fr; }
  .library-preview-grid { grid-template-columns: 1fr; }
  .player-layout { grid-template-columns: 1fr; }
  .player-title-actions { justify-content: flex-start; }
  .player-control-row .button {
    min-height: 34px;
    padding: 0 10px;
    font-size: 12px;
  }
  .detail-layout { grid-template-columns: 1fr; }
  .detail-summary-grid { grid-template-columns: 1fr; }
  .topbar { flex-direction: column; }
  .shell-topbar-media.topbar {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  .shell-topbar { padding: 10px 12px; }
  .shell-nav-links { justify-content: flex-start; }
  .shell-topbar-media .shell-nav-links {
    justify-content: flex-end;
  }
  .shell-user { display: none; }
  .card { padding: 18px; }
  .hero-card { padding: 16px 18px; }
  .compact-hero-card,
  .search-results-shell,
  .premium-search-shell,
  .premium-overview-card {
    padding: 12px 14px;
  }
  .player-title-inline h1 { font-size: 22px; }
  .search-actions { gap: 8px; }
  .search-actions .button {
    min-height: 36px;
    padding: 0 12px;
    font-size: 12px;
  }
  .compact-hero-bar {
    align-items: flex-start;
  }
  .premium-overview-card .compact-status-row {
    align-items: flex-start;
  }
  .premium-overview-card .compact-status-row .status {
    flex-basis: 100%;
  }
  .side-dock {
    display: none;
  }
  .compact-hero-actions {
    width: 100%;
    justify-content: flex-start;
  }
  .compact-status-row {
    align-items: flex-start;
  }
  .collapsible-summary { padding: 18px; }
  .collapsible-content { padding: 16px 18px 18px; }
  .shell { width: min(100% - 24px, 1180px); }
  #settings-source-list,
  #premium-source-list,
  #settings-subscription-list {
    max-height: 42vh;
  }
  .side-dock { bottom: 84px; }
  .side-dock-left { left: 12px; }
  .side-dock-right { right: 12px; }
  .code-input-large { min-height: 220px; max-height: 48vh; }
  .skip-indicator {
    min-width: 80px;
    padding: 12px 14px;
    font-size: 20px;
  }
  .skip-indicator-backward { left: 14px; }
  .skip-indicator-forward { right: 14px; }
}

@media (max-width: 640px) {
  .player-title-inline h1 {
    font-size: 20px;
  }
  .player-navigation-card .collapsible-summary,
  .player-metadata-card .collapsible-summary,
  .player-library-card .collapsible-summary {
    padding: 14px 16px;
  }
  .player-navigation-card .collapsible-content,
  .player-metadata-card .collapsible-content,
  .player-library-card .collapsible-content {
    padding: 14px 16px 16px;
  }
}
"#;
