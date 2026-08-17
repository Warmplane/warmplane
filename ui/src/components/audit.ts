import { store } from '../state';
import { api, AuditEventItem } from '../api';

export function renderAudit(): string {
  const state = store.getState();
  const events = state.auditEvents || [];
  const stats = state.auditStats || { total_events: 0, by_status: { success: 0, failed: 0, denied: 0, intercepted: 0 } };
  const verification = state.auditVerification;
  const filters = state.auditFilters;
  const totalMatched = state.auditTotal ?? events.length;
  const selectedEvent = state.auditSelectedEvent;

  // Derive known servers for server filter dropdown
  const knownServers = Object.keys(state.config?.mcpServers || {});

  // Pagination calculation
  const limit = filters.limit || 25;
  const offset = filters.offset || 0;
  const currentPage = Math.floor(offset / limit) + 1;
  const totalPages = Math.max(1, Math.ceil(totalMatched / limit));
  const startItem = totalMatched === 0 ? 0 : offset + 1;
  const endItem = Math.min(offset + limit, totalMatched);

  const exportCsvUrl = api.getAuditExportUrl(
    {
      actor_id: filters.search ? undefined : undefined,
      server_id: filters.serverId !== 'all' ? filters.serverId : undefined,
      event_type: filters.eventType !== 'all' ? filters.eventType : undefined,
      status: filters.status !== 'all' ? filters.status : undefined,
      search: filters.search.trim() ? filters.search.trim() : undefined,
    },
    'csv'
  );

  const exportJsonlUrl = api.getAuditExportUrl(
    {
      server_id: filters.serverId !== 'all' ? filters.serverId : undefined,
      event_type: filters.eventType !== 'all' ? filters.eventType : undefined,
      status: filters.status !== 'all' ? filters.status : undefined,
      search: filters.search.trim() ? filters.search.trim() : undefined,
    },
    'jsonl'
  );

  const verificationBadge = verification ? (
    verification.is_valid ? `
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--green-400);">
        <span>🛡️</span>
        <span style="font-weight: 600;">Chain Verified: 100% Tamper Free (${verification.total_records} events)</span>
      </div>
    ` : `
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--red-400);">
        <span>⚠️</span>
        <span style="font-weight: 600;">TAMPER DETECTED at Record #${verification.corrupted_at_index}</span>
      </div>
    `
  ) : `
    <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
      🛡️ Verify Cryptographic Hash Chain
    </button>
  `;

  // Filter toolbar HTML
  const serverOptionsHtml = knownServers
    .map(s => `<option value="${escapeHtml(s)}" ${filters.serverId === s ? 'selected' : ''}>${escapeHtml(s)}</option>`)
    .join('');

  const filterToolbarHtml = `
    <div class="bento-card" style="padding: 14px 16px; margin-bottom: 16px; background: rgba(18, 24, 38, 0.7); border: 1px solid var(--border);">
      <div style="display: grid; grid-template-columns: 2fr 1fr 1.2fr 1.2fr auto auto; gap: 10px; align-items: center;">
        <!-- Full-text search input -->
        <div style="position: relative;">
          <input 
            type="text" 
            id="audit-search-input" 
            class="input-control" 
            style="width: 100%; padding-left: 28px; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            placeholder="Search trace, actor, capability, hash, error..." 
            value="${escapeHtml(filters.search)}"
            oninput="window.app.handleAuditSearchInput(this.value)"
          />
          <span style="position: absolute; left: 8px; top: 7px; font-size: 12px; color: var(--text-dim);">🔍</span>
        </div>

        <!-- Status Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditStatusFilter(this.value)"
          >
            <option value="all" ${filters.status === 'all' ? 'selected' : ''}>All Statuses</option>
            <option value="success" ${filters.status === 'success' ? 'selected' : ''}>🟢 Success</option>
            <option value="denied" ${filters.status === 'denied' ? 'selected' : ''}>🔴 Denied</option>
            <option value="intercepted" ${filters.status === 'intercepted' ? 'selected' : ''}>🟡 HITL Intercept</option>
            <option value="failed" ${filters.status === 'failed' ? 'selected' : ''}>❌ Failed</option>
            <option value="cancelled" ${filters.status === 'cancelled' ? 'selected' : ''}>⚪ Cancelled</option>
          </select>
        </div>

        <!-- Event Type Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditEventTypeFilter(this.value)"
          >
            <option value="all" ${filters.eventType === 'all' ? 'selected' : ''}>All Event Types</option>
            <option value="tool_execution" ${filters.eventType === 'tool_execution' ? 'selected' : ''}>Tool Execution</option>
            <option value="tool_intercepted_hitl" ${filters.eventType === 'tool_intercepted_hitl' ? 'selected' : ''}>HITL Intercept</option>
            <option value="approval_granted" ${filters.eventType === 'approval_granted' ? 'selected' : ''}>Approval Granted</option>
            <option value="approval_rejected" ${filters.eventType === 'approval_rejected' ? 'selected' : ''}>Approval Rejected</option>
            <option value="approval_expired" ${filters.eventType === 'approval_expired' ? 'selected' : ''}>Approval Expired</option>
            <option value="policy_violation" ${filters.eventType === 'policy_violation' ? 'selected' : ''}>Policy Violation</option>
            <option value="config_mutation" ${filters.eventType === 'config_mutation' ? 'selected' : ''}>Config Mutation</option>
            <option value="sampling_call" ${filters.eventType === 'sampling_call' ? 'selected' : ''}>Sampling Call</option>
            <option value="resource_access" ${filters.eventType === 'resource_access' ? 'selected' : ''}>Resource Access</option>
          </select>
        </div>

        <!-- Server Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${filters.serverId === 'all' ? 'selected' : ''}>All MCP Servers</option>
            ${serverOptionsHtml}
          </select>
        </div>

        <!-- Clear Filters Button -->
        <div>
          <button 
            class="btn btn-ghost" 
            style="padding: 6px 12px; font-size: 11.5px; height: 32px;" 
            onclick="window.app.clearAuditFilters()"
            title="Reset all search queries and filters"
          >
            ✕ Reset
          </button>
        </div>
      </div>
    </div>
  `;

  // Pagination Toolbar HTML
  const paginationControlsHtml = `
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${startItem}–${endItem}</strong> of <strong style="color: var(--text-main);">${totalMatched}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="input-control" 
          style="font-size: 11.5px; padding: 2px 6px; height: 26px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
          onchange="window.app.handleAuditPageSize(this.value)"
        >
          <option value="10" ${limit === 10 ? 'selected' : ''}>10 / page</option>
          <option value="25" ${limit === 25 ? 'selected' : ''}>25 / page</option>
          <option value="50" ${limit === 50 ? 'selected' : ''}>50 / page</option>
          <option value="100" ${limit === 100 ? 'selected' : ''}>100 / page</option>
        </select>
      </div>

      <div style="display: flex; align-items: center; gap: 6px;">
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${currentPage <= 1 ? 'disabled style="opacity: 0.4; cursor: not-allowed;"' : ''}
          onclick="window.app.auditGoToPage(1)"
          title="First Page"
        >
          ⏮ First
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${currentPage <= 1 ? 'disabled style="opacity: 0.4; cursor: not-allowed;"' : ''}
          onclick="window.app.auditPrevPage()"
        >
          ◀ Prev
        </button>
        <span style="font-size: 12px; font-weight: 600; color: var(--text-main); padding: 0 8px;">
          Page ${currentPage} of ${totalPages}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${currentPage >= totalPages ? 'disabled style="opacity: 0.4; cursor: not-allowed;"' : ''}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${currentPage >= totalPages ? 'disabled style="opacity: 0.4; cursor: not-allowed;"' : ''}
          onclick="window.app.auditGoToPage(${totalPages})"
          title="Last Page"
        >
          Last ⏭
        </button>
      </div>
    </div>
  `;

  let eventRowsHtml = '';
  if (events.length === 0) {
    eventRowsHtml = `
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">🔍</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;
  } else {
    eventRowsHtml = events.map(e => {
      const timeStr = new Date(Math.floor(e.timestamp_ns / 1_000_000)).toLocaleString();
      let statusBadge = `<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>`;
      if (e.status === 'denied') {
        statusBadge = `<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>`;
      } else if (e.status === 'intercepted') {
        statusBadge = `<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>`;
      } else if (e.status === 'failed') {
        statusBadge = `<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>`;
      } else if (e.status === 'cancelled') {
        statusBadge = `<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>`;
      }

      const argsStr = e.sanitized_args ? JSON.stringify(e.sanitized_args) : '-';
      const actor = e.actor_id || e.operator_id || 'anonymous';
      const server = e.server_id || 'system';
      const capOrTarget = e.capability_id || e.event_type;
      const latencyStr = e.execution_latency_us ? `${(e.execution_latency_us / 1000).toFixed(1)}ms` : '-';

      return `
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${escapeHtml(e.id)}</span>
              ${statusBadge}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${escapeHtml(capOrTarget)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${escapeHtml(timeStr)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${escapeHtml(e.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect 🔍
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${escapeHtml(actor)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${escapeHtml(server)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${escapeHtml(e.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${latencyStr}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${escapeHtml(argsStr)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${escapeHtml(e.prev_hash.slice(0, 16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${escapeHtml(e.hash.slice(0, 16))}...</span></div>
          </div>
        </div>
      `;
    }).join('');
  }

  // Detail Modal inspection dialog HTML
  let modalHtml = '';
  if (selectedEvent) {
    const fullTimeStr = new Date(Math.floor(selectedEvent.timestamp_ns / 1_000_000)).toISOString();
    modalHtml = `
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">🔒</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${escapeHtml(selectedEvent.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${escapeHtml(fullTimeStr)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedEvent.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedEvent.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${escapeHtml(selectedEvent.server_id || 'system')}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedEvent.capability_id || '-')}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedEvent.actor_id || selectedEvent.operator_id || 'anonymous')}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${escapeHtml(selectedEvent.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${escapeHtml(selectedEvent.request_id || '-')}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${escapeHtml(selectedEvent.client_ip || '-')}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${selectedEvent.execution_latency_us ? `${(selectedEvent.execution_latency_us / 1000).toFixed(2)} ms` : '-'}</span></div>
            </div>

            ${selectedEvent.error_message ? `
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${escapeHtml(selectedEvent.error_code || 'ERROR')}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${escapeHtml(selectedEvent.error_message)}</div>
              </div>
            ` : ''}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${escapeHtml(JSON.stringify(selectedEvent.sanitized_args || {}, null, 2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${selectedEvent.sanitized_response ? `
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${escapeHtml(JSON.stringify(selectedEvent.sanitized_response, null, 2))}</pre>
              </div>
            ` : ''}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${escapeHtml(selectedEvent.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${escapeHtml(selectedEvent.hash)}</div>
              </div>
            </div>
          </div>

          <!-- Modal Footer -->
          <div style="padding: 12px 20px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end;">
            <button class="btn btn-primary" style="font-size: 12px;" onclick="window.app.selectAuditEvent(null)">Close</button>
          </div>
        </div>
      </div>
    `;
  }

  return `
    <div class="content-header" style="margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;">
      <div>
        <h1 style="font-size: 20px; font-weight: 700; margin-bottom: 4px; display: flex; align-items: center; gap: 8px;">
          <span>🔒 WORM Audit Trail & Compliance Log</span>
        </h1>
        <p style="font-size: 12.5px; color: var(--text-dim);">Cryptographically tamper-evident, append-only execution log for SOC2 & ISO 27001 compliance.</p>
      </div>
      <div style="display: flex; gap: 8px; align-items: center;">
        ${verificationBadge}
        <a href="${exportCsvUrl}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">📥 Export CSV</a>
        <a href="${exportJsonlUrl}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">📥 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">🔄 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${stats.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${stats.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${stats.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${stats.by_status.denied}</div>
      </div>
    </div>

    <!-- Search & Filter Toolbar -->
    ${filterToolbarHtml}

    <!-- Event Timeline List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${events.length} events loaded on this page</span>
    </div>

    <!-- Event Rows -->
    <div>
      ${eventRowsHtml}
    </div>

    <!-- Pagination Footer -->
    ${totalMatched > 0 ? paginationControlsHtml : ''}

    <!-- Modal Popup for Event Inspection -->
    ${modalHtml}
  `;
}

function escapeHtml(str: string): string {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

