import { AppState } from '../state';

export function renderApprovals(state: AppState): string {
  const pending = state.approvals.filter(a => a.status === 'pending');
  const history = state.approvals.filter(a => a.status !== 'pending');
  const approvedCount = state.approvals.filter(a => a.status === 'approved').length;
  const rejectedCount = state.approvals.filter(a => a.status === 'rejected').length;
  const requireApproval = state.config.policy?.require_approval || state.config.policy?.requireApproval || [];

  const pendingHtml = pending.length === 0 ? `
    <div style="padding: 48px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 48px; height: 48px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--green-400); font-size: 20px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 15px; font-weight: 600; color: var(--text-main); margin-bottom: 6px;">All Clear — No Pending Approvals</div>
      <div style="font-size: 12px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool invocations intercepted by <code style="color: var(--amber-300); font-family: var(--ff-mono);">require_approval</code> governance rules will suspend execution and appear here for operator inspection, argument modification, and cryptographic gating.
      </div>
    </div>
  ` : pending.map(appr => `
    <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
      <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
        <div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
              PENDING APPROVAL
            </span>
            <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${escapeHtml(appr.id)}</span>
          </div>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(appr.capability_id)}</span>
            <span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${escapeHtml(appr.server_id)}</span></span>
          </div>
        </div>

        <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
          <div>Created: <span style="color: var(--text-muted);">${new Date(appr.created_at * 1000).toLocaleTimeString()}</span></div>
          <div style="color: var(--amber-400); margin-top: 2px;">Expires: ${new Date(appr.expires_at * 1000).toLocaleTimeString()}</div>
        </div>
      </div>

      <!-- Caller Context -->
      ${(appr.context || appr.request_id) ? `
        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
          ${appr.request_id ? `<div><span style="color: var(--text-dim);">Request:</span> <span style="color: var(--text-main);">${escapeHtml(appr.request_id)}</span></div>` : ''}
          ${appr.context?.actor_id ? `<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${escapeHtml(appr.context.actor_id)}</span></div>` : ''}
          ${appr.context?.operation_id ? `<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${escapeHtml(appr.context.operation_id)}</span></div>` : ''}
          ${appr.context?.work_item_id ? `<div><span style="color: var(--text-dim);">Work Item:</span> <span style="color: var(--text-main);">${escapeHtml(appr.context.work_item_id)}</span></div>` : ''}
        </div>
      ` : ''}

      <!-- Arguments Editor -->
      <div style="margin-bottom: 14px;">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px;">
          Parameters (Editable before approval)
        </div>
        <textarea id="appr-args-${appr.id}" class="form-textarea" rows="4" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px; line-height: 1.4;">${escapeHtml(JSON.stringify(appr.sanitized_args, null, 2))}</textarea>
      </div>

      <!-- Action Footer -->
      <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <input id="appr-operator-${appr.id}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 200px; padding: 5px 10px; font-size: 11px;">
        </div>

        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-danger" onclick="window.app.promptReject('${escapeHtml(appr.id)}')">
            ✕ Reject
          </button>
          <button class="btn btn-primary" onclick="window.app.submitApproval('${escapeHtml(appr.id)}')">
            ✓ Approve &amp; Execute
          </button>
        </div>
      </div>
    </div>
  `).join('');

  const rulesListHtml = requireApproval.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. All non-denied capabilities execute immediately.
    </div>
  ` : requireApproval.map(rule => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">🛡️ ${escapeHtml(rule)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join('');

  const historyHtml = history.length === 0 ? `
    <div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
      No historical operator decisions recorded in this session.
    </div>
  ` : `
    <div style="overflow-x: auto;">
      <table style="width: 100%; border-collapse: collapse; font-family: var(--ff-mono); font-size: 11.5px; text-align: left;">
        <thead>
          <tr style="border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 10.5px; text-transform: uppercase;">
            <th style="padding: 10px 14px;">Status</th>
            <th style="padding: 10px 14px;">Capability / Tool</th>
            <th style="padding: 10px 14px;">Approval ID</th>
            <th style="padding: 10px 14px;">Operator</th>
            <th style="padding: 10px 14px;">Reason / Notes</th>
            <th style="padding: 10px 14px; text-align: right;">Timestamp</th>
          </tr>
        </thead>
        <tbody>
          ${history.map(h => `
            <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
              <td style="padding: 10px 14px;">
                <span class="brand-badge" style="${
                  h.status === 'approved'
                    ? 'background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);'
                    : h.status === 'rejected'
                    ? 'background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);'
                    : 'background: var(--surface-hover); color: var(--text-dim);'
                }">
                  ${h.status.toUpperCase()}
                </span>
              </td>
              <td style="padding: 10px 14px; font-weight: 600; color: var(--text-main);">${escapeHtml(h.capability_id)}</td>
              <td style="padding: 10px 14px; color: var(--text-dim); font-size: 10.5px;">${escapeHtml(h.id)}</td>
              <td style="padding: 10px 14px; color: var(--text-muted);">${escapeHtml(h.operator || 'system')}</td>
              <td style="padding: 10px 14px; color: ${h.reason ? 'var(--red-400)' : 'var(--text-dim)'};">${h.reason ? `"${escapeHtml(h.reason)}"` : '—'}</td>
              <td style="padding: 10px 14px; text-align: right; color: var(--text-dim);">${new Date(h.created_at * 1000).toLocaleTimeString()}</td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `;

  return `
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${pending.length > 0 ? 'var(--amber-300)' : 'var(--green-400)'}; border-color: ${pending.length > 0 ? 'rgba(245, 158, 11, 0.4)' : 'rgba(52, 211, 153, 0.4)'}; background: ${pending.length > 0 ? 'rgba(245, 158, 11, 0.1)' : 'rgba(52, 211, 153, 0.1)'};">
          ${pending.length} PENDING DECISION${pending.length === 1 ? '' : 'S'}
        </span>
        <span style="font-size: 12px; color: var(--text-dim);">
          Inspect, parameter-tweak, and approve or reject sensitive capability executions in real-time.
        </span>
      </div>
      <button class="btn btn-ghost" onclick="window.app.refreshApprovals()" style="font-size: 11.5px;">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
        Refresh Queue
      </button>
    </div>

    <!-- Top Bento Metrics (Full 12-column span) -->
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Pending Decisions</div>
        <div class="stat-value" style="color: ${pending.length > 0 ? 'var(--amber-400)' : 'var(--text-main)'};">${pending.length}</div>
        <div class="stat-sub">Suspended in-flight executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Approved Executions</div>
        <div class="stat-value" style="color: var(--green-400);">${approvedCount}</div>
        <div class="stat-sub">Operator sanctioned calls</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Rejected Requests</div>
        <div class="stat-value" style="color: var(--red-400);">${rejectedCount}</div>
        <div class="stat-sub">Blocked &amp; reported to agent</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Active Gating Rules</div>
        <div class="stat-value" style="color: var(--cyan-400);">${requireApproval.length}</div>
        <div class="stat-sub">require_approval patterns</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Pending Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Decision (${pending.length})</span>
        </div>
        <div>
          ${pendingHtml}
        </div>
      </div>

      <!-- Right Column: Active Rules & Guidelines -->
      <div class="col-4">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <span>🛡️ Gating Policy Rules</span>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.switchTab('policy')">Edit in Policy →</button>
        </div>
        <div class="bento-card" style="margin-bottom: 14px;">
          ${rulesListHtml}
        </div>

        <div class="bento-card">
          <div class="stat-label" style="margin-bottom: 8px;">Security Notice</div>
          <div style="font-size: 11.5px; color: var(--text-dim); line-height: 1.5;">
            Warmplane enforces an HMAC-SHA256 signature on external webhook approval dispatches. Intercepted payloads are securely held until sanctioned by an operator.
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom History (Full 12 columns) -->
    <div class="bento-card col-12" style="margin-top: 10px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px;">
          📜 Recent Decision History (${history.length})
        </div>
      </div>
      ${historyHtml}
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}


