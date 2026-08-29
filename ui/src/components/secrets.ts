import { store } from '../state';
import { api } from '../api';
import { findTemplateForServer } from '../templates';

export function renderSecrets(): string {
  const state = store.getState();
  const configuredSecrets = state.secrets || [];
  const servers = state.config.mcpServers || {};

  // Discover any servers that are missing required keys defined in their template
  const missingSecrets: Array<{
    server: string;
    key: string;
    uri: string;
    is_vault: boolean;
    exists: boolean;
    backend: string;
    display: string;
    is_unconfigured_requirement?: boolean;
  }> = [];

  for (const [srvName, srvCfg] of Object.entries(servers)) {
    const tmpl = findTemplateForServer(srvName, srvCfg.command, srvCfg.args);
    if (tmpl) {
      const confKeys = Object.keys(srvCfg.env || {});
      for (const field of tmpl.envFields) {
        if (field.required && !confKeys.includes(field.key)) {
          missingSecrets.push({
            server: srvName,
            key: field.key,
            uri: '(Not Configured)',
            is_vault: false,
            exists: false,
            backend: 'Required Variable',
            display: `Required by ${tmpl.name} template (${field.label})`,
            is_unconfigured_requirement: true,
          });
        }
      }
    }
  }

  const allSecrets = [...configuredSecrets, ...missingSecrets];
  const total = allSecrets.length;
  const securedCount = allSecrets.filter(s => s.is_vault && s.exists !== false).length;
  const missingCount = allSecrets.filter(s => s.exists === false).length;
  const unsecuredCount = allSecrets.filter(s => !s.is_vault && s.exists !== false).length;

  const rowsHtml = allSecrets.length === 0 ? `
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  ` : allSecrets.map(s => {
    let badge = `<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>`;
    const isMissing = s.exists === false;

    if (s.is_unconfigured_requirement) {
      badge = `<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Not Configured (Required)</span>`;
    } else if (isMissing) {
      badge = `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">⚠️ Missing from ${escapeHtml(s.backend)}</span>`;
    } else if (s.is_vault) {
      badge = `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">🔒 ${escapeHtml(s.backend)}</span>`;
    }

    return `
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 10px 16px; border-bottom: 1px solid var(--border-subtle); align-items: center; font-size: 12px;">
        <span style="font-weight: 700; color: var(--text-main);">${escapeHtml(s.server)}</span>
        <span style="font-family: var(--ff-mono); color: ${isMissing ? 'var(--amber-300)' : 'var(--amber-300)'};">${escapeHtml(s.key)}</span>
        <span style="font-family: var(--ff-mono); font-size: 11px; color: ${isMissing ? 'var(--red-400)' : 'var(--text-muted)'}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${escapeHtml(s.display)}</span>
        <div>${badge}</div>
        <div style="display: flex; gap: 6px; justify-content: flex-end;">
          ${s.is_unconfigured_requirement ? `
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${escapeHtml(s.server)}', '${escapeHtml(s.key)}')">➕ Configure in Keychain</button>
          ` : isMissing ? `
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${escapeHtml(s.server)}', '${escapeHtml(s.key)}')">➕ Re-add Key</button>
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeSecretFromConfig('${escapeHtml(s.server)}', '${escapeHtml(s.key)}')">Remove from Config</button>
          ` : !s.is_vault ? `
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${escapeHtml(s.server)}', '${escapeHtml(s.key)}')">🔒 Move to Keychain</button>
          ` : `
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.deleteVaultSecret('${escapeHtml(s.key)}', '${escapeHtml(s.server)}')">Delete Key</button>
          `}
        </div>
      </div>
    `;
  }).join('');

  return `
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Manage native OS Keychain credentials (macOS Keychain, Linux Secret Service, 1Password). Secrets are injected directly in-memory at process launch and never saved to disk in plaintext.
    </div>

    <!-- Stat Header Cards -->
    <div class="bento-grid" style="margin-bottom: 20px;">
      <div class="bento-card col-4">
        <div class="stat-label">Total Required &amp; Configured</div>
        <div class="stat-value" style="color: var(--cyan-400);">${total}</div>
        <div class="stat-sub">Across all configured MCP servers</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Secured via Vault / Keychain</div>
        <div class="stat-value" style="color: var(--green-400);">${securedCount}</div>
        <div class="stat-sub">Zero-disk plaintext exposure</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Missing or Unsecured</div>
        <div class="stat-value" style="color: ${missingCount + unsecuredCount > 0 ? 'var(--red-400)' : 'var(--green-400)'};">${missingCount + unsecuredCount}</div>
        <div class="stat-sub">${missingCount > 0 ? `${missingCount} missing required key(s)` : unsecuredCount > 0 ? 'Recommend migrating to Keychain' : 'All credentials protected'}</div>
      </div>
    </div>

    <!-- Action Drawer / Store New Secret -->
    <div class="bento-card" style="margin-bottom: 20px; padding: 14px 18px; border-color: rgba(59, 130, 246, 0.3);">
      <div style="font-size: 13.5px; font-weight: 700; color: var(--text-main); margin-bottom: 10px;">
        🔑 Store New Secret in OS Keychain
      </div>
      <div style="display: grid; grid-template-columns: 200px 1fr 140px auto; gap: 10px; align-items: center;">
        <input type="text" class="form-input" id="vault-new-key" placeholder="Key identifier, e.g. github_token" style="font-size: 12px;">
        <input type="password" class="form-input" id="vault-new-val" placeholder="Secret value (will be written to OS Keychain)" style="font-size: 12px;">
        <input type="text" class="form-input" id="vault-new-service" placeholder="Service (warmplane)" value="warmplane" style="font-size: 12px;">
        <button class="btn btn-primary" onclick="window.app.saveNewVaultSecret()">Save to Keychain</button>
      </div>
    </div>

    <!-- Secrets Ledger Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden;">
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 8px 16px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 11px; font-weight: 600;">
        <span>SERVER</span>
        <span>VARIABLE KEY</span>
        <span>VALUE / URI SCHEME</span>
        <span>SECURITY STATUS</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      <div id="secrets-table-rows">
        ${rowsHtml}
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
