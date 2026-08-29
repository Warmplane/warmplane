import { store } from '../state';
import { api } from '../api';
import { findTemplateForServer } from '../templates';

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

      const template = findTemplateForServer(k, s.command, s.args);
      const configuredEnvKeys = Object.keys(s.env || {});
      const missingRequiredEnv = (template?.envFields || []).filter(
        f => f.required && !configuredEnvKeys.includes(f.key)
      );

      const envBadgesList = s.env ? Object.entries(s.env).map(([envK, v]) => {
        if (v.startsWith('keychain://')) return `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">🔒 ${escapeHtml(envK)} (Keychain)</span>`;
        if (v.startsWith('op://')) return `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">🔒 ${escapeHtml(envK)} (1Password)</span>`;
        if (v.startsWith('env://')) return `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">🔒 ${escapeHtml(envK)} (Env)</span>`;
        return `<span style="color: var(--text-dim);">${escapeHtml(envK)}=***</span>`;
      }) : [];

      for (const m of missingRequiredEnv) {
        envBadgesList.push(`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);" title="Required environment variable '${escapeHtml(m.key)}' is missing">⚠️ Missing ${escapeHtml(m.key)}</span>`);
      }

      const envBadges = envBadgesList.length > 0 ? envBadgesList.join(' ') : 'None';

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
          ? `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              ✔ IN CONSTELLATION
              <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline;" onclick="window.app.toggleServerInProfile('${escapeHtml(activeProfName!)}', '${escapeHtml(k)}', false)">Exclude</button>
            </span>`
          : `<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              🚫 EXCLUDED FROM PROFILE: ${escapeHtml(activeProfName!)}
              <button style="background: none; border: none; color: var(--green-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline; font-weight: 700;" onclick="window.app.toggleServerInProfile('${escapeHtml(activeProfName!)}', '${escapeHtml(k)}', true)">+ Include</button>
            </span>`
      ) : '';

      const cardStyle = (isProfileActive && !isIncludedInProfile)
        ? `margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);`
        : `margin-bottom: 12px; border-color: ${isDegraded ? 'rgba(251, 191, 36, 0.3)' : isError ? 'rgba(248, 113, 113, 0.3)' : 'var(--border)'};`;

      return `
        <div class="bento-card" style="${cardStyle}">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
            <div style="flex: 1; min-width: 260px;">
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
            <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${escapeHtml(k)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${escapeHtml(k)}')">🔍 Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${escapeHtml(k)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${escapeHtml(k)}')">${isProfileActive ? 'Delete Globally' : 'Remove'}</button>
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
  const currentCategory = state.clientFilterCategory || 'all';
  const searchQuery = (state.clientSearchQuery || '').toLowerCase().trim();

  if (clients.length === 0) {
    return '';
  }

  const attachedCount = clients.filter(c => c.is_attached).length;
  const detectedCount = clients.filter(c => c.config_exists && !c.is_attached).length;
  const idesCount = clients.filter(c => c.category.toLowerCase().includes('ide') || c.category.toLowerCase().includes('extension')).length;
  const agentsCount = clients.filter(c => c.category.toLowerCase().includes('agent') || c.category.toLowerCase().includes('cli') || c.category.toLowerCase().includes('platform')).length;

  // Filter clients
  const filteredClients = clients.filter(c => {
    // Search query filter
    if (searchQuery) {
      const matchName = c.name.toLowerCase().includes(searchQuery);
      const matchId = c.id.toLowerCase().includes(searchQuery);
      const matchCat = c.category.toLowerCase().includes(searchQuery);
      const matchPath = c.config_path.toLowerCase().includes(searchQuery);
      if (!matchName && !matchId && !matchCat && !matchPath) return false;
    }

    // Category tab filter
    if (currentCategory === 'connected') return c.is_attached;
    if (currentCategory === 'ready') return c.config_exists || c.app_installed;
    if (currentCategory === 'ides') return c.category.toLowerCase().includes('ide') || c.category.toLowerCase().includes('extension');
    if (currentCategory === 'agents') return c.category.toLowerCase().includes('agent') || c.category.toLowerCase().includes('cli') || c.category.toLowerCase().includes('platform');
    return true;
  });

  const categoryPills = [
    { id: 'all', label: `All Ecosystems (${clients.length})` },
    { id: 'ready', label: `Ready / Installed (${detectedCount + attachedCount})` },
    { id: 'connected', label: `⚡ Connected (${attachedCount})` },
    { id: 'ides', label: `IDEs & Editors (${idesCount})` },
    { id: 'agents', label: `Agents & CLIs (${agentsCount})` },
  ].map(tab => {
    const isActive = currentCategory === tab.id;
    const activeStyle = isActive
      ? 'background: var(--amber-400); color: #000; font-weight: 700; border-color: var(--amber-400);'
      : 'background: var(--surface); color: var(--text-muted); border-color: var(--border);';
    return `
      <button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; border-radius: 100px; ${activeStyle}" onclick="window.app.setClientCategoryFilter('${escapeHtml(tab.id)}')">
        ${escapeHtml(tab.label)}
      </button>
    `;
  }).join('');

  const clientRows = filteredClients.length === 0
    ? `<div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">No AI clients match the filter "${escapeHtml(searchQuery || currentCategory)}".</div>`
    : filteredClients.map(c => {
        const isAttached = c.is_attached;
        const isDetected = c.config_exists;
        const isInstalled = c.app_installed;

        let statusBadge = `<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>`;
        if (isAttached) {
          const profText = c.attached_profile ? ` · ${c.attached_profile}` : '';
          statusBadge = `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected${escapeHtml(profText)}</span>`;
        } else if (isDetected) {
          statusBadge = `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready</span>`;
        } else if (isInstalled) {
          statusBadge = `<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ Installed</span>`;
        }

        const activeProfile = state.activeProfile;
        const profileOptions = profiles.map(p => `
          <option value="${escapeHtml(p)}" ${activeProfile === p || c.attached_profile === p ? 'selected' : ''}>Profile: ${escapeHtml(p)}</option>
        `).join('');

        const actionBtn = isAttached
          ? `<button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${escapeHtml(c.id)}')">Disconnect</button>`
          : `<button class="btn btn-primary" style="padding: 3px 10px; font-size: 11px;" onclick="window.app.attachClient('${escapeHtml(c.id)}')">⚡ Connect</button>`;

        return `
          <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; align-items: center; gap: 12px; padding: 8px 12px; background: var(--surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='var(--surface)'">
            <div>
              <div style="font-weight: 700; font-size: 13px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
                <span>${escapeHtml(c.name)}</span>
              </div>
              <div style="font-size: 10.5px; color: var(--text-dim);">${escapeHtml(c.category)}</div>
            </div>

            <div>
              ${statusBadge}
            </div>

            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${escapeHtml(c.config_path)}">
              ${escapeHtml(c.config_path)}
            </div>

            <div>
              ${profiles.length > 0 && !isAttached ? `
                <select id="client-prof-${escapeHtml(c.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 100%;">
                  <option value="">All Tools (Default)</option>
                  ${profileOptions}
                </select>
              ` : `<span style="font-size: 11px; color: var(--text-dim);">${c.other_servers_count > 0 ? `${c.other_servers_count} other tools` : 'Single facade'}</span>`}
            </div>

            <div style="text-align: right;">
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
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${attachedCount > 0 ? `${attachedCount} Connected` : `${clients.length} Ecosystems Supported`}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">
            Attach Warmplane's unified facade to desktop IDEs, CLI assistants, and autonomous agent platforms without editing JSON/TOML files.
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 3px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan Ecosystems</button>
          <span style="font-size: 12px; color: var(--text-dim);">${isCollapsed ? '▼ Show' : '▲ Hide'}</span>
        </div>
      </div>

      ${!isCollapsed ? `
        <div style="margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; margin-bottom: 12px;">
            <div style="display: flex; flex-wrap: wrap; gap: 6px;">
              ${categoryPills}
            </div>
            <div style="position: relative;">
              <input type="text" class="form-input" style="font-size: 11px; padding: 4px 10px; width: 180px; height: 26px; border-radius: 100px;" placeholder="🔍 Search clients..." value="${escapeHtml(state.clientSearchQuery || '')}" oninput="window.app.setClientSearchQuery(this.value)">
            </div>
          </div>

          <div style="display: flex; flex-direction: column; gap: 6px;">
            <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; gap: 12px; padding: 4px 12px; font-family: var(--ff-mono); font-size: 10px; color: var(--text-dim); font-weight: 700; text-transform: uppercase;">
              <span>Application</span>
              <span>Status</span>
              <span>Configuration Path</span>
              <span>Constellation Scope</span>
              <span style="text-align: right;">Action</span>
            </div>
            ${clientRows}
          </div>
        </div>
      ` : ''}
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
