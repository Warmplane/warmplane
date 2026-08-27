import { store } from '../state';
import { api } from '../api';

export function renderServers(): string {
  const state = store.getState();
  const servers = state.config.mcpServers || {};
  const keys = Object.keys(servers);
  const activeProfName = state.activeProfile;
  const activeProfile = activeProfName ? state.config.profiles?.[activeProfName] : undefined;
  const isProfileActive = !!activeProfile;
  const includedServers = activeProfile?.servers || [];

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
      const isIncludedInProfile = !isProfileActive || includedServers.includes(k);

      const envBadges = s.env ? Object.entries(s.env).map(([k, v]) => {
        if (v.startsWith('keychain://')) return `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">🔒 ${escapeHtml(k)} (Keychain)</span>`;
        if (v.startsWith('op://')) return `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">🔒 ${escapeHtml(k)} (1Password)</span>`;
        if (v.startsWith('env://')) return `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">🔒 ${escapeHtml(k)} (Env)</span>`;
        return `<span style="color: var(--text-dim);">${escapeHtml(k)}=***</span>`;
      }).join(' ') : 'None';

      const cb = (state.circuitBreakers || []).find(c => c.server_id === k);
      let cbBadge = `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>`;
      if (cb) {
        if (cb.state === 'open') {
          cbBadge = `<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${cb.consecutive_failures} failures)</span>`;
        } else if (cb.state === 'half_open') {
          cbBadge = `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${cb.consecutive_successes} probe)</span>`;
        }
      }

      const res = s.resilience || state.config.resilience;
      const resDetails = res ? `FT: ${res.failureThreshold || 3} · Cooldown: ${(res.cooldownMs || 30000) / 1000}s · AutoRestart: ${res.autoRestart !== false ? 'ON' : 'OFF'}` : 'Default Resilience';

      const isDegraded = statusInfo.status === 'degraded';
      const isError = statusInfo.status === 'error' || statusInfo.status === 'disconnected';
      const statusColor = isDegraded ? 'var(--amber-400)' : isError ? 'var(--red-400)' : 'var(--green-400)';

      const errorBannerHtml = (isDegraded || isError) && statusInfo.error ? `
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${escapeHtml(statusInfo.error)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${escapeHtml(k)}')">Details</button>
        </div>
      ` : '';

      const profileBoundaryBadge = isProfileActive ? (
        isIncludedInProfile
          ? `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">✔ IN CONSTELLATION</span>`
          : `<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08);">🚫 EXCLUDED FROM PROFILE: ${escapeHtml(activeProfName!)}</span>`
      ) : '';

      const cardStyle = (isProfileActive && !isIncludedInProfile)
        ? `margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);`
        : `margin-bottom: 12px; border-color: ${isDegraded ? 'rgba(251, 191, 36, 0.3)' : isError ? 'rgba(248, 113, 113, 0.3)' : 'var(--border)'};`;

      return `
        <div class="bento-card" style="${cardStyle}">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
            <div style="flex: 1; min-width: 280px;">
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${statusColor}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${escapeHtml(k)}</span>
                <span class="brand-badge">${transport}</span>
                <span class="brand-badge" style="color: ${statusColor}; border-color: rgba(245, 158, 11, 0.3);">Status: ${escapeHtml(statusInfo.status)}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${statusInfo.protocol_version}</span>
                ${cbBadge}
                ${profileBoundaryBadge}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${s.command ? 'Command: ' : 'URL: '}<code>${escapeHtml(cmd || '')}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>🛡️ ${escapeHtml(resDetails)}</span>
                ${s.env && Object.keys(s.env).length > 0 ? `<span>Env: ${envBadges}</span>` : ''}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0; flex-wrap: wrap;">
              ${isProfileActive ? (
                isIncludedInProfile ? `
                  <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);" onclick="window.app.toggleServerInProfile('${escapeHtml(activeProfName!)}', '${escapeHtml(k)}', false)">
                    Exclude from Profile
                  </button>
                ` : `
                  <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.toggleServerInProfile('${escapeHtml(activeProfName!)}', '${escapeHtml(k)}', true)">
                    + Include in Profile
                  </button>
                `
              ) : ''}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${escapeHtml(k)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${escapeHtml(k)}')">🔍 Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${escapeHtml(k)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${escapeHtml(k)}')">Remove</button>
            </div>
          </div>
          ${errorBannerHtml}
        </div>
      `;
    }).join('');
  }

  const profileConstellationBanner = isProfileActive ? `
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center; gap: 14px; flex-wrap: wrap;">
      <div style="display: flex; align-items: center; gap: 10px; flex: 1; min-width: 260px;">
        <span style="font-size: 18px; flex-shrink: 0;">🌌</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400); display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <span>Active Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${escapeHtml(activeProfName!)}</code></span>
            <span class="brand-badge" style="color: var(--text-main);">${includedServers.length} of ${keys.length} servers included</span>
          </div>
          <div style="font-size: 11.5px; color: var(--text-muted); margin-top: 2px;">
            Excluded servers are unavailable to clients connected via this profile. Tools from excluded servers are automatically hidden.
          </div>
        </div>
      </div>
      <div style="display: flex; gap: 8px; flex-shrink: 0;">
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.switchTab('profiles')">Manage Profiles</button>
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.setActiveProfile(null)">View All Servers</button>
      </div>
    </div>
  ` : '';

  return `
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${escapeHtml(state.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${profileConstellationBanner}

    ${contentHtml}

    ${renderClientIntegrations()}
  `;
}

function renderClientIntegrations(): string {
  const state = store.getState();
  const clients = state.clients || [];
  const profiles = Object.keys(state.config.profiles || {});
  const isCollapsed = state.clientsCollapsed;

  if (clients.length === 0) {
    return '';
  }

  const attachedCount = clients.filter(c => c.is_attached).length;

  const clientCards = clients.map(c => {
    const isAttached = c.is_attached;
    const isDetected = c.config_exists;
    const isInstalled = c.app_installed;

    let badge = `<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>`;
    if (isAttached) {
      const profText = c.attached_profile ? ` · Profile: ${c.attached_profile}` : '';
      badge = `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected${escapeHtml(profText)}</span>`;
    } else if (isDetected) {
      badge = `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready to Connect</span>`;
    } else if (isInstalled) {
      badge = `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ App Installed</span>`;
    }

    const activeProfile = state.activeProfile;
    const profileOptions = profiles.map(p => `
      <option value="${escapeHtml(p)}" ${activeProfile === p || c.attached_profile === p ? 'selected' : ''}>Profile: ${escapeHtml(p)}</option>
    `).join('');

    const actionBtn = isAttached
      ? `<button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${escapeHtml(c.id)}')">Disconnect</button>`
      : `<button class="btn btn-primary" style="padding: 4px 10px; font-size: 11px;" onclick="window.app.attachClient('${escapeHtml(c.id)}')">⚡ Connect</button>`;

    return `
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px 14px; display: flex; flex-direction: column; justify-content: space-between; gap: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
          <div>
            <div style="font-weight: 700; font-size: 13.5px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span>${escapeHtml(c.name)}</span>
            </div>
            <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${escapeHtml(c.category)}</div>
          </div>
          ${badge}
        </div>
        
        <div style="font-family: var(--ff-mono); font-size: 10px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${escapeHtml(c.config_path)}">
          ${escapeHtml(c.config_path)}
        </div>

        <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: 4px; padding-top: 6px; border-top: 1px solid var(--border-subtle);">
          ${profiles.length > 0 && !isAttached ? `
            <select id="client-prof-${escapeHtml(c.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 130px;">
              <option value="">All Tools (Default)</option>
              ${profileOptions}
            </select>
          ` : `<div style="font-size: 10.5px; color: var(--text-dim);">${c.other_servers_count > 0 ? `${c.other_servers_count} other tools` : 'Single tool facade'}</div>`}
          ${actionBtn}
        </div>
      </div>
    `;
  }).join('');

  return `
    <div class="bento-card" style="margin-top: 28px; padding: 14px 18px; border-color: rgba(245, 158, 11, 0.2); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div>
          <div style="font-size: 14px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
            <span>⚡ 1-Click AI Client Integrations</span>
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${attachedCount > 0 ? `${attachedCount} Connected` : 'Auto-Sync'}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">
            Attach Warmplane's unified facade to desktop IDEs and agents without editing JSON files.
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 3px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan IDEs</button>
          <span style="font-size: 12px; color: var(--text-dim);">${isCollapsed ? '▼ Show' : '▲ Hide'}</span>
        </div>
      </div>

      ${!isCollapsed ? `
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          ${clientCards}
        </div>
      ` : ''}
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
