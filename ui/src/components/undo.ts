import { escapeHtml } from '../escape';
import { AppState } from '../state';

export function renderUndoFeed(state: AppState): string {
  const entries = [...(state.undoEntries || [])].sort((a, b) => b.created_at_ms - a.created_at_ms);
  if (state.undoError) return `<section class="bento-card"><h2>Action feed unavailable</h2><p>${escapeHtml(state.undoError)}</p><button class="btn btn-ghost" data-undo-action="refresh" ${state.undoLoading ? 'disabled' : ''}>Retry</button></section>`;
  const heldTasks = (state.tasks || []).filter(task => task.phase === 'pending_emission');
  if (entries.length === 0 && heldTasks.length === 0 && state.undoTotal === 0) return `<section class="bento-card"><h2>Action feed</h2><p>No reversible actions or held sends are currently recorded.</p><p class="text-dim">New armed inverses appear here with actor, profile, capability, and expiry. Snapshot contents and action arguments are never shown.</p><button class="btn btn-ghost" data-undo-action="refresh" ${state.undoLoading ? 'disabled' : ''}>Refresh</button></section>`;

  const groups = new Map<string, typeof entries>();
  for (const entry of entries) {
    const key = entry.work_item_id || '';
    groups.set(key, [...(groups.get(key) || []), entry]);
  }
  return `<section class="bento-card"><header style="display:flex;justify-content:space-between;align-items:center"><div><h2>Action feed</h2><p class="text-dim">Value-free action records. Upstream state is checked by configured contracts before an inverse runs.</p></div><button class="btn btn-ghost" data-undo-action="refresh" ${state.undoLoading ? 'disabled' : ''}>Refresh</button></header>
    ${heldTasks.map(task => {
      const remaining = Math.max(0, Math.ceil(((task.releaseAtMs || 0) - Date.now()) / 1000));
      const total = Math.max(1, Math.ceil(((task.releaseAtMs || 0) - (task.createdAtEpochSecs || 0) * 1000) / 1000));
      return `<article style="margin-top:12px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)"><div style="display:flex;justify-content:space-between;gap:12px"><div><span class="brand-badge">HELD SEND</span><strong style="margin-left:8px">${escapeHtml(task.capabilityId || 'Delayed action')}</strong><div class="text-dim" style="margin-top:6px">Actor: ${escapeHtml(task.context?.actor_id || 'Unknown')} · Task: ${escapeHtml(task.taskId)}</div></div><div class="text-dim">${remaining}s until release</div></div><progress aria-label="Time until send release" value="${Math.min(remaining, total)}" max="${total}" style="width:100%;margin-top:10px"></progress>${task.cancellable && remaining > 0 ? `<button class="btn btn-danger" style="margin-top:12px" data-undo-action="cancel-task" data-undo-target="${escapeHtml(task.taskId)}">Cancel now</button>` : ''}</article>`;
    }).join('')}
    ${Array.from(groups.entries()).map(([workItem, group]) => `<section style="margin-top:18px"><header style="display:flex;justify-content:space-between;align-items:center"><strong>${workItem ? `Work item ${escapeHtml(workItem)}` : 'Unassigned work item'}</strong>${workItem && group.some(e => e.status === 'armed' && e.expires_at_ms > Date.now()) ? `<button class="btn btn-danger" data-undo-action="rollback" data-undo-target="${escapeHtml(workItem)}">Rollback entire work item</button>` : ''}</header>
      ${group.map(entry => {
        const remaining = Math.max(0, Math.ceil((entry.expires_at_ms - Date.now()) / 1000));
        const total = Math.max(1, Math.ceil((entry.expires_at_ms - entry.created_at_ms) / 1000));
        const canUndo = entry.status === 'armed' && remaining > 0;
        return `<article style="margin-top:10px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)">
          <div style="display:flex;justify-content:space-between;gap:12px"><div><strong>${escapeHtml(entry.forward_capability)}</strong><div class="text-dim" style="margin-top:6px">Actor: ${escapeHtml(entry.actor_id || 'Unknown')} · Profile: ${escapeHtml(entry.profile_id || 'Default')}</div><div class="text-dim">Inverse: ${escapeHtml(entry.inverse_capability)}</div></div><div style="text-align:right"><span class="brand-badge">${escapeHtml(entry.status)}</span><div class="text-dim" style="margin-top:6px">${remaining > 0 ? `${remaining}s remaining` : 'Window expired'}</div></div></div>
          <div class="text-dim" style="margin-top:10px;font-size:11px">Undo ID: ${escapeHtml(entry.undo_id)} · Registered ${escapeHtml(new Date(entry.created_at_ms).toLocaleString())}</div>
          <progress aria-label="Undo window remaining" value="${Math.min(remaining, total)}" max="${total}" style="width:100%;margin-top:8px"></progress>
          <div class="text-dim" style="margin-top:6px;font-size:11px">Configured inverse fields: ${escapeHtml((entry.change_preview || []).join(', ') || 'withheld')}</div>
          ${canUndo ? `<button class="btn btn-primary" style="margin-top:12px" data-undo-action="execute" data-undo-target="${escapeHtml(entry.undo_id)}">Undo action</button>` : ''}
        </article>`;
      }).join('')}</section>`).join('')}
    <nav aria-label="Undo history pages" style="display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:16px;">
      <button class="btn btn-ghost" data-undo-action="previous" ${state.undoLoading || state.undoOffset === 0 ? 'disabled' : ''}>Previous</button>
      <span role="status">${state.undoLoading ? 'Loading…' : `${state.undoTotal ? state.undoOffset + 1 : 0}–${state.undoOffset + entries.length} of ${state.undoTotal}`}</span>
      <button class="btn btn-ghost" data-undo-action="next" ${state.undoLoading || state.undoOffset + 50 >= state.undoTotal ? 'disabled' : ''}>Next</button>
    </nav>
  </section>`;
}

