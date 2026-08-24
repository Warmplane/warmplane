import { AppState } from '../state';
import { TaskItem } from '../api';

export function renderTasks(state: AppState): string {
  const tasks = state.tasks || [];
  const filterStatus = state.taskFilterStatus || 'all';

  const inputRequired = tasks.filter(t => t.status === 'input_required');
  const working = tasks.filter(t => t.status === 'working');
  const completed = tasks.filter(t => t.status === 'completed');
  const cancelled = tasks.filter(t => t.status === 'cancelled');
  const failed = tasks.filter(t => t.status === 'failed');

  const filteredTasks = filterStatus === 'all' 
    ? tasks 
    : tasks.filter(t => t.status === filterStatus);

  const requireApproval = state.config.policy?.require_approval || state.config.policy?.requireApproval || [];

  // 1. Pending / Input-Required Action Cards
  const inputRequiredHtml = inputRequired.length === 0 ? `
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  ` : inputRequired.map(task => {
    const inputReqs = task.inputRequests || {};
    const inputKeys = Object.keys(inputReqs);
    const hasInputReqs = inputKeys.length > 0;
    const now = Math.floor(Date.now() / 1000);
    const ttlLeft = task.expiresAtEpochSecs ? Math.max(0, task.expiresAtEpochSecs - now) : (task.ttlSeconds || 300);

    return `
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${escapeHtml(task.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${escapeHtml(task.capabilityId || 'Tool Execution')}
              </span>
              ${task.serverId ? `<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${escapeHtml(task.serverId)}</span></span>` : ''}
            </div>
          </div>

          <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
            ${task.createdAtEpochSecs ? `<div>Created: <span style="color: var(--text-muted);">${new Date(task.createdAtEpochSecs * 1000).toLocaleTimeString()}</span></div>` : ''}
            <div style="color: var(--amber-400); margin-top: 2px;">TTL Remaining: ${ttlLeft}s</div>
          </div>
        </div>

        <!-- Caller Context -->
        ${(task.context) ? `
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
            ${task.context.actor_id ? `<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${escapeHtml(task.context.actor_id)}</span></div>` : ''}
            ${task.context.operation_id ? `<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${escapeHtml(task.context.operation_id)}</span></div>` : ''}
            ${task.context.grant_id ? `<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${escapeHtml(task.context.grant_id)}</span></div>` : ''}
          </div>
        ` : ''}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${hasInputReqs ? 'Required Input Responses (MRTR / HITL)' : 'Input Responses Payload (JSON)'}
          </div>

          ${hasInputReqs ? `
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${inputKeys.map(k => {
                const reqMeta = inputReqs[k] || {};
                const promptText = typeof reqMeta === 'string' ? reqMeta : (reqMeta.prompt || reqMeta.description || reqMeta.title || k);
                const reqType = reqMeta.type || 'text';
                const initialVal = reqMeta.default !== undefined ? JSON.stringify(reqMeta.default) : (reqMeta.value !== undefined ? JSON.stringify(reqMeta.value) : (reqMeta.sanitized_args ? JSON.stringify(reqMeta.sanitized_args, null, 2) : ''));

                return `
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${escapeHtml(k)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${escapeHtml(reqType)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${escapeHtml(promptText)}</div>
                    ${(reqType === 'confirmation' || reqType === 'boolean') ? `
                      <select id="task-input-${escapeHtml(task.taskId)}-${escapeHtml(k)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    ` : (reqMeta.sanitized_args || reqType === 'object' || reqType === 'json') ? `
                      <textarea id="task-input-${escapeHtml(task.taskId)}-${escapeHtml(k)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${escapeHtml(initialVal)}</textarea>
                    ` : `
                      <input id="task-input-${escapeHtml(task.taskId)}-${escapeHtml(k)}" type="text" class="form-input" value="${escapeHtml(initialVal)}" placeholder="Enter ${escapeHtml(k)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `;
              }).join('')}
            </div>
          ` : `
            <textarea id="task-raw-input-${escapeHtml(task.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${escapeHtml(task.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${escapeHtml(task.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${escapeHtml(task.taskId)}')">
              ✓ Submit &amp; Resume
            </button>
          </div>
        </div>
      </div>
    `;
  }).join('');

  // 2. Policy Gating Rules Sidebar
  const rulesListHtml = requireApproval.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. Gated execution rules convert matching tool calls into tasks in real-time.
    </div>
  ` : requireApproval.map(rule => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">🛡️ ${escapeHtml(rule)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join('');

  // 3. Task History Table
  const tableRowsHtml = filteredTasks.length === 0 ? `
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${escapeHtml(filterStatus)}".
      </td>
    </tr>
  ` : filteredTasks.map(t => {
    const statusBg = 
      t.status === 'completed' ? 'background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);' :
      t.status === 'working' ? 'background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);' :
      t.status === 'input_required' ? 'background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);' :
      t.status === 'cancelled' ? 'background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);' :
      'background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);';

    const progressPercent = t.progress !== undefined ? Math.round(t.progress * 100) : (t.status === 'completed' ? 100 : t.status === 'working' ? 50 : 0);
    const createdStr = t.createdAtEpochSecs ? new Date(t.createdAtEpochSecs * 1000).toLocaleTimeString() : '—';

    return `
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${statusBg}">
            ${t.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${escapeHtml(t.capabilityId || 'Tool Execution')}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${escapeHtml(t.taskId)}
        </td>
        <td style="padding: 10px 14px; width: 140px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="flex: 1; height: 6px; background: var(--surface-card); border-radius: 3px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${progressPercent}%; background: ${t.status === 'completed' ? 'var(--green-400)' : 'var(--amber-400)'}; transition: width 0.3s;"></div>
            </div>
            <span style="font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">${progressPercent}%</span>
          </div>
        </td>
        <td style="padding: 10px 14px; color: var(--text-dim); font-size: 11px; text-align: right;">
          ${createdStr}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          ${t.status === 'input_required' || t.status === 'working' ? `
            <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${escapeHtml(t.taskId)}')">Cancel</button>
          ` : `
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.inspectTaskDetails('${escapeHtml(t.taskId)}')">Inspect</button>
          `}
        </td>
      </tr>
    `;
  }).join('');

  return `
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${inputRequired.length > 0 ? 'var(--amber-300)' : 'var(--green-400)'}; border-color: ${inputRequired.length > 0 ? 'rgba(245, 158, 11, 0.4)' : 'rgba(52, 211, 153, 0.4)'}; background: ${inputRequired.length > 0 ? 'rgba(245, 158, 11, 0.1)' : 'rgba(52, 211, 153, 0.1)'};">
          ${inputRequired.length} ACTION REQUIRED
        </span>
        <span style="font-size: 12px; color: var(--text-dim);">
          SEP-2663 Tasks Extension (<code style="color: var(--amber-300); font-family: var(--ff-mono);">io.modelcontextprotocol/tasks</code>) and Unified Human-in-the-Loop execution control.
        </span>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.refreshTasks()" style="font-size: 11.5px;">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
          Refresh Tasks
        </button>
      </div>
    </div>

    <!-- Top Bento Metrics (Full 12-column span) -->
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Input Required (HITL)</div>
        <div class="stat-value" style="color: ${inputRequired.length > 0 ? 'var(--amber-400)' : 'var(--text-main)'};">${inputRequired.length}</div>
        <div class="stat-sub">Awaiting operator decision or response</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Working / In-Flight</div>
        <div class="stat-value" style="color: var(--cyan-400);">${working.length}</div>
        <div class="stat-sub">Asynchronous active executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Completed Tasks</div>
        <div class="stat-value" style="color: var(--green-400);">${completed.length}</div>
        <div class="stat-sub">Finished successfully</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Cancelled / Failed</div>
        <div class="stat-value" style="color: ${failed.length > 0 ? 'var(--red-400)' : 'var(--text-muted)'};">${cancelled.length + failed.length}</div>
        <div class="stat-sub">Terminated or errored</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Input Required Action Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Action (${inputRequired.length})</span>
        </div>
        <div>
          ${inputRequiredHtml}
        </div>
      </div>

      <!-- Right Column: Active Governance Rules & Architecture -->
      <div class="col-4">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <span>🛡️ Gating Policy Rules</span>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.switchTab('policy')">Edit in Policy →</button>
        </div>
        <div class="bento-card" style="margin-bottom: 14px;">
          ${rulesListHtml}
        </div>

        <div class="bento-card">
          <div class="stat-label" style="margin-bottom: 8px;">SEP-2663 Protocol Standard</div>
          <div style="font-size: 11.5px; color: var(--text-dim); line-height: 1.5;">
            Warmplane exposes compliant <code>task_get</code>, <code>task_update</code>, and <code>task_cancel</code> tools directly on the MCP facade, enabling seamless agent delegation with non-blocking lifecycle management.
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom History & Task Registry Table (Full 12 columns) -->
    <div class="bento-card col-12" style="margin-top: 10px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px;">
          📜 Task Registry (${filteredTasks.length})
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span style="font-size: 11px; color: var(--text-dim);">Filter Status:</span>
          <select class="form-input" style="padding: 3px 8px; font-size: 11px; width: 140px;" onchange="window.app.filterTasksByStatus(this.value)">
            <option value="all" ${filterStatus === 'all' ? 'selected' : ''}>All Statuses</option>
            <option value="input_required" ${filterStatus === 'input_required' ? 'selected' : ''}>input_required</option>
            <option value="working" ${filterStatus === 'working' ? 'selected' : ''}>working</option>
            <option value="completed" ${filterStatus === 'completed' ? 'selected' : ''}>completed</option>
            <option value="cancelled" ${filterStatus === 'cancelled' ? 'selected' : ''}>cancelled</option>
            <option value="failed" ${filterStatus === 'failed' ? 'selected' : ''}>failed</option>
          </select>
        </div>
      </div>

      <div style="overflow-x: auto;">
        <table style="width: 100%; border-collapse: collapse; font-family: var(--ff-mono); font-size: 11.5px; text-align: left;">
          <thead>
            <tr style="border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 10.5px; text-transform: uppercase;">
              <th style="padding: 10px 14px;">Status</th>
              <th style="padding: 10px 14px;">Capability / Tool</th>
              <th style="padding: 10px 14px;">Task ID</th>
              <th style="padding: 10px 14px;">Progress</th>
              <th style="padding: 10px 14px; text-align: right;">Created</th>
              <th style="padding: 10px 14px; text-align: right;">Actions</th>
            </tr>
          </thead>
          <tbody>
            ${tableRowsHtml}
          </tbody>
        </table>
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
