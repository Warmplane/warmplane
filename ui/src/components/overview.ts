import { store } from '../state';

export function renderOverview(): string {
  const state = store.getState();
  const servers = state.config.mcpServers || {};
  const serverKeys = Object.keys(servers);
  const warmCount = serverKeys.length;

  let serverCardsHtml = '';
  if (serverKeys.length === 0) {
    serverCardsHtml = `
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;
  } else {
    serverCardsHtml = serverKeys.map(k => {
      const s = servers[k];
      const transport = s.command ? 'stdio' : 'http / sse';
      const cmd = s.command ? `${s.command} ${(s.args || []).join(' ')}` : s.url;
      const statusInfo = state.serverStatuses[k] || { status: 'connected', protocol_version: '2026-07-28' };
      const isDegraded = statusInfo.status === 'degraded';
      const isError = statusInfo.status === 'error' || statusInfo.status === 'disconnected';
      const statusColor = isDegraded ? 'var(--amber-400)' : isError ? 'var(--red-400)' : 'var(--green-400)';

      return `
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${statusColor}; display: inline-block;"></span>
              ${escapeHtml(k)}
            </span>
            <span class="brand-badge">${transport}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${escapeHtml(cmd || '')}">
            ${escapeHtml(cmd || '')}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${statusColor};">${escapeHtml(statusInfo.status)}</strong></span>
            <span>Protocol: ${statusInfo.protocol_version}</span>
          </div>
        </div>
      `;
    }).join('');
  }

  const logsHtml = state.eventLogs.length === 0 ? `
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  ` : state.eventLogs.map(l => `
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${escapeHtml(l.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${escapeHtml(l.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(l.target)}</span>
      <span style="color: var(--green-400);">${escapeHtml(l.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${escapeHtml(l.latency)}</span>
    </div>
  `).join('');

  // Live Metrics Calculations
  const metrics = state.metrics;
  const totalCatalogReqs = metrics.totalCatalogRequests;
  const etagHits = metrics.totalEtagHits;
  const etagHitRateStr = totalCatalogReqs > 0 
    ? `${((etagHits / totalCatalogReqs) * 100).toFixed(1)}%` 
    : '0.0%';
  const etagSubStr = totalCatalogReqs > 0
    ? `${etagHits} of ${totalCatalogReqs} requests served via HTTP 304`
    : 'Waiting for client requests';

  const toolCalls = metrics.totalToolCalls;
  const avgLatencyStr = toolCalls > 0
    ? `${((metrics.totalToolDurationUs / toolCalls) / 1000).toFixed(1)}ms`
    : '0.0ms';
  const latencySubStr = toolCalls > 0
    ? `${toolCalls} tool executions processed`
    : 'Local worker task queues warm';

  const totalAliases = Object.keys(state.config.capabilityAliases || {}).length +
    Object.keys(state.config.resourceAliases || {}).length +
    Object.keys(state.config.promptAliases || {}).length;

  const savingsStr = totalAliases > 0 ? `${totalAliases * 18}B / call` : '0B';
  const savingsSubStr = totalAliases > 0 ? `${totalAliases} active facade aliases pruning prompt size` : 'Configure aliases in Studio to reduce prompt size';
  const tasks = state.tasks || [];
  const inputReqTasks = tasks.filter(t => t.status === 'input_required').length;
  const activeTasks = tasks.filter(t => t.status === 'working' || t.status === 'input_required').length;

  const clients = state.clients || [];
  const attachedClientsCount = clients.filter(c => c.is_attached).length;
  const detectedClientsCount = clients.filter(c => c.config_exists && !c.is_attached).length;
  const isCollapsed = state.clientsCollapsed;

  const clientsHeaderBadge = attachedClientsCount > 0
    ? `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${attachedClientsCount} Connected</span>`
    : detectedClientsCount > 0
    ? `<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${detectedClientsCount} Ready to Connect</span>`
    : `<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>`;

  const profiles = Object.keys(state.config.profiles || {});
  const activeProfile = state.activeProfile;

  const miniClientPills = clients.map(c => {
    const isAttached = c.is_attached;
    const isDetected = c.config_exists;
    const isInstalled = c.app_installed;
    
    let dotColor = 'rgba(255, 255, 255, 0.2)';
    let statusText = 'Not Found';
    if (isAttached) {
      dotColor = 'var(--green-400)';
      statusText = c.attached_profile ? `Connected (${c.attached_profile})` : 'Connected (All Tools)';
    } else if (isDetected) {
      dotColor = 'var(--amber-300)';
      statusText = 'Ready to Attach';
    } else if (isInstalled) {
      dotColor = 'var(--cyan-400)';
      statusText = 'Installed';
    }

    const profileOptions = profiles.map(p => `
      <option value="${escapeHtml(p)}" ${activeProfile === p || c.attached_profile === p ? 'selected' : ''}>${escapeHtml(p)}</option>
    `).join('');

    const actionBtn = isAttached
      ? `<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${escapeHtml(c.id)}')">Detach</button>`
      : isDetected || isInstalled
      ? `
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${profiles.length > 0 ? `
            <select id="overview-client-prof-${escapeHtml(c.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!activeProfile ? 'selected' : ''}>All Tools</option>
              ${profileOptions}
            </select>
          ` : ''}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${escapeHtml(c.id)}')">⚡ Connect</button>
        </div>
      `
      : '';

    return `
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${dotColor}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${escapeHtml(c.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${escapeHtml(statusText)}</div>
          </div>
        </div>
        ${actionBtn}
      </div>
    `;
  }).join('');

  const clientsSectionHtml = `
    <div class="bento-card" style="margin-top: 18px; padding: 12px 16px; border-color: rgba(245, 158, 11, 0.25); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div style="display: flex; align-items: center; gap: 10px;">
          <span style="font-size: 13.5px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
            <span>⚡ 1-Click AI Client Integrations</span>
          </span>
          ${clientsHeaderBadge}
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan</button>
          <span style="font-size: 12px; color: var(--text-dim);">${isCollapsed ? '▼ Show' : '▲ Hide'}</span>
        </div>
      </div>

      ${!isCollapsed ? `
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); gap: 10px; margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--border-subtle);">
          ${miniClientPills}
        </div>
      ` : ''}
    </div>
  `;

  return `
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${savingsStr}</div>
        <div class="stat-sub">${savingsSubStr}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${etagHitRateStr}</div>
        <div class="stat-sub">${etagSubStr}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Tasks &amp; HITL State</div>
        <div class="stat-value" style="color: ${inputReqTasks > 0 ? 'var(--amber-400)' : 'var(--green-400)'};">${inputReqTasks > 0 ? `${inputReqTasks} Action Req` : `${activeTasks} Active`}</div>
        <div class="stat-sub">${inputReqTasks > 0 ? 'Awaiting Human-in-the-Loop decision' : `${tasks.length} total registered tasks`}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${warmCount} Active</div>
        <div class="stat-sub">${warmCount > 0 ? 'Persistent worker task channels' : 'No active upstream servers'}</div>
      </div>
    </div>

    ${clientsSectionHtml}

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${warmCount}) →</button>
    </div>

    <div class="bento-grid" style="margin-bottom: 24px;">
      ${serverCardsHtml}
    </div>

    <div style="font-size: 15px; font-weight: 700; color: var(--text-main); margin-bottom: 12px;">
      Live Control Plane Event Stream
    </div>
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 11.5px;">
      <div style="display: grid; grid-template-columns: 80px 100px 1fr 100px 80px; padding: 8px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TIME</span>
        <span>METHOD</span>
        <span>EVENT / TARGET</span>
        <span>STATUS</span>
        <span style="text-align: right;">LATENCY</span>
      </div>
      <div id="overview-event-rows">
        ${logsHtml}
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
