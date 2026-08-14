import { store } from '../state';
import { api } from '../api';

export function renderServers(): string {
  const state = store.getState();
  const servers = state.config.mcpServers || {};
  const keys = Object.keys(servers);

  let contentHtml = '';
  if (keys.length === 0) {
    contentHtml = `
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${escapeHtml(state.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;
  } else {
    contentHtml = keys.map(k => {
      const s = servers[k];
      const transport = s.command ? 'stdio' : 'http / sse';
      const cmd = s.command ? `${s.command} ${(s.args || []).join(' ')}` : s.url;
      const statusInfo = state.serverStatuses[k] || { status: 'connected', protocol_version: '2026-07-28' };
      const envKeys = s.env ? Object.keys(s.env).map(e => `${e}=***`).join(', ') : 'None';

      return `
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--green-400); display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${escapeHtml(k)}</span>
              <span class="brand-badge">${transport}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${statusInfo.protocol_version}</span>
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${s.command ? 'Command: ' : 'URL: '}<code>${escapeHtml(cmd || '')}</code>
            </div>
            ${s.env && Object.keys(s.env).length > 0 ? `<div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 2px;">Env: ${escapeHtml(envKeys)}</div>` : ''}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.deleteServer('${escapeHtml(k)}')">Remove</button>
          </div>
        </div>
      `;
    }).join('');
  }

  return `
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${escapeHtml(state.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload</button>
        <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Custom Server</button>
        <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
      </div>
    </div>

    ${contentHtml}
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
