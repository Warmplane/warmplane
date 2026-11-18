import { escapeHtml } from '../escape';
import { store } from '../state';
import { renderCapabilityInput, renderReversibilityEditor } from './reversibility-editor';

export type AliasKind = 'tool' | 'resource' | 'prompt';
const aliasGroups = [
  { kind: 'tool', label: 'Tool', color: 'cyan-400', key: 'capabilityAliases' },
  { kind: 'resource', label: 'Resource', color: 'green-400', key: 'resourceAliases' },
  { kind: 'prompt', label: 'Prompt', color: 'amber-300', key: 'promptAliases' },
] as const;

export function getSavedAlias(kind: AliasKind, name: string) {
  const group = aliasGroups.find(group => group.kind === kind);
  const aliases = group && store.getState().config[group.key];
  return aliases && Object.hasOwn(aliases, name) ? aliases[name] : undefined;
}

export interface AliasTestStatus {
  running: boolean;
  savedAlias: string;
  message: string;
  failed?: boolean;
}

export function renderAliasList(tests: ReadonlyMap<string, AliasTestStatus>): string {
  const { config, configPath } = store.getState();
  const rows = aliasGroups.flatMap(({ kind, label, color, key }) =>
    Object.entries(config[key] || {}).map(([name, alias]) => {
      const details = typeof alias === 'object' ? alias : undefined;
      const contract = kind === 'tool' ? details?.reversibility : undefined;
      const status = tests.get(name);
      const test = status?.running || status?.savedAlias === JSON.stringify(alias) ? status : undefined;
      const attrs = `data-alias-kind="${kind}" data-alias-name="${escapeHtml(name)}"`;
      return `
        <div class="feed-row alias-row">
          <span style="color:var(--${color});">${label}</span>
          <div class="alias-row-name">
            <strong>${escapeHtml(name)}</strong>
            ${contract ? `<span class="brand-badge">${escapeHtml(contract.class)}</span>` : ''}
            ${details?.passthrough ? '<span class="brand-badge">Passthrough</span>' : ''}
          </div>
          <div class="alias-row-target">
            <div>${escapeHtml(details?.target ?? alias)}</div>
            ${details?.summary ? `<div class="text-dim">${escapeHtml(details.summary)}</div>` : ''}
          </div>
          <div class="alias-row-actions">
            <button type="button" class="btn btn-ghost" data-alias-action="edit" ${attrs}>Edit</button>
            ${contract?.testTarget ? `<button type="button" class="btn btn-ghost" data-alias-action="test" ${attrs} aria-disabled="${!!test?.running}">${test?.running ? 'Testing…' : 'Test saved contract'}</button>` : ''}
            <button type="button" class="btn btn-ghost" data-alias-action="delete" ${attrs} aria-label="Delete ${escapeHtml(name)}">Delete</button>
          </div>
          ${contract?.testTarget ? `<div role="status" class="alias-test-status${test?.failed ? ' alias-test-failed' : ''}">${escapeHtml(test?.message)}</div>` : ''}
        </div>`;
    })
  );
  if (!rows.length) {
    return `<div class="bento-card text-dim">No aliases configured in ${escapeHtml(configPath)}. Use the form above to add one.</div>`;
  }
  return `<div class="alias-list">${rows.join('')}</div>`;
}

export function renderAliases(): string {
  return `
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Shorten capability IDs and supply custom descriptions to prune prompt tokens. Enable <b>Native Passthrough</b> for tools to expose them directly in <code style="color: var(--amber-300);">tools/list</code> with their native schema.
    </div>

    <!-- Quick Add / Edit Form -->
    <form id="alias-form" class="bento-card" novalidate style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" id="alias-form-title">Create New Alias</span>
        <button type="button" id="alias-cancel-btn" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--text-dim);" onclick="window.app.resetAliasForm()">Cancel</button>
      </div>
      <div class="alias-form-fields">
        <label for="alias-kind">Alias kind
        <select class="form-input" id="alias-kind" onchange="window.app.updateAliasKind(this.value)">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        </label>
        <label for="alias-name">Public alias
        <input type="text" class="form-input" id="alias-name" placeholder="e.g. search">
        </label>
        <div>
          <label for="alias-target">Canonical target</label>
          ${renderCapabilityInput('alias-target', 'Canonical target', 'e.g. semble.search')}
        </div>
      </div>
      <div style="display: grid; grid-template-columns: 1fr auto; gap: 12px; align-items: center;">
        <input type="text" class="form-input" id="alias-summary" aria-label="Alias summary" placeholder="Optional custom description / prompt instruction (e.g. Fast hybrid code search)" style="font-size: 12px;">
        <label id="alias-passthrough-container" style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); cursor: pointer; user-select: none; white-space: nowrap;">
          <input type="checkbox" id="alias-passthrough" style="accent-color: var(--amber-400); cursor: pointer;">
          <span>⚡ Native Passthrough Tool</span>
        </label>
      </div>
      ${renderReversibilityEditor()}
      <div id="alias-form-error" role="alert" aria-live="polite" hidden style="margin-top:12px;padding:10px 12px;border:1px solid rgba(248,113,113,.35);border-radius:var(--radius-sm);background:rgba(248,113,113,.1);color:var(--red-400);font-size:12px;"></div>
      <button type="submit" id="alias-save-btn" class="btn btn-primary" style="margin-top:12px;">Save</button>
    </form>
    <div id="alias-list"></div>
  `;
}
