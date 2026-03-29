use serde_json::Value as JsonValue;

use crate::auth::AuthUser;

use super::super::{
    assets::{LOGIN_SCRIPT, PREMIUM_SETTINGS_SCRIPT, SETTINGS_SCRIPT},
    render::{
        build_rust_detail_url, build_rust_player_url, count_enabled_sources, escape_html,
        escape_script_json, json_array_len, json_string, pretty_json, preview_card,
        render_home_library_entries, render_saved_items_list,
    },
    scripts::{
        ADMIN_SCRIPT, DETAIL_SCRIPT, DISCOVERY_SCRIPT, PLAYER_SCRIPT, PREMIUM_PAGE_SCRIPT,
        SEARCH_SCRIPT,
    },
};
use super::shared::MediaRequest;

fn premium_enabled_source_names(premium_sources: &JsonValue) -> Vec<String> {
    premium_sources
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| {
            item.get("enabled")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false)
        })
        .filter_map(|item| item.get("name").and_then(JsonValue::as_str))
        .map(str::to_string)
        .collect()
}

pub(super) fn render_index_body(
    auth_user: &AuthUser,
    initial_query: &str,
    settings_value: &JsonValue,
    history_value: &JsonValue,
    favorites_value: &JsonValue,
) -> String {
    let favorites_panel_action = if auth_user.disable_premium {
        r#"<span class="chip">Premium 已禁用</span>"#.to_string()
    } else {
        r#"<a class="button" href="/premium">前往 Premium</a>"#.to_string()
    };
    let home_premium_entry = if auth_user.disable_premium {
        String::new()
    } else {
        r#"<a class="button button-small" href="/premium">Premium</a>"#.to_string()
    };

    format!(
        r#"
<section class="stack home-hero-stack">
  <article class="card stack hero-card home-search-card compact-hero-card">
    <form id="search-form" class="stack form-grid compact-search-form">
      <label class="field compact-search-field">
        <div class="search-input-shell">
          <input id="search-query" name="q" placeholder="输入影片名称，例如：仙逆" value="{}" required autocomplete="off" aria-autocomplete="list" aria-expanded="false" aria-controls="search-history-dropdown" />
          <div id="search-history-dropdown" class="search-history-dropdown hidden" aria-label="搜索历史下拉" role="listbox"></div>
        </div>
      </label>
      <div class="row wrap gap-sm search-actions compact-search-actions">
        <button class="button primary" type="submit">开始搜索</button>
        <button id="clear-search" class="button" type="button">清空结果</button>
      </div>
    </form>
    <div class="row space-between wrap gap-sm compact-status-row">
      <p id="search-status" class="status muted">输入关键词后会直接在下方显示结果。</p>
      <div class="chip-list compact-meta-chips">
        <span class="chip">线路 {}</span>
        <span class="chip">收藏 {}</span>
        <span class="chip">历史 {}</span>
      </div>
    </div>
  </article>
</section>

<section class="card stack search-results-shell">
  <div class="row space-between wrap gap-sm">
    <div class="results-section-heading">
      <h2>搜索结果</h2>
    </div>
    <div class="row wrap gap-sm align-controls">
      <div id="search-display-toggle" class="segmented-control">
        <button class="segment" type="button" data-display-mode="normal">普通</button>
        <button class="segment" type="button" data-display-mode="grouped">分组</button>
      </div>
      <label class="inline-control">
        <span>排序</span>
        <select id="search-sort-select">
          <option value="default">默认</option>
          <option value="relevance">相关性</option>
          <option value="latency-asc">延迟低到高</option>
          <option value="date-desc">年份新到旧</option>
          <option value="date-asc">年份旧到新</option>
          <option value="rating-desc">评分高到低</option>
          <option value="name-asc">名称 A-Z</option>
          <option value="name-desc">名称 Z-A</option>
        </select>
      </label>
      <span id="search-progress" class="chip">等待搜索</span>
      <span id="search-total" class="chip">结果 0</span>
    </div>
  </div>
  <div id="search-filter-toolbar" class="stack hidden" aria-label="搜索结果筛选">
    <div class="row space-between wrap gap-sm">
      <strong>结果筛选</strong>
      <div class="row wrap gap-sm">
        <span id="search-filter-summary" class="chip">未启用筛选</span>
        <button id="clear-search-filters" class="button button-small" type="button">清空筛选</button>
      </div>
    </div>
    <div class="stack gap-sm">
      <div class="stack gap-sm">
        <span class="muted tiny">来源徽标</span>
        <div id="search-source-badges" class="chip-list">
          <span class="chip">等待搜索结果</span>
        </div>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">类型徽标</span>
        <div id="search-type-badges" class="chip-list">
          <span class="chip">等待搜索结果</span>
        </div>
      </div>
    </div>
  </div>
  <div id="search-results" class="results-grid empty-state">请输入关键词开始搜索。</div>
</section>

<div id="home-library-overlay" class="side-drawer-overlay hidden" aria-hidden="true">
  <aside id="home-library-drawer" class="side-drawer" role="dialog" aria-modal="true" aria-label="首页收藏和历史侧栏" tabindex="-1">
    <div class="row space-between wrap gap-sm">
      <div class="stack compact-card-body">
        <span class="eyebrow">快速访问</span>
        <h2>首页侧栏</h2>
        <p class="muted">在浮层里切换历史和收藏，快速播放、看详情或回填同名搜索。</p>
      </div>
      <div class="row wrap gap-sm">
        <button class="button button-small" type="button" data-home-library-selection-toggle>批量选择</button>
        <button class="button button-small" type="button" data-home-library-select-all>全选当前列表</button>
        <button class="button button-small danger" type="button" data-home-library-remove-selected>移除已选</button>
        <button class="button button-small" type="button" data-home-library-undo disabled>撤销上次移除</button>
        <button class="button button-small" type="button" data-home-library-copy>复制当前列表</button>
        <button class="button button-small" type="button" data-home-library-share>复制分享包</button>
        <button class="button button-small" type="button" data-home-library-share-link>复制分享链接</button>
        <button class="button button-small" type="button" data-home-library-share-link-merge>复制合并分享链接</button>
        <button class="button button-small" type="button" data-home-library-share-native>系统分享当前列表</button>
        <button class="button button-small" type="button" data-home-library-snapshot-save>保存快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-rename>重命名快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-duplicate>克隆快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-restore>恢复快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-merge>合并快照</button>
        <button class="button button-small danger" type="button" data-home-library-snapshot-delete>删除快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-export>导出快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-share>复制快照分享包</button>
        <button class="button button-small" type="button" data-home-library-snapshot-share-link>复制快照分享链接</button>
        <button class="button button-small" type="button" data-home-library-snapshot-share-link-merge>复制快照合并链接</button>
        <button class="button button-small" type="button" data-home-library-snapshot-import>导入快照</button>
        <button class="button button-small" type="button" data-home-library-snapshot-import-merge>合并导入快照</button>
        <button class="button button-small" type="button" data-home-library-export>导出当前列表</button>
        <button class="button button-small" type="button" data-home-library-import>导入当前列表</button>
        <button class="button button-small" type="button" data-home-library-import-merge>合并导入</button>
        <button class="button button-small" type="button" data-home-library-import-clipboard>剪贴板导入</button>
        <button class="button button-small" type="button" data-home-library-import-clipboard-merge>剪贴板合并</button>
        <button class="button button-small" type="button" data-home-library-dedupe>当前列表去重</button>
        <button class="button button-small" type="button" data-home-library-clear>清空当前列表</button>
        <button id="close-home-library-drawer" class="button" type="button">关闭</button>
      </div>
    </div>
    <div class="row space-between wrap gap-sm">
      <div id="home-library-drawer-toggle" class="segmented-control" role="tablist" aria-label="首页侧栏切换">
        <button class="segment active" type="button" role="tab" aria-selected="true" aria-controls="home-library-drawer-panel-history" id="home-library-drawer-tab-history" data-library-drawer-tab="history">历史</button>
        <button class="segment" type="button" role="tab" aria-selected="false" aria-controls="home-library-drawer-panel-favorites" id="home-library-drawer-tab-favorites" data-library-drawer-tab="favorites">收藏</button>
      </div>
      <span class="chip" data-home-library-status>历史面板</span>
    </div>
    <div class="row wrap gap-sm">
      <input class="input compact-input" type="search" placeholder="筛选当前列表" data-home-library-filter />
      <button class="button button-small" type="button" data-home-library-filter-clear>清空筛选</button>
      <label class="inline-control">
        <span>排序</span>
        <select class="compact-input" data-home-library-sort>
          <option value="recent-desc">最近添加</option>
          <option value="recent-asc">最早添加</option>
          <option value="title-asc">标题 A-Z</option>
          <option value="title-desc">标题 Z-A</option>
          <option value="source-asc">来源 A-Z</option>
          <option value="source-desc">来源 Z-A</option>
        </select>
      </label>
    </div>
    <div id="home-library-drawer-panel-history" data-library-drawer-panel="history" role="tabpanel" aria-labelledby="home-library-drawer-tab-history">
      {}
    </div>
    <div id="home-library-drawer-panel-favorites" data-library-drawer-panel="favorites" class="hidden" role="tabpanel" aria-labelledby="home-library-drawer-tab-favorites" hidden>
      {}
    </div>
  </aside>
</div>

<details class="card collapsible-card home-shortcuts-card">
  <summary class="collapsible-summary">
    <div>
      <h2>快捷入口</h2>
      <p class="muted">常用入口默认折叠，不占首屏主位。</p>
    </div>
    <div class="chip-list">
      <span class="chip">线路：{}</span>
    </div>
  </summary>
  <div class="collapsible-content">
    <div class="row wrap gap-sm">
      <button class="button button-small" type="button" data-quick-query="庆余年">庆余年</button>
      <button class="button button-small" type="button" data-quick-query="仙逆">仙逆</button>
      <button class="button button-small" type="button" data-quick-query="狂飙">狂飙</button>
      <button id="open-home-library-drawer" class="button button-small" type="button" data-open-home-library aria-controls="home-library-drawer" aria-expanded="false">侧栏</button>
      <a class="button button-small" href="/settings">设置</a>
      {}
      {}
    </div>
  </div>
</details>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>最近记录</h2>
      <p class="muted">历史和收藏摘要。</p>
    </div>
    <div class="row wrap gap-sm">
      <span class="chip">历史 {}</span>
      <span class="chip">收藏 {}</span>
    </div>
  </summary>
  <div class="collapsible-content">
    <section class="grid two-col align-start home-secondary-grid">
      <article class="card stack compact-card">
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>最近历史</h2>
            <p class="muted">最近同步的播放记录。</p>
          </div>
          <a class="button" href="/settings">前往设置</a>
        </div>
        {}
      </article>
      <article class="card stack compact-card">
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>最近收藏</h2>
            <p class="muted">最近同步的收藏。</p>
          </div>
          {}
        </div>
        {}
      </article>
    </section>
  </div>
</details>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div class="row space-between wrap gap-sm">
      <div>
        <h2>快速访问</h2>
        <p class="muted">历史、收藏和快照管理。</p>
      </div>
      <span id="home-library-summary" class="chip">历史 {} / 收藏 {}</span>
    </div>
  </summary>
  <div class="collapsible-content stack">
    <div class="row wrap gap-sm">
      <button class="button button-small" type="button" data-home-library-selection-toggle>批量选择</button>
      <button class="button button-small" type="button" data-home-library-select-all>全选当前列表</button>
      <button class="button button-small danger" type="button" data-home-library-remove-selected>移除已选</button>
      <button class="button button-small" type="button" data-home-library-undo disabled>撤销上次移除</button>
      <button class="button button-small" type="button" data-home-library-copy>复制当前列表</button>
      <button class="button button-small" type="button" data-home-library-share>复制分享包</button>
      <button class="button button-small" type="button" data-home-library-share-link>复制分享链接</button>
      <button class="button button-small" type="button" data-home-library-share-link-merge>复制合并分享链接</button>
      <button class="button button-small" type="button" data-home-library-share-native>系统分享当前列表</button>
      <button class="button button-small" type="button" data-home-library-snapshot-save>保存快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-rename>重命名快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-duplicate>克隆快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-restore>恢复快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-merge>合并快照</button>
      <button class="button button-small danger" type="button" data-home-library-snapshot-delete>删除快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-export>导出快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-share>复制快照分享包</button>
      <button class="button button-small" type="button" data-home-library-snapshot-share-link>复制快照分享链接</button>
      <button class="button button-small" type="button" data-home-library-snapshot-share-link-merge>复制快照合并链接</button>
      <button class="button button-small" type="button" data-home-library-snapshot-import>导入快照</button>
      <button class="button button-small" type="button" data-home-library-snapshot-import-merge>合并导入快照</button>
      <button class="button button-small" type="button" data-home-library-export>导出当前列表</button>
      <button class="button button-small" type="button" data-home-library-import>导入当前列表</button>
      <button class="button button-small" type="button" data-home-library-import-merge>合并导入</button>
      <button class="button button-small" type="button" data-home-library-import-clipboard>剪贴板导入</button>
      <button class="button button-small" type="button" data-home-library-import-clipboard-merge>剪贴板合并</button>
      <button class="button button-small" type="button" data-home-library-dedupe>当前列表去重</button>
      <button class="button button-small" type="button" data-home-library-clear>清空当前列表</button>
    </div>
    <div id="home-library-toggle" class="segmented-control" role="tablist" aria-label="首页快速访问切换">
      <button class="segment active" type="button" role="tab" aria-selected="true" aria-controls="home-library-panel-history" id="home-library-tab-history" data-library-tab="history">历史</button>
      <button class="segment" type="button" role="tab" aria-selected="false" aria-controls="home-library-panel-favorites" id="home-library-tab-favorites" data-library-tab="favorites">收藏</button>
    </div>
    <div class="row wrap gap-sm">
      <input class="input compact-input" type="search" placeholder="筛选当前列表" data-home-library-filter />
      <button class="button button-small" type="button" data-home-library-filter-clear>清空筛选</button>
      <label class="inline-control">
        <span>排序</span>
        <select class="compact-input" data-home-library-sort>
          <option value="recent-desc">最近添加</option>
          <option value="recent-asc">最早添加</option>
          <option value="title-asc">标题 A-Z</option>
          <option value="title-desc">标题 Z-A</option>
          <option value="source-asc">来源 A-Z</option>
          <option value="source-desc">来源 Z-A</option>
        </select>
      </label>
    </div>
    <div id="home-library-panel-history" data-library-panel="history" role="tabpanel" aria-labelledby="home-library-tab-history">
      {}
    </div>
    <div id="home-library-panel-favorites" data-library-panel="favorites" class="hidden" role="tabpanel" aria-labelledby="home-library-tab-favorites" hidden>
      {}
    </div>
  </div>
</details>

<input id="home-library-import-file" type="file" accept="application/json,.json" hidden />
<input id="home-library-snapshot-import-file" type="file" accept="application/json,.json" hidden />

<details class="card collapsible-card home-tools-card">
  <summary class="collapsible-summary">
    <div>
      <h2>更多工具</h2>
      <p class="muted">热门搜索和历史记录放在这里，不占首屏主区。</p>
    </div>
    <div class="row wrap gap-sm">
      <span class="chip">快捷搜索</span>
      <span id="search-history-count" class="chip">历史 0</span>
    </div>
  </summary>
  <div class="collapsible-content">
    <section class="grid two-col align-start home-secondary-grid">
      <article class="card stack inset-card">
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>热门功能</h2>
            <p class="muted">常用搜索入口。</p>
          </div>
          <span class="chip">首页入口</span>
        </div>
        <div class="results-grid compact-results-grid">
          <article class="result-card">
            <div class="stack compact-card-body">
              <strong>热门国漫</strong>
              <p class="muted">一键回填常用关键词。</p>
              <div class="row wrap gap-sm">
                <button class="button button-small" type="button" data-quick-query="仙逆">仙逆</button>
                <button class="button button-small" type="button" data-quick-query="凡人修仙传">凡人修仙传</button>
                <button class="button button-small" type="button" data-quick-query="斗破苍穹">斗破苍穹</button>
              </div>
            </div>
          </article>
          <article class="result-card">
            <div class="stack compact-card-body">
              <strong>热门剧集</strong>
              <p class="muted">直接发起常用搜索。</p>
              <div class="row wrap gap-sm">
                <button class="button button-small" type="button" data-quick-query="庆余年">庆余年</button>
                <button class="button button-small" type="button" data-quick-query="漫长的季节">漫长的季节</button>
                <button class="button button-small" type="button" data-quick-query="狂飙">狂飙</button>
              </div>
            </div>
          </article>
        </div>
      </article>

      <article class="card stack inset-card">
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>搜索历史</h2>
            <p class="muted">最近搜索过的关键词。</p>
          </div>
          <button id="clear-search-history" class="button" type="button">清空历史</button>
        </div>
        <div id="search-history-list" class="saved-list empty-state">当前还没有搜索历史。</div>
      </article>
    </section>
  </div>
</details>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>豆瓣推荐</h2>
      <p class="muted">推荐区默认折叠，避免首页过长。</p>
    </div>
    <div class="row wrap gap-sm">
      <div id="content-type-toggle" class="segmented-control">
        <button class="segment active" type="button" data-content-type="movie">电影</button>
        <button class="segment" type="button" data-content-type="tv">电视剧</button>
      </div>
      <span id="discovery-tag-count" class="chip">标签 0</span>
    </div>
  </summary>
  <div class="collapsible-content stack">
    <div id="discovery-tags" class="tag-cloud empty-state">正在加载豆瓣标签...</div>
    <div id="discovery-results" class="results-grid compact-results-grid discovery-results-grid empty-state">正在加载推荐内容...</div>
  </div>
</details>

<script id="initial-query" type="application/json">{}</script>
<script id="initial-settings" type="application/json">{}</script>
<script id="home-state" type="application/json">{}</script>
<script>{}</script>
"#,
        escape_html(initial_query),
        count_enabled_sources(settings_value),
        json_array_len(favorites_value),
        json_array_len(history_value),
        render_home_library_entries(history_value, "当前没有可快速访问的历史记录。"),
        render_home_library_entries(favorites_value, "当前没有可快速访问的收藏内容。"),
        count_enabled_sources(settings_value),
        home_premium_entry,
        if auth_user.is_admin {
            r#"<a class="button button-small" href="/admin">管理后台</a>"#
        } else {
            r#"<a class="button button-small" href="/login">账户入口</a>"#
        },
        json_array_len(history_value),
        json_array_len(favorites_value),
        render_saved_items_list(history_value, "最近没有同步历史记录", "/player"),
        favorites_panel_action,
        render_saved_items_list(favorites_value, "最近没有同步收藏记录", "/player"),
        json_array_len(history_value),
        json_array_len(favorites_value),
        render_home_library_entries(history_value, "当前没有可快速访问的历史记录。"),
        render_home_library_entries(favorites_value, "当前没有可快速访问的收藏内容。"),
        escape_script_json(&json_string(initial_query)),
        escape_script_json(&json_string(settings_value)),
        escape_script_json(&json_string(&serde_json::json!({
            "history": history_value,
            "favorites": favorites_value,
        }))),
        format!("{}\n{}", SEARCH_SCRIPT, DISCOVERY_SCRIPT),
    )
}

pub(super) fn render_settings_body(
    auth_user: &AuthUser,
    subscription_sources: &str,
    settings_value: &JsonValue,
    has_env_password: bool,
    persist_password: bool,
) -> String {
    let settings_pretty = pretty_json(settings_value);
    let premium_settings_link = if auth_user.disable_premium {
        String::new()
    } else {
        r#"<a class="button button-small" href="/premium/settings">Premium 设置</a>"#.to_string()
    };
    let admin_link = if auth_user.is_admin {
        r#"<a class="button button-small" href="/admin">管理后台</a>"#.to_string()
    } else {
        String::new()
    };

    format!(
        r#"
<section>
  <article class="card stack compact-hero-card settings-overview-card">
    <div class="row space-between wrap gap-sm compact-hero-bar">
      <div class="stack compact-card-body">
        <h2>设置</h2>
        <p class="muted">账户与常用入口。</p>
      </div>
      <div class="row wrap gap-sm compact-hero-actions">
        {}
        {}
      </div>
    </div>
    <div class="chip-list compact-chip-list">
      <span class="chip">用户：{}</span>
      <span class="chip">角色：{}</span>
      <span class="chip">Premium：{}</span>
      <span id="subscription-summary" class="chip">订阅 0</span>
    </div>
    <details class="collapsible-card inset-card">
      <summary class="collapsible-summary">
        <div>
          <h3>快捷入口</h3>
          <p class="muted">系统订阅源和复制入口。</p>
        </div>
      </summary>
      <div class="collapsible-content stack">
        <pre id="subscription-sources" class="code-block">{}</pre>
        <div class="row gap-sm wrap">
          <button id="copy-subscription-sources" class="button" type="button">复制订阅配置</button>
        </div>
      </div>
    </details>
  </article>
</section>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>访问控制</h2>
      <p class="muted">本地访问密码与环境门禁。</p>
    </div>
    <div class="row wrap gap-sm">
      <span id="access-password-count" class="chip">密码 0</span>
      <span class="chip">环境密码：{}</span>
      <span class="chip">持久解锁：{}</span>
    </div>
  </summary>
  <div class="collapsible-content stack">
    <label class="checkbox-row">
      <input id="access-control-enabled" type="checkbox" />
      <span>启用本地访问控制配置</span>
    </label>
    <form id="access-password-form" class="row gap-sm wrap">
      <input id="access-password-input" type="password" class="input flex-1" placeholder="添加本地访问密码..." />
      <button id="access-password-add" class="button" type="submit">添加密码</button>
    </form>
    <div id="access-password-list" class="saved-list empty-state">当前还没有本地访问密码。</div>
  </div>
</details>

<section class="grid two-col align-start">
  <details class="card collapsible-card" open>
    <summary class="collapsible-summary">
      <div>
        <h2>常用设置</h2>
        <p class="muted">优先展示最常改的项。</p>
      </div>
      <div class="row gap-sm wrap">
        <button id="sync-quick-settings" class="button" type="button">从 JSON 读取</button>
        <button id="apply-quick-settings" class="button primary" type="button">应用到 JSON</button>
      </div>
    </summary>
    <div class="collapsible-content stack">
      <div class="grid two-col">
        <label class="field">
          <span>搜索展示</span>
          <select id="quick-search-display-mode">
            <option value="normal">普通</option>
            <option value="grouped">分组</option>
          </select>
        </label>
        <label class="field">
          <span>搜索排序</span>
          <select id="quick-sort-by">
            <option value="default">默认</option>
            <option value="relevance">相关性</option>
            <option value="latency-asc">延迟低到高</option>
            <option value="date-desc">年份新到旧</option>
            <option value="date-asc">年份旧到新</option>
            <option value="rating-desc">评分高到低</option>
            <option value="name-asc">名称 A-Z</option>
            <option value="name-desc">名称 Z-A</option>
          </select>
        </label>
        <label class="field">
          <span>代理模式</span>
          <select id="quick-proxy-mode">
            <option value="retry">失败重试</option>
            <option value="none">始终直连</option>
            <option value="always">始终代理</option>
          </select>
        </label>
        <label class="field">
          <span>全屏方式</span>
          <select id="quick-fullscreen-type">
            <option value="native">原生全屏</option>
            <option value="window">窗口全屏</option>
          </select>
        </label>
      </div>
      <div class="grid two-col">
        <label class="checkbox-row">
          <input id="quick-search-history" type="checkbox" />
          <span>启用搜索历史</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-watch-history" type="checkbox" />
          <span>启用观看历史</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-realtime-latency" type="checkbox" />
          <span>搜索时实时测速</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-remember-scroll" type="checkbox" />
          <span>记住返回滚动位置</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-episode-reverse-order" type="checkbox" />
          <span>反向显示选集顺序</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-auto-next-episode" type="checkbox" />
          <span>自动连播下一集</span>
        </label>
        <label class="checkbox-row">
          <input id="quick-show-mode-indicator" type="checkbox" />
          <span>显示播放器模式角标</span>
        </label>
      </div>
      <div class="grid two-col">
        <label class="checkbox-row">
          <input id="quick-auto-skip-intro" type="checkbox" />
          <span>自动跳过片头</span>
        </label>
        <label class="field">
          <span>片头秒数</span>
          <input id="quick-skip-intro-seconds" type="number" min="0" step="1" value="0" />
        </label>
        <label class="checkbox-row">
          <input id="quick-auto-skip-outro" type="checkbox" />
          <span>自动跳过片尾</span>
        </label>
        <label class="field">
          <span>片尾秒数</span>
          <input id="quick-skip-outro-seconds" type="number" min="0" step="1" value="0" />
        </label>
      </div>
      <div class="grid two-col">
        <label class="field">
          <span>广告过滤模式</span>
          <select id="quick-ad-filter-mode">
            <option value="off">关闭</option>
            <option value="keyword">关键词</option>
            <option value="heuristic">智能</option>
            <option value="aggressive">激进</option>
          </select>
        </label>
        <label class="field">
          <span>广告关键词</span>
          <textarea id="quick-ad-keywords" class="code-input compact-code-input" spellcheck="false" placeholder="每行一个关键词，例如 advert&#10;preroll"></textarea>
        </label>
      </div>
    </div>
  </details>

  <article class="card stack">
    <div class="row space-between wrap gap-sm">
      <div>
        <h2>线路管理</h2>
        <p class="muted">默认先显示常用前几条，展开后再看全部。</p>
      </div>
      <div class="row gap-sm wrap">
        <span id="settings-source-count" class="chip">线路 0 / 0</span>
        <span id="settings-premium-count" class="chip">Premium 0 / 0</span>
      </div>
    </div>
    <div class="row gap-sm wrap">
      <button id="enable-all-sources" class="button" type="button">全部启用</button>
      <button id="disable-all-sources" class="button" type="button">全部禁用</button>
      <button id="restore-default-sources" class="button" type="button">恢复仓库默认</button>
      <button id="toggle-settings-source-limit" class="button" type="button">显示全部</button>
    </div>
    <div class="row gap-sm wrap">
      <input id="settings-source-search" class="input compact-input flex-1" placeholder="搜索线路..." />
    </div>
    <details class="collapsible-card inset-card">
      <summary class="collapsible-summary">
        <div>
          <h3>新增或编辑线路</h3>
          <p class="muted">把长表单默认收起，只有需要增改线路时再展开。</p>
        </div>
        <span class="chip">表单</span>
      </summary>
      <div class="collapsible-content">
        <form id="settings-source-form" class="stack form-grid">
          <div class="grid two-col">
            <label class="field">
              <span>线路 ID</span>
              <input id="settings-source-id" placeholder="例如：heimuer" required />
            </label>
            <label class="field">
              <span>线路名称</span>
              <input id="settings-source-name" placeholder="例如：黑木耳" required />
            </label>
            <label class="field">
              <span>Base URL</span>
              <input id="settings-source-base-url" placeholder="https://example.com" required />
            </label>
            <label class="field">
              <span>优先级</span>
              <input id="settings-source-priority" type="number" min="0" step="1" value="1" />
            </label>
            <label class="field">
              <span>搜索路径</span>
              <input id="settings-source-search-path" placeholder="/api.php/provide/vod/" required />
            </label>
            <label class="field">
              <span>详情路径</span>
              <input id="settings-source-detail-path" placeholder="/api.php/provide/vod/" required />
            </label>
          </div>
          <label class="checkbox-row">
            <input id="settings-source-enabled" type="checkbox" checked />
            <span>启用该线路</span>
          </label>
          <div class="row gap-sm wrap">
            <button id="settings-source-submit" class="button primary" type="submit">新增线路</button>
            <button id="settings-source-cancel" class="button" type="button" disabled>取消编辑</button>
          </div>
        </form>
      </div>
    </details>
    <div id="settings-source-list" class="saved-list empty-state">正在读取线路配置...</div>
  </article>
</section>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>导入线路与订阅</h2>
      <p class="muted">导入线路 JSON 或订阅源。</p>
    </div>
    <span id="settings-subscription-count" class="chip">订阅配置 0</span>
  </summary>
  <div class="collapsible-content stack">
    <p class="muted">支持直接粘贴线路 JSON，或把系统订阅源同步到当前设置，再批量抓取订阅里的线路。</p>
    <label class="field">
      <span>导入内容</span>
      <textarea id="settings-import-payload" class="code-input code-input-medium" spellcheck="false" placeholder="粘贴线路 JSON，或粘贴订阅 URL / 订阅 JSON"></textarea>
    </label>
    <div class="row gap-sm wrap">
      <button id="import-source-json" class="button" type="button">导入线路 JSON</button>
      <button id="import-subscriptions" class="button" type="button">导入订阅配置</button>
      <button id="sync-system-subscriptions" class="button primary" type="button">同步系统订阅源</button>
    </div>
    <div id="settings-subscription-list" class="saved-list empty-state">当前还没有订阅配置。</div>
  </div>
</details>

<section class="grid two-col align-start">
<details class="card collapsible-card">
  <summary class="collapsible-summary">
  <div>
      <h2>设置 JSON</h2>
      <p class="muted">高级编辑。</p>
    </div>
    <span class="chip">编辑器</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="row gap-sm wrap">
      <button id="refresh-settings" class="button" type="button">刷新</button>
      <button id="export-settings" class="button" type="button">导出</button>
      <button id="format-settings" class="button" type="button">格式化</button>
      <button id="save-settings" class="button primary" type="button">保存</button>
    </div>
    <textarea id="settings-json" class="code-input code-input-large" spellcheck="false">{}</textarea>
    <p id="settings-status" class="status muted"></p>
  </div>
</details>

  <details class="card collapsible-card">
    <summary class="collapsible-summary">
      <div>
        <h2>账户操作</h2>
        <p class="muted">密码和登录态管理。</p>
      </div>
      <span class="chip">按需展开</span>
    </summary>
    <div class="collapsible-content stack">
      <form id="password-form" class="stack form-grid">
        <label class="field">
          <span>当前密码</span>
          <input id="current-password" type="password" autocomplete="current-password" required />
        </label>
        <label class="field">
          <span>新密码</span>
          <input id="new-password" type="password" autocomplete="new-password" minlength="6" required />
        </label>
        <div class="row gap-sm wrap">
          <button class="button primary" type="submit">更新密码</button>
          <button id="logout-button" class="button danger" type="button">退出登录</button>
        </div>
      </form>
      <p id="password-status" class="status muted"></p>
    </div>
  </details>
</section>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>数据管理</h2>
      <p class="muted">备份、清理和恢复。</p>
    </div>
    <span class="chip">备份与清理</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="row gap-sm wrap">
      <button id="clear-synced-data" class="button" type="button">清空已同步数据</button>
      <button id="reset-all-data" class="button danger" type="button">清除所有数据</button>
    </div>
    <p class="muted">清理当前账号下已经同步到服务端的历史、收藏、搜索缓存与 Premium 缓存，不影响登录态和密码。</p>
    <div class="grid two-col">
      <label class="checkbox-row">
        <input id="backup-include-search-history" type="checkbox" checked />
        <span>备份搜索历史</span>
      </label>
      <label class="checkbox-row">
        <input id="backup-include-watch-history" type="checkbox" checked />
        <span>备份观看历史与收藏</span>
      </label>
      <label class="checkbox-row">
        <input id="backup-include-premium-data" type="checkbox" checked />
        <span>备份 Premium 搜索、历史、收藏与标签</span>
      </label>
    </div>
    <div class="row gap-sm wrap">
      <button id="export-backup" class="button" type="button">导出完整备份</button>
      <button id="import-backup-file-trigger" class="button" type="button">导入备份文件</button>
      <input id="import-backup-file" type="file" accept="application/json,.json" class="hidden" />
    </div>
    <p class="muted tiny">完整备份会始终包含当前 settings 与线路配置，并可选带上搜索、历史、收藏和 Premium 同步数据。</p>
  </div>
</details>

<script id="initial-settings" type="application/json">{}</script>
<script>{}</script>
"#,
        premium_settings_link,
        admin_link,
        escape_html(&auth_user.username),
        if auth_user.is_admin {
            "管理员"
        } else {
            "普通用户"
        },
        if auth_user.disable_premium {
            "已禁用"
        } else {
            "已启用"
        },
        escape_html(subscription_sources),
        if has_env_password {
            "已启用"
        } else {
            "未启用"
        },
        if persist_password { "启用" } else { "关闭" },
        escape_html(&settings_pretty),
        escape_script_json(&json_string(settings_value)),
        SETTINGS_SCRIPT,
    )
}

pub(super) fn render_detail_body(
    auth_user: &AuthUser,
    media: &MediaRequest,
    source_config: Option<JsonValue>,
    favorites_data_key: &str,
    favorites_value: &JsonValue,
) -> String {
    let rust_player_url = build_rust_player_url(
        &media.video_id,
        &media.source,
        &media.title,
        Some(&media.grouped_sources),
        None,
        media.is_premium,
    );
    format!(
        r#"
<section class="grid two-col align-start">
  <article class="card stack hero-card">
    <h1>{}</h1>
    <p class="muted">查看详情、选集和收藏。</p>
    <div class="row wrap gap-sm">
      <a class="button" href="/">返回首页</a>
      <a class="button primary" href="{}">打开播放器</a>
    </div>
    <div class="chip-list">
      <span class="chip">来源：{}</span>
      <span class="chip">用户：{}</span>
      <span class="chip">模式：{}</span>
    </div>
  </article>
  {}
</section>

<section class="grid detail-layout align-start">
  <article class="card stack">
    <div id="detail-summary" class="empty-state">正在加载详情...</div>
    <div class="row wrap gap-sm">
      <button id="detail-favorite-toggle" class="button" type="button">收藏中...</button>
      <span id="detail-favorite-status" class="status muted">正在读取收藏状态...</span>
    </div>
  </article>
  <article class="card stack">
    <div class="row space-between wrap gap-sm">
      <div>
        <h2>选集</h2>
        <p class="muted">点击后进入播放器。</p>
      </div>
      <a class="button" href="{}">立即播放</a>
    </div>
    <div id="episode-list" class="episodes-grid empty-state">正在加载选集...</div>
  </article>
</section>

<script id="detail-state" type="application/json">{}</script>
<script>{}</script>
"#,
        escape_html(&media.title),
        escape_html(&rust_player_url),
        escape_html(&media.source),
        escape_html(&auth_user.username),
        if media.is_premium {
            "Premium"
        } else {
            "普通"
        },
        preview_card("详情", "详情、收藏和播放共用同一套数据。",),
        escape_html(&rust_player_url),
        escape_script_json(&json_string(&serde_json::json!({
            "videoId": media.video_id,
            "source": media.source,
            "title": media.title,
            "premium": media.is_premium,
            "playerUrl": rust_player_url,
            "groupedSources": media.grouped_sources,
            "sourceConfig": source_config,
            "favoritesKey": favorites_data_key,
            "favoritesData": favorites_value,
        }))),
        DETAIL_SCRIPT,
    )
}

pub(super) fn render_player_body(
    _auth_user: &AuthUser,
    media: &MediaRequest,
    source_config: Option<JsonValue>,
    settings_value: &JsonValue,
    history_data_key: &str,
    history_value: &JsonValue,
    favorites_data_key: &str,
    favorites_value: &JsonValue,
) -> String {
    let rust_detail_url = build_rust_detail_url(
        &media.video_id,
        &media.source,
        &media.title,
        Some(&media.grouped_sources),
        media.is_premium,
    );
    let display_title = media.title.trim();
    format!(
        r#"
<section class="grid player-layout align-start">
  <article class="card stack compact-card">
    <div class="video-shell">
      <span id="playback-mode-badge" class="mode-badge hidden">直连模式</span>
      <video id="player-video" class="player-video" controls playsinline webkit-playsinline="true" x-webkit-airplay="allow" preload="metadata"></video>
      <div id="skip-backward-indicator" class="skip-indicator skip-indicator-backward hidden" aria-hidden="true">-10秒</div>
      <div id="skip-forward-indicator" class="skip-indicator skip-indicator-forward hidden" aria-hidden="true">+10秒</div>
    </div>
    <div class="row space-between wrap gap-sm player-info-row">
      <div class="stack player-title-stack">
        <h1 class="player-title-heading" title="{}">{}</h1>
        <p class="muted tiny player-inline-meta">来源 {} · {}</p>
      </div>
      <div class="row wrap gap-sm player-title-actions">
        <a class="button button-small player-back-link" href="{}">详情</a>
      </div>
    </div>
    <div class="row wrap gap-sm player-control-row">
      <button id="prev-episode" class="button" type="button">上一集</button>
      <button id="next-episode" class="button" type="button">下一集</button>
      <button id="seek-backward" class="button" type="button">后退 10 秒</button>
      <button id="seek-forward" class="button" type="button">前进 10 秒</button>
      <button id="toggle-proxy" class="button" type="button">切换代理</button>
      <button id="toggle-fullscreen" class="button" type="button">进入全屏</button>
      <button id="player-favorite-toggle" class="button" type="button">收藏中...</button>
      <label class="inline-control">
        <span>倍速</span>
        <select id="playback-rate-select">
          <option value="0.5">0.5x</option>
          <option value="0.75">0.75x</option>
          <option value="1" selected>1.0x</option>
          <option value="1.25">1.25x</option>
          <option value="1.5">1.5x</option>
          <option value="2">2.0x</option>
        </select>
      </label>
      <span id="player-progress-indicator" class="chip">00:00 / 00:00</span>
    </div>
    <details class="collapsible-card inset-card player-tools-card">
      <summary class="collapsible-summary">
        <div class="stack compact-card-body">
          <h2>更多工具</h2>
          <p class="muted">恢复进度、投屏、画中画和链接操作。</p>
        </div>
        <span class="chip">按需展开</span>
      </summary>
      <div class="collapsible-content">
        <div class="row wrap gap-sm player-control-row player-secondary-controls">
          <button id="resume-playback" class="button" type="button" disabled>恢复进度</button>
          <button id="reload-player" class="button" type="button">重新加载</button>
          <button id="toggle-pip" class="button" type="button">画中画</button>
          <button id="remote-playback" class="button" type="button">投屏 / 分享</button>
          <button id="copy-stream-url" class="button" type="button">复制播放地址</button>
          <button id="toggle-player-preferences" class="button" type="button" aria-controls="player-preferences-panel" aria-expanded="false">播放偏好</button>
        </div>
      </div>
    </details>
    <p id="player-status" class="status muted">正在加载视频详情...</p>
    <p id="player-favorite-status" class="status muted">正在读取收藏状态...</p>
    <p id="player-shortcuts" class="muted tiny">空格播放/暂停，方向键快退/快进，F 全屏，P 画中画，M 静音，[ / ] 切集，C 复制播放地址；移动端双击左右半区可快退/快进。</p>
    <section id="player-preferences-panel" class="card stack hidden" aria-label="播放器偏好面板">
      <div class="row space-between wrap gap-sm">
        <div>
          <h2>播放偏好</h2>
          <p class="muted">调整当前播放器常用偏好。</p>
        </div>
        <button id="close-player-preferences" class="button" type="button">关闭</button>
      </div>
      <div class="grid two-col">
        <label class="field">
          <span>全屏方式</span>
          <select id="player-pref-fullscreen-type">
            <option value="native">原生全屏</option>
            <option value="window">网页全屏</option>
          </select>
        </label>
        <label class="field">
          <span>广告过滤模式</span>
          <select id="player-pref-ad-filter-mode">
            <option value="off">关闭</option>
            <option value="keyword">关键词</option>
            <option value="heuristic">智能</option>
            <option value="aggressive">激进</option>
          </select>
        </label>
      </div>
      <div class="grid two-col">
        <label class="checkbox-row">
          <input id="player-pref-show-mode-indicator" type="checkbox" />
          <span>显示模式角标</span>
        </label>
        <label class="checkbox-row">
          <input id="player-pref-auto-next-episode" type="checkbox" />
          <span>自动连播下一集</span>
        </label>
        <label class="checkbox-row">
          <input id="player-pref-auto-switch-source-on-failure" type="checkbox" />
          <span>失败时自动切换来源</span>
        </label>
        <label class="checkbox-row">
          <input id="player-pref-auto-skip-intro" type="checkbox" />
          <span>自动跳过片头</span>
        </label>
        <label class="field">
          <span>片头秒数</span>
          <input id="player-pref-skip-intro-seconds" type="number" min="0" step="1" value="0" />
        </label>
        <label class="checkbox-row">
          <input id="player-pref-auto-skip-outro" type="checkbox" />
          <span>自动跳过片尾</span>
        </label>
        <label class="field">
          <span>片尾秒数</span>
          <input id="player-pref-skip-outro-seconds" type="number" min="0" step="1" value="0" />
        </label>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">来源偏好</span>
        <p id="player-source-diagnostics" class="muted tiny">正在读取来源诊断...</p>
        <div class="row wrap gap-sm">
          <button id="player-pref-clear-source-preference" class="button button-small" type="button">清除当前片名来源偏好</button>
          <button id="player-pref-clear-failed-trail" class="button button-small" type="button">清除失败来源轨迹</button>
          <button id="player-pref-copy-source-diagnostics" class="button button-small" type="button">复制来源诊断</button>
        </div>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">重试与代理诊断</span>
        <p id="player-retry-diagnostics" class="muted tiny">正在读取重试状态...</p>
        <div class="row wrap gap-sm">
          <button id="player-pref-force-direct" class="button button-small" type="button">强制直连</button>
          <button id="player-pref-force-proxy" class="button button-small" type="button">强制代理</button>
          <button id="player-pref-reset-retry-state" class="button button-small" type="button">重置重试状态</button>
          <button id="player-pref-copy-retry-diagnostics" class="button button-small" type="button">复制重试诊断</button>
          <button id="player-pref-copy-playback-diagnostics-json" class="button button-small" type="button">复制播放诊断 JSON</button>
          <button id="player-pref-export-playback-diagnostics" class="button button-small" type="button">导出播放诊断</button>
        </div>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">播放书签</span>
        <p id="player-bookmarks-summary" class="muted tiny">正在读取播放书签...</p>
        <div class="row wrap gap-sm">
          <button id="player-pref-save-bookmark" class="button button-small" type="button">保存当前书签</button>
          <button id="player-pref-clear-bookmarks" class="button button-small danger" type="button">清空当前片目书签</button>
        </div>
        <div id="player-bookmarks-list" class="saved-list empty-state">当前还没有播放书签。</div>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">链接操作</span>
        <div class="row wrap gap-sm">
          <button id="player-pref-copy-page-link" class="button button-small" type="button">复制页面链接</button>
          <button id="player-pref-copy-original-link" class="button button-small" type="button">复制原始链接</button>
          <button id="player-pref-copy-proxy-link" class="button button-small" type="button">复制代理链接</button>
          <button id="player-pref-copy-active-link" class="button button-small" type="button">复制当前链接</button>
        </div>
      </div>
    </section>
  </article>

  <div class="stack player-secondary-column">
    <details class="card collapsible-card player-navigation-card">
      <summary class="collapsible-summary">
        <div class="stack compact-card-body">
          <h2>播放导航</h2>
          <p class="muted">选集和来源按需展开，不再把播放器主体压到下方。</p>
        </div>
        <div class="chip-list">
          <span id="episode-count" class="chip">选集 0</span>
          <span id="player-source-count" class="chip">来源 0</span>
        </div>
      </summary>
      <div class="collapsible-content stack">
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>选集</h2>
            <p class="muted">在当前页内切集。</p>
          </div>
        </div>
        <div id="player-episodes" class="episodes-grid empty-state">正在加载选集...</div>
        <div class="row space-between wrap gap-sm">
          <div>
            <h2>来源切换</h2>
            <p class="muted">在播放器内直接切换来源。</p>
          </div>
          <button id="refresh-sources" class="button" type="button">刷新来源</button>
        </div>
        <div id="player-sources" class="saved-list empty-state">正在准备来源列表...</div>
      </div>
    </details>

    <details class="card collapsible-card player-metadata-card">
      <summary class="collapsible-summary">
        <div class="stack compact-card-body">
          <h2>简介与资料</h2>
          <p class="muted">海报、年份和简介默认折叠，避免正文过长。</p>
        </div>
        <span class="chip">按需展开</span>
      </summary>
      <div class="collapsible-content stack">
        <div id="player-metadata" class="empty-state">正在加载简介...</div>
      </div>
    </details>
  </div>
</section>

<details class="card collapsible-card player-library-card">
  <summary class="collapsible-summary">
    <div class="stack compact-card-body">
      <h2>播放侧栏</h2>
      <p class="muted">历史和收藏快捷跳转，默认收起减少页面长度。</p>
    </div>
    <span id="player-library-count" class="chip">历史 0</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="row wrap gap-sm">
      <button id="library-tab-history" class="button active" type="button" data-library-tab="history">历史</button>
      <button id="library-tab-favorites" class="button" type="button" data-library-tab="favorites">收藏</button>
      <button id="clear-library" class="button danger" type="button">清空当前</button>
    </div>
    <div id="player-library" class="saved-list empty-state">正在读取历史和收藏...</div>
  </div>
</details>

<script id="player-state" type="application/json">{}</script>
<script src="https://cdn.jsdelivr.net/npm/hls.js@1.6.15/dist/hls.min.js"></script>
<script>{}</script>
"#,
        escape_html(if display_title.is_empty() {
            "播放器"
        } else {
            display_title
        }),
        escape_html(if display_title.is_empty() {
            "播放器"
        } else {
            display_title
        }),
        escape_html(&media.source),
        if media.is_premium {
            "Premium"
        } else {
            "普通"
        },
        escape_html(&rust_detail_url),
        escape_script_json(&json_string(&serde_json::json!({
            "videoId": media.video_id,
            "source": media.source,
            "title": media.title,
            "premium": media.is_premium,
            "episode": media.episode,
            "groupedSources": media.grouped_sources,
            "sourceConfig": source_config,
            "settings": settings_value,
            "historyKey": history_data_key,
            "historyData": history_value,
            "favoritesKey": favorites_data_key,
            "favoritesData": favorites_value,
        }))),
        PLAYER_SCRIPT,
    )
}

pub(super) fn render_premium_body(
    premium_sources: &JsonValue,
    initial_query: &str,
    realtime_latency: bool,
    search_history: bool,
    search_display_mode: &str,
    sort_by: &str,
    premium_history: &JsonValue,
    premium_favorites: &JsonValue,
) -> String {
    let enabled_source_names = premium_enabled_source_names(premium_sources);
    let enabled_source_summary = if enabled_source_names.is_empty() {
        r#"<span class="chip">暂无启用源</span>"#.to_string()
    } else {
        enabled_source_names
            .iter()
            .take(6)
            .map(|name| format!(r#"<span class="chip">{}</span>"#, escape_html(name)))
            .collect::<Vec<_>>()
            .join("")
    };
    format!(
        r#"
<div id="premium-library-overlay" class="side-drawer-overlay hidden" aria-hidden="true">
  <aside id="premium-library-drawer" class="side-drawer" role="dialog" aria-modal="true" aria-label="Premium 收藏和历史侧栏" tabindex="-1">
    <div class="row space-between wrap gap-sm">
      <div class="stack compact-card-body">
        <h2>Premium 侧栏</h2>
        <p class="muted">历史和收藏快捷访问。</p>
      </div>
      <div class="row wrap gap-sm">
        <button class="button button-small" type="button" data-premium-library-selection-toggle>批量选择</button>
        <button class="button button-small" type="button" data-premium-library-select-all>全选当前列表</button>
        <button class="button button-small danger" type="button" data-premium-library-remove-selected>移除已选</button>
        <button class="button button-small" type="button" data-premium-library-undo disabled>撤销上次移除</button>
        <button class="button button-small" type="button" data-premium-library-copy>复制当前列表</button>
        <button class="button button-small" type="button" data-premium-library-share>复制分享包</button>
        <button class="button button-small" type="button" data-premium-library-share-link>复制分享链接</button>
        <button class="button button-small" type="button" data-premium-library-share-link-merge>复制合并分享链接</button>
        <button class="button button-small" type="button" data-premium-library-share-native>系统分享当前列表</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-save>保存快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-rename>重命名快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-duplicate>克隆快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-restore>恢复快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-merge>合并快照</button>
        <button class="button button-small danger" type="button" data-premium-library-snapshot-delete>删除快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-export>导出快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-share>复制快照分享包</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-share-link>复制快照分享链接</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-share-link-merge>复制快照合并链接</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-import>导入快照</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-import-merge>合并导入快照</button>
        <button class="button button-small" type="button" data-premium-library-export>导出当前列表</button>
        <button class="button button-small" type="button" data-premium-library-import>导入当前列表</button>
        <button class="button button-small" type="button" data-premium-library-import-merge>合并导入</button>
        <button class="button button-small" type="button" data-premium-library-import-clipboard>剪贴板导入</button>
        <button class="button button-small" type="button" data-premium-library-import-clipboard-merge>剪贴板合并</button>
        <button class="button button-small" type="button" data-premium-library-dedupe>当前列表去重</button>
        <button class="button button-small" type="button" data-premium-library-clear>清空当前列表</button>
        <button id="close-premium-library-drawer" class="button" type="button">关闭</button>
      </div>
    </div>
    <div class="row space-between wrap gap-sm">
      <div id="premium-library-toggle" class="segmented-control" role="tablist" aria-label="Premium 侧栏切换">
        <button class="segment active" type="button" role="tab" aria-selected="true" aria-controls="premium-library-panel-history" id="premium-library-tab-history" data-premium-library-tab="history">历史</button>
        <button class="segment" type="button" role="tab" aria-selected="false" aria-controls="premium-library-panel-favorites" id="premium-library-tab-favorites" data-premium-library-tab="favorites">收藏</button>
      </div>
      <span id="premium-library-status" class="chip">历史面板</span>
    </div>
    <div class="row wrap gap-sm">
      <input class="input compact-input" type="search" placeholder="筛选当前列表" data-premium-library-filter />
      <button class="button button-small" type="button" data-premium-library-filter-clear>清空筛选</button>
      <label class="inline-control">
        <span>排序</span>
        <select class="compact-input" data-premium-library-sort>
          <option value="recent-desc">最近添加</option>
          <option value="recent-asc">最早添加</option>
          <option value="title-asc">标题 A-Z</option>
          <option value="title-desc">标题 Z-A</option>
          <option value="source-asc">来源 A-Z</option>
          <option value="source-desc">来源 Z-A</option>
        </select>
      </label>
    </div>
    <div id="premium-library-panel-history" data-premium-library-panel="history" role="tabpanel" aria-labelledby="premium-library-tab-history">
      {}
    </div>
    <div id="premium-library-panel-favorites" data-premium-library-panel="favorites" class="hidden" role="tabpanel" aria-labelledby="premium-library-tab-favorites" hidden>
      {}
    </div>
  </aside>
</div>

<section class="card stack premium-search-shell">
  <div class="row space-between wrap gap-sm">
    <div class="results-section-heading">
      <h2>Premium 搜索</h2>
    </div>
    <div class="row wrap gap-sm">
      <span id="premium-search-progress" class="chip">等待搜索</span>
      <span id="premium-search-total" class="chip">结果 0</span>
    </div>
  </div>
  <form id="premium-search-form" class="stack compact-search-form">
    <label class="field compact-search-field">
      <span>搜索关键词</span>
      <div class="search-input-shell">
        <input id="premium-search-query" name="q" placeholder="输入影片名称，例如：庆余年" value="{}" required autocomplete="off" aria-autocomplete="list" aria-expanded="false" aria-controls="premium-search-history-dropdown" />
        <div id="premium-search-history-dropdown" class="search-history-dropdown hidden" aria-label="Premium 搜索历史下拉" role="listbox"></div>
      </div>
    </label>
    <div class="row wrap gap-sm compact-search-actions">
      <button class="button primary" type="submit">搜索 Premium</button>
      <button id="premium-search-clear" class="button" type="button">返回内容流</button>
    </div>
  </form>
  <div class="row space-between wrap gap-sm compact-inline-status premium-search-status-row">
    <div class="chip-list compact-chip-list">
      <span class="chip">启用源 {}</span>
      <span class="chip">收藏 {}</span>
      <span class="chip">历史 {}</span>
    </div>
    <div class="row wrap gap-sm compact-hero-actions">
      <p id="premium-status" class="status muted">正在加载 Premium 标签和内容...</p>
      <span id="premium-loaded-count" class="chip">列表已载入</span>
      <button id="premium-refresh" class="button button-small primary" type="button">刷新内容</button>
    </div>
  </div>
</section>

<section id="premium-search-section" class="card stack hidden">
  <div class="row space-between wrap gap-sm">
    <div>
      <h2>Premium 搜索结果</h2>
      <p class="muted">点击后进入详情或播放器。</p>
    </div>
    <div class="row wrap gap-sm align-controls">
      <div id="premium-search-display-toggle" class="segmented-control">
        <button class="segment" type="button" data-premium-display-mode="normal">普通</button>
        <button class="segment" type="button" data-premium-display-mode="grouped">分组</button>
      </div>
      <label class="inline-control">
        <span>排序</span>
        <select id="premium-search-sort-select">
          <option value="default">默认</option>
          <option value="relevance">相关性</option>
          <option value="latency-asc">延迟低到高</option>
          <option value="date-desc">年份新到旧</option>
          <option value="date-asc">年份旧到新</option>
          <option value="rating-desc">评分高到低</option>
          <option value="name-asc">名称 A-Z</option>
          <option value="name-desc">名称 Z-A</option>
        </select>
      </label>
      <span id="premium-search-summary" class="chip">结果 0</span>
    </div>
  </div>
  <div id="premium-search-filter-toolbar" class="stack hidden" aria-label="Premium 搜索结果筛选">
    <div class="row space-between wrap gap-sm">
      <strong>结果筛选</strong>
      <div class="row wrap gap-sm">
        <span id="premium-search-filter-summary" class="chip">未启用筛选</span>
        <button id="clear-premium-search-filters" class="button button-small" type="button">清空筛选</button>
      </div>
    </div>
    <div class="stack gap-sm">
      <div class="stack gap-sm">
        <span class="muted tiny">来源徽标</span>
        <div id="premium-search-source-badges" class="chip-list">
          <span class="chip">等待搜索结果</span>
        </div>
      </div>
      <div class="stack gap-sm">
        <span class="muted tiny">类型徽标</span>
        <div id="premium-search-type-badges" class="chip-list">
          <span class="chip">等待搜索结果</span>
        </div>
      </div>
    </div>
  </div>
  <div id="premium-search-results" class="results-grid empty-state">请输入关键词开始搜索。</div>
</section>

<div id="premium-discovery-sections" class="stack">
<details class="card collapsible-card premium-tags-card">
  <summary class="collapsible-summary">
    <div class="stack compact-card-body">
      <h2>分类标签</h2>
      <p class="muted">默认折叠完整标签云，首屏优先留给搜索和内容流。</p>
    </div>
    <span id="premium-tag-count" class="chip">标签 0</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="row wrap gap-sm">
      <button id="premium-tag-manage-toggle" class="button button-small" type="button">管理标签</button>
      <button id="restore-premium-tags" class="button button-small" type="button">恢复默认</button>
      <button id="export-premium-tags" class="button button-small" type="button">导出标签</button>
      <button id="import-premium-tags-trigger" class="button button-small" type="button">导入标签</button>
      <button id="import-premium-tags-merge-trigger" class="button button-small" type="button">合并导入标签</button>
      <button id="share-premium-tags" class="button button-small" type="button">复制标签分享包</button>
      <button id="share-premium-tags-link" class="button button-small" type="button">复制标签分享链接</button>
      <button id="share-premium-tags-link-merge" class="button button-small" type="button">复制标签合并链接</button>
    </div>
    <div id="premium-tags" class="tag-cloud empty-state">正在加载标签...</div>
  </div>
</details>

<section class="card stack">
  <div class="row space-between wrap gap-sm">
    <div>
      <h2>Premium 列表</h2>
      <p class="muted">点击卡片进入详情页。</p>
    </div>
    <div class="row wrap gap-sm">
      <span id="premium-page-indicator" class="chip">第 1 页</span>
      <button id="premium-load-more" class="button" type="button">加载更多</button>
    </div>
  </div>
  <div id="premium-results" class="results-grid compact-results-grid premium-results-grid empty-state">正在加载内容...</div>
</section>
</div>

<details class="card collapsible-card premium-overview-card">
  <summary class="collapsible-summary">
    <div class="stack compact-card-body">
      <h2>Premium 概览</h2>
      <p class="muted">摘要和启用源详情默认折叠，正文优先留给搜索和内容流。</p>
    </div>
    <span class="chip">源 {}</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="chip-list compact-chip-list">
      <span class="chip">启用源 {}</span>
      <span class="chip">收藏 {}</span>
      <span class="chip">历史 {}</span>
    </div>
    <details class="card collapsible-card premium-source-card">
      <summary class="collapsible-summary">
        <div class="stack compact-card-body">
          <h2>当前启用源</h2>
          <p class="muted">按需展开查看源摘要和完整 JSON。</p>
        </div>
      </summary>
      <div class="collapsible-content stack">
        <div class="chip-list">{}</div>
        <pre class="code-block">{}</pre>
      </div>
    </details>
  </div>
</details>

<details class="card collapsible-card premium-history-card">
  <summary class="collapsible-summary">
    <div class="stack compact-card-body">
      <h2>搜索历史</h2>
      <p class="muted">搜索历史默认折叠，不再挤占内容流首屏。</p>
    </div>
  </summary>
  <div class="collapsible-content stack">
    <div class="row space-between wrap gap-sm">
      <span id="premium-history-count" class="chip">历史 0</span>
      <button id="premium-history-clear" class="button" type="button">清空历史</button>
    </div>
    <div id="premium-history-list" class="saved-list empty-state">当前还没有 Premium 搜索历史。</div>
  </div>
</details>

<details class="card collapsible-card premium-library-preview-card">
  <summary class="collapsible-summary">
    <div class="stack compact-card-body">
      <h2>Premium 快速访问</h2>
      <p class="muted">正文只保留预览，完整管理继续放在抽屉里。</p>
    </div>
  </summary>
  <div class="collapsible-content stack">
    <div class="row space-between wrap gap-sm">
      <span id="premium-library-summary" class="chip">收藏 {} / 历史 {}</span>
      <div class="row wrap gap-sm">
        <button id="open-premium-library-drawer" class="button button-small" type="button" data-open-premium-library aria-controls="premium-library-drawer" aria-expanded="false">打开侧栏</button>
        <button class="button button-small" type="button" data-premium-library-share>复制分享包</button>
        <button class="button button-small" type="button" data-premium-library-snapshot-save>保存快照</button>
      </div>
    </div>
    <div class="library-preview-grid">
      <article class="stack gap-sm" data-premium-library-main-panel="history">
        <div class="row space-between wrap gap-sm">
          <div>
            <h3>最近历史</h3>
            <p class="muted">这里读取当前账号下已同步的 Premium 历史。</p>
          </div>
          <span class="chip">历史</span>
        </div>
        {}
      </article>
      <article class="stack gap-sm" data-premium-library-main-panel="favorites">
        <div class="row space-between wrap gap-sm">
          <div>
            <h3>最近收藏</h3>
            <p class="muted">这里读取当前账号下已同步的 Premium 收藏。</p>
          </div>
          <a class="button button-small" href="/player?premium=1">打开播放器</a>
        </div>
        {}
      </article>
    </div>
  </div>
</details>

<input id="premium-library-import-file" type="file" accept="application/json,.json" hidden />
<input id="premium-library-snapshot-import-file" type="file" accept="application/json,.json" hidden />
<input id="premium-tags-import-file" type="file" accept="application/json,.json" hidden />

<script id="premium-state" type="application/json">{}</script>
<script>{}</script>
"#,
        render_home_library_entries(premium_history, "当前没有可快速访问的 Premium 历史。"),
        render_home_library_entries(premium_favorites, "当前没有可快速访问的 Premium 收藏。"),
        escape_html(initial_query),
        enabled_source_names.len(),
        json_array_len(premium_favorites),
        json_array_len(premium_history),
        enabled_source_names.len(),
        enabled_source_names.len(),
        json_array_len(premium_favorites),
        json_array_len(premium_history),
        enabled_source_summary,
        escape_html(&pretty_json(premium_sources)),
        json_array_len(premium_favorites),
        json_array_len(premium_history),
        render_saved_items_list(
            premium_history,
            "当前没有可快速访问的 Premium 历史。",
            "/player?premium=1"
        ),
        render_saved_items_list(
            premium_favorites,
            "当前没有已同步的 Premium 收藏。",
            "/player?premium=1"
        ),
        escape_script_json(&json_string(&serde_json::json!({
            "premiumSources": premium_sources,
            "initialQuery": initial_query,
            "realtimeLatency": realtime_latency,
            "searchHistory": search_history,
            "searchDisplayMode": if search_display_mode == "grouped" { "grouped" } else { "normal" },
            "sortBy": sort_by,
            "history": premium_history,
            "favorites": premium_favorites,
        }))),
        PREMIUM_PAGE_SCRIPT,
    )
}

pub(super) fn render_admin_body(auth_user: &AuthUser) -> String {
    format!(
        r#"
<section class="grid two-col align-start">
  <article class="card stack hero-card">
    <h1>管理后台</h1>
    <p class="muted">用户管理与管理员改密。</p>
    <div class="chip-list">
      <span class="chip">当前管理员：{}</span>
      <span class="chip">角色：管理员</span>
      <span class="chip">权限：用户管理</span>
    </div>
  </article>
  {}
</section>

<section class="grid two-col align-start">
  <article class="card stack">
    <div class="row space-between wrap gap-sm">
      <div>
        <h2>创建用户</h2>
        <p class="muted">创建用户并设置 Premium 权限。</p>
      </div>
    </div>
    <form id="create-user-form" class="stack form-grid">
      <label class="field">
        <span>用户名</span>
        <input id="create-username" autocomplete="username" placeholder="例如：demo-user" required />
      </label>
      <label class="field">
        <span>密码</span>
        <input id="create-password" type="password" autocomplete="new-password" minlength="6" placeholder="至少 6 位" required />
      </label>
      <label class="checkbox-row">
        <input id="create-disable-premium" type="checkbox" checked />
        <span>禁用 Premium 内容</span>
      </label>
      <div class="row gap-sm wrap">
        <button class="button primary" type="submit">创建用户</button>
        <button id="refresh-users" class="button" type="button">刷新列表</button>
      </div>
    </form>
    <p id="admin-status" class="status muted"></p>
  </article>

  <article class="card stack">
    <div>
      <span class="eyebrow">账户安全</span>
      <h2>修改自己的密码</h2>
    </div>
    <form id="admin-password-form" class="stack form-grid">
      <label class="field">
        <span>当前密码</span>
        <input id="admin-current-password" type="password" autocomplete="current-password" required />
      </label>
      <label class="field">
        <span>新密码</span>
        <input id="admin-new-password" type="password" autocomplete="new-password" minlength="6" required />
      </label>
      <div class="row gap-sm wrap">
        <button class="button primary" type="submit">更新密码</button>
        <a class="button" href="/settings">前往设置页</a>
      </div>
    </form>
    <p id="admin-password-status" class="status muted"></p>
  </article>
</section>

<section class="card stack">
  <div class="row space-between wrap gap-sm">
    <div>
      <h2>用户列表</h2>
      <p class="muted">支持在线编辑用户名、重置密码、切换 Premium 权限和删除用户。</p>
    </div>
    <span id="admin-users-count" class="chip">用户 0</span>
  </div>
  <div id="admin-users" class="user-grid empty-state">正在加载用户列表...</div>
</section>

<script>{}</script>
"#,
        escape_html(&auth_user.username),
        preview_card("管理", "支持创建、编辑、删除和改密。",),
        ADMIN_SCRIPT,
    )
}

pub(super) fn render_login_body(next: &str) -> String {
    format!(
        r#"
<section class="stack card hero-card">
  <h1>登录到 RVideo</h1>
  <p class="muted">登录后继续使用同一套账户数据。</p>
  <form id="login-form" class="stack form-grid">
    <label class="field">
      <span>用户名</span>
      <input id="username" name="username" autocomplete="username" placeholder="admin" required />
    </label>
    <label class="field">
      <span>密码</span>
      <input id="password" name="password" type="password" autocomplete="current-password" placeholder="请输入密码" required />
    </label>
    <div class="row gap-sm">
      <button class="button primary" type="submit">登录</button>
    </div>
  </form>
  <p id="login-status" class="status muted"></p>
</section>
{}
<script id="next-path" type="application/json">{}</script>
<script>{}</script>
"#,
        preview_card("登录", "使用账号进入当前页面。",),
        escape_script_json(&json_string(next)),
        LOGIN_SCRIPT,
    )
}

pub(super) fn render_premium_settings_body(
    auth_user: &AuthUser,
    premium_sources_value: &JsonValue,
) -> String {
    let sync_notice = if auth_user.disable_premium {
        r#"
<article class="card compact-card premium-sync-card">
  <div class="row wrap gap-sm">
    <span class="chip">Premium 已禁用</span>
    <span class="muted tiny">当前账户无法使用高级源。</span>
  </div>
</article>
"#
        .to_string()
    } else {
        r#"
<article class="card compact-card premium-sync-card">
  <div class="row wrap gap-sm">
    <span class="chip">同步</span>
    <span class="muted tiny">保存后会立即写回当前设置。</span>
  </div>
</article>
"#
        .to_string()
    };
    format!(
        r#"
<section>
  <article class="card stack compact-hero-card settings-overview-card">
    <div class="row space-between wrap gap-sm compact-hero-bar">
      <div class="stack compact-card-body">
        <h2>Premium 设置</h2>
        <p class="muted">高级源与常用入口。</p>
      </div>
      <div class="row gap-sm wrap compact-hero-actions">
        <a class="button button-small" href="/settings">返回设置</a>
      </div>
    </div>
    <div class="chip-list compact-chip-list">
      <span class="chip">用户：{}</span>
      <span class="chip">Premium：{}</span>
    </div>
  </article>
</section>

{}

<section class="card stack">
  <div class="row space-between wrap gap-sm">
    <div>
      <h2>结构化高级源</h2>
      <p class="muted">默认先显示常用前几条。</p>
    </div>
    <span id="premium-source-count" class="chip">源 0 / 0</span>
  </div>
  <div class="row gap-sm wrap">
    <button id="premium-enable-all" class="button" type="button">全部启用</button>
    <button id="premium-disable-all" class="button" type="button">全部禁用</button>
    <button id="restore-default-premium-sources" class="button" type="button">恢复仓库默认</button>
    <button id="toggle-premium-source-limit" class="button" type="button">显示全部</button>
  </div>
  <div class="row gap-sm wrap">
    <input id="premium-source-search" class="input compact-input flex-1" placeholder="搜索高级源..." />
  </div>
  <details class="collapsible-card inset-card">
    <summary class="collapsible-summary">
      <div>
        <h3>新增或编辑高级源</h3>
        <p class="muted">新增或修改高级源。</p>
      </div>
      <span class="chip">表单</span>
    </summary>
    <div class="collapsible-content">
      <form id="premium-source-form" class="stack form-grid">
        <div class="grid two-col">
          <label class="field">
            <span>线路 ID</span>
            <input id="premium-source-id" placeholder="例如：ffm3u8" required />
          </label>
          <label class="field">
            <span>线路名称</span>
            <input id="premium-source-name" placeholder="例如：FFM3U8" required />
          </label>
          <label class="field">
            <span>Base URL</span>
            <input id="premium-source-base-url" placeholder="https://example.com" required />
          </label>
          <label class="field">
            <span>优先级</span>
            <input id="premium-source-priority" type="number" min="0" step="1" value="1" />
          </label>
          <label class="field">
            <span>搜索路径</span>
            <input id="premium-source-search-path" placeholder="/api.php/provide/vod/" required />
          </label>
          <label class="field">
            <span>详情路径</span>
            <input id="premium-source-detail-path" placeholder="/api.php/provide/vod/" required />
          </label>
        </div>
        <label class="checkbox-row">
          <input id="premium-source-enabled" type="checkbox" checked />
          <span>启用该高级源</span>
        </label>
        <div class="row gap-sm wrap">
          <button id="premium-source-submit" class="button primary" type="submit">新增高级源</button>
          <button id="premium-source-cancel" class="button" type="button" disabled>取消编辑</button>
        </div>
      </form>
    </div>
  </details>
  <div id="premium-source-list" class="saved-list empty-state">正在读取高级源配置...</div>
</section>

<details class="card collapsible-card">
  <summary class="collapsible-summary">
    <div>
      <h2>Premium Sources JSON</h2>
      <p class="muted">高级编辑。</p>
    </div>
    <span class="chip">编辑器</span>
  </summary>
  <div class="collapsible-content stack">
    <div class="row gap-sm wrap">
      <button id="download-premium" class="button" type="button">导出</button>
      <button id="format-premium" class="button" type="button">格式化</button>
      <button id="save-premium" class="button primary" type="button">保存</button>
    </div>
    <textarea id="premium-json" class="code-input code-input-large" spellcheck="false">{}</textarea>
    <p id="premium-status" class="status muted"></p>
  </div>
</details>

<script id="initial-premium-settings" type="application/json">{}</script>
<script>{}</script>
"#,
        escape_html(&auth_user.username),
        if auth_user.disable_premium {
            "已禁用"
        } else {
            "已启用"
        },
        sync_notice,
        escape_html(&pretty_json(premium_sources_value)),
        escape_script_json(&json_string(premium_sources_value)),
        PREMIUM_SETTINGS_SCRIPT,
    )
}
