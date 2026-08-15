import { store } from '../state';
import { api, AuditEventItem } from '../api';

export function renderAudit(): string {
  const state = store.getState();
  const events = state.auditEvents || [];
  const stats = state.auditStats || { total_events: 0, by_status: { success: 0, failed: 0, denied: 0, intercepted: 0 } };
  const verification = state.auditVerification;

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

  let eventRowsHtml = '';
  if (events.length === 0) {
    eventRowsHtml = `
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Audit Events Recorded</div>
        <div style="font-size: 12px;">Tool executions, HITL reviews, and policy blocks will be immutably recorded here with SHA-256 hash chains.</div>
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
      }

      const argsStr = e.sanitized_args ? JSON.stringify(e.sanitized_args) : '-';
      const actor = e.actor_id || e.operator_id || 'anonymous';
      const capOrTarget = e.capability_id || e.event_type;
      const latencyStr = e.execution_latency_us ? `${(e.execution_latency_us / 1000).toFixed(1)}ms` : '-';

      return `
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${escapeHtml(e.id)}</span>
              ${statusBadge}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${escapeHtml(capOrTarget)}</span>
            </div>
            <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${escapeHtml(timeStr)}</div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${escapeHtml(actor)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${escapeHtml(e.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${latencyStr}</span></div>
            <div><span style="color: var(--text-muted);">Type:</span> <span style="color: var(--text-main); font-weight: 500;">${escapeHtml(e.event_type)}</span></div>
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
        <a href="/v1/audit/export?format=csv" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;">📥 Export CSV</a>
        <a href="/v1/audit/export?format=jsonl" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;">📥 Export JSONL</a>
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

    <!-- Event Timeline List -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${events.length} events loaded</span>
    </div>

    <div>
      ${eventRowsHtml}
    </div>
  `;
}

function escapeHtml(str: string): string {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
