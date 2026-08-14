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
        No facade aliases configured in ${escapeHtml(state.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;
  } else {
    for (const [alias, target] of capAliases) {
      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${escapeHtml(target)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
    for (const [alias, target] of resAliases) {
      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${escapeHtml(target)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
    for (const [alias, target] of promptAliases) {
      rowsHtml += `
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(alias)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${escapeHtml(target)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${escapeHtml(alias)}')">✕</button>
          </div>
        </div>
      `;
    }
  }

  return `
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Facade &amp; Alias Studio</div>
        <div style="font-size: 11px; color: var(--text-dim);">Shorten capability IDs to prune prompt tokens and create stable public interfaces</div>
      </div>
    </div>

    <!-- Quick Add Form -->
    <div class="bento-card" style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px;">
        <span class="stat-label">Create New Alias</span>
      </div>
      <div style="display: grid; grid-template-columns: 140px 1fr 1fr 100px; gap: 10px; align-items: center; position: relative;">
        <select class="form-input" id="alias-kind">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        <input type="text" class="form-input" id="alias-name" placeholder="Public alias (e.g. db.query)" onkeydown="if(event.key==='Enter') window.app.createAlias()">
        <div style="position: relative; width: 100%;">
          <input type="text" class="form-input" id="alias-target" autocomplete="off" placeholder="Target ID (e.g. docker.list_containers)" style="width: 100%;" oninput="window.app.handleAliasTargetInput(this.value)" onkeydown="if(event.key==='Enter') window.app.createAlias()" onfocus="window.app.handleAliasTargetInput(this.value)" onblur="setTimeout(() => window.app.hideAliasDropdown(), 200)">
          <div id="alias-suggestions-dropdown" style="display: none; position: absolute; top: calc(100% + 4px); left: 0; right: 0; max-height: 240px; overflow-y: auto; background: var(--surface-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: 0 8px 24px rgba(0,0,0,0.4); z-index: 1000; font-family: var(--ff-mono); font-size: 11.5px;"></div>
        </div>
        <button class="btn btn-primary" onclick="window.app.createAlias()">+ Save</button>
      </div>
    </div>

    <!-- Aliases Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 12px;">
      <div style="display: grid; grid-template-columns: 90px 180px 1fr 80px; padding: 10px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TYPE</span>
        <span>PUBLIC ALIAS</span>
        <span>CANONICAL TARGET</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      ${rowsHtml}
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
