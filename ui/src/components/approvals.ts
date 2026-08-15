import { AppState } from '../state';

export function renderApprovals(state: AppState): string {
  const pending = state.approvals.filter(a => a.status === 'pending');
  const history = state.approvals.filter(a => a.status !== 'pending');

  const pendingHtml = pending.length === 0 ? `
    <div style="padding: 40px 20px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">All Clear — No Pending Approvals</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 460px; margin: 0 auto;">
        Tool calls intercepted by <code style="color: var(--amber-300); font-family: var(--ff-mono);">require_approval</code> policy rules will appear here for review and execution gating.
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

  const historyHtml = history.length === 0 ? '' : `
    <div style="margin-top: 32px; border-top: 1px solid var(--border); padding-top: 18px;">
      <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px;">
        Recent History (${history.length})
      </div>
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 11.5px;">
        ${history.map(h => `
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 10px;">
              <span class="brand-badge" style="${
                h.status === 'approved'
                  ? 'background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);'
                  : h.status === 'rejected'
                  ? 'background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);'
                  : 'background: var(--surface-hover); color: var(--text-dim);'
              }">
                ${h.status.toUpperCase()}
              </span>
              <span style="font-weight: 600; color: var(--text-main);">${escapeHtml(h.capability_id)}</span>
              <span style="color: var(--text-dim); font-size: 10.5px;">${escapeHtml(h.id)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 16px; color: var(--text-dim);">
              ${h.operator ? `<span>Operator: <span style="color: var(--text-muted);">${escapeHtml(h.operator)}</span></span>` : ''}
              ${h.reason ? `<span style="color: var(--red-400); font-style: italic;">"${escapeHtml(h.reason)}"</span>` : ''}
              <span>${new Date(h.created_at * 1000).toLocaleTimeString()}</span>
            </div>
          </div>
        `).join('')}
      </div>
    </div>
  `;

  return `
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding-bottom: 14px; border-bottom: 1px solid var(--border);">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
          <span>🛡️ Human-in-the-Loop Review Queue</span>
          <span class="brand-badge" style="font-size: 10px; padding: 2px 8px;">
            ${pending.length} Pending
          </span>
        </div>
        <div style="font-size: 11.5px; color: var(--text-dim); margin-top: 3px;">
          Inspect, parameter-tweak, and approve or reject sensitive capability executions in real-time.
        </div>
      </div>
      <button class="btn btn-ghost" onclick="window.app.refreshApprovals()" style="font-size: 11.5px;">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
        Refresh Queue
      </button>
    </div>

    <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
      <span>⚡ Awaiting Operator Decision (${pending.length})</span>
    </div>

    <div>
      ${pendingHtml}
    </div>

    ${historyHtml}
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

