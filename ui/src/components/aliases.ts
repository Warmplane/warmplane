import { store } from '../state';
import { api } from '../api';

export function renderAliases(): string {
  const state = store.getState();
  const cfg = state.config;
  const capAliases = Object.entries(cfg.capabilityAliases || {});
  const resAliases = Object.entries(cfg.resourceAliases || {});
  const promptAliases = Object.entries(cfg.promptAliases || {});

  let rowsHtml = '';
  if (capAliases.length === 0 && resAliases.length === 0 && promptAliases.length === 0) {
    rowsHtml = `
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${escapeHtml(state.configPath)}. Add short names or custom descriptions to prune token payload sizes.
      </div>
    `;
  } else {
    for (const [alias, targetVal] of capAliases) {
      const targetStr = typeof targetVal === 'string' ? targetVal : targetVal.target;
      const summaryStr = typeof targetVal === 'object' && targetVal.summary ? targetVal.summary : '';
      const isPassthrough = typeof targetVal === 'object' && !!targetVal.passthrough;
      const ptBadge = isPassthrough
        ? `<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1); margin-left: 6px; font-size: 10px; padding: 1px 6px;">⚡ passthrough</span>`
        : '';
      const descBadge = summaryStr ? `<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">💬 ${escapeHtml(summaryStr)}</div>` : '';

      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <div style="display: flex; align-items: center;">
            <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
            ${ptBadge}
          </div>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono);">${escapeHtml(targetStr)}</span>
            ${descBadge}
          </div>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
    for (const [alias, targetVal] of resAliases) {
      const targetStr = typeof targetVal === 'string' ? targetVal : targetVal.target;
      const summaryStr = typeof targetVal === 'object' && targetVal.summary ? targetVal.summary : '';
      const descBadge = summaryStr ? `<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">💬 ${escapeHtml(summaryStr)}</div>` : '';

      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono);">${escapeHtml(targetStr)}</span>
            ${descBadge}
          </div>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
    for (const [alias, targetVal] of promptAliases) {
      const targetStr = typeof targetVal === 'string' ? targetVal : targetVal.target;
      const summaryStr = typeof targetVal === 'object' && targetVal.summary ? targetVal.summary : '';
      const descBadge = summaryStr ? `<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">💬 ${escapeHtml(summaryStr)}</div>` : '';

      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono);">${escapeHtml(targetStr)}</span>
            ${descBadge}
          </div>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
  }

  return `
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Shorten capability IDs and supply custom descriptions to prune prompt tokens. Enable <b>Native Passthrough</b> for tools to expose them directly in <code style="color: var(--amber-300);">tools/list</code> with their native schema.
    </div>

    <!-- Quick Add Form -->
    <div class="bento-card" style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px;">
        <span class="stat-label">Create New Alias</span>
      </div>
      <div style="display: grid; grid-template-columns: 140px 1fr 1fr 100px; gap: 10px; align-items: center; position: relative; margin-bottom: 8px;">
        <select class="form-input" id="alias-kind" onchange="const cb = document.getElementById('alias-passthrough-container'); if (cb) cb.style.display = this.value === 'tool' ? 'flex' : 'none';">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        <input type="text" class="form-input" id="alias-name" placeholder="Public alias (e.g. search)" onkeydown="if(event.key==='Enter') window.app.createAlias()">
        <div style="position: relative; width: 100%;">
          <input type="text" class="form-input" id="alias-target" autocomplete="off" placeholder="Target ID (e.g. semble.search)" style="width: 100%;" oninput="window.app.handleAliasTargetInput(this.value)" onkeydown="if(event.key==='Enter') window.app.createAlias()" onfocus="window.app.handleAliasTargetInput(this.value)" onblur="setTimeout(() => window.app.hideAliasDropdown(), 200)">
          <div id="alias-suggestions-dropdown" style="display: none; position: absolute; top: calc(100% + 4px); left: 0; right: 0; max-height: 240px; overflow-y: auto; background: var(--surface-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: 0 8px 24px rgba(0,0,0,0.4); z-index: 1000; font-family: var(--ff-mono); font-size: 11.5px;"></div>
        </div>
        <button class="btn btn-primary" onclick="window.app.createAlias()">+ Save</button>
      </div>
      <div style="display: grid; grid-template-columns: 1fr auto; gap: 12px; align-items: center;">
        <input type="text" class="form-input" id="alias-summary" placeholder="Optional custom description / prompt instruction (e.g. Fast hybrid code search)" onkeydown="if(event.key==='Enter') window.app.createAlias()" style="font-size: 12px;">
        <label id="alias-passthrough-container" style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); cursor: pointer; user-select: none; white-space: nowrap;">
          <input type="checkbox" id="alias-passthrough" style="accent-color: var(--amber-400); cursor: pointer;">
          <span>⚡ Native Passthrough Tool</span>
        </label>
      </div>
    </div>

    <!-- Aliases Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 12px;">
      <div style="display: grid; grid-template-columns: 90px 180px 1fr 80px; padding: 10px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TYPE</span>
        <span>PUBLIC ALIAS</span>
        <span>CANONICAL TARGET & SUMMARY</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      ${rowsHtml}
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
