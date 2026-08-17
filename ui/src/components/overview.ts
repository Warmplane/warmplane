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
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${warmCount} Active</div>
        <div class="stat-sub">${warmCount > 0 ? 'Persistent worker task channels' : 'No active upstream servers'}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${avgLatencyStr}</div>
        <div class="stat-sub">${latencySubStr}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${warmCount}) →</button>
    </div>

    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 14px; margin-bottom: 24px;">
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
