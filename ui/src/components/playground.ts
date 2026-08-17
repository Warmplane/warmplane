import { store } from '../state';
import { api } from '../api';

export function renderPlayground(): string {
  const state = store.getState();
  const caps = state.capabilities;
  const selectedId = state.selectedCapabilityId || (caps.length > 0 ? caps[0].id : null);
  const selectedCap = caps.find(c => c.id === selectedId);

  let capListHtml = '';
  if (caps.length === 0) {
    capListHtml = `
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;
  } else {
    capListHtml = caps.map(c => {
      const active = c.id === selectedId ? 'active' : '';
      return `
        <div class="cap-item ${active}" onclick="window.app.selectCapability('${escapeHtml(c.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${escapeHtml(c.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${escapeHtml(c.mode || 'read')}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${escapeHtml(c.server || 'local')}</div>
        </div>
      `;
    }).join('');
  }

  const initialArgs = selectedCap && selectedCap.input_schema ? JSON.stringify(selectedCap.input_schema.properties || {}, null, 2) : '{}';

  return `
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 120px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${caps.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${capListHtml}
        </div>
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${escapeHtml(selectedCap ? selectedCap.id : 'No Capability Selected')}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${escapeHtml(selectedCap ? (selectedCap.summary || selectedCap.description) : 'Connect servers to inspect and execute tools')}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${selectedCap ? '' : 'disabled'}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Execute Capability
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
              <label class="form-label" style="margin: 0;">Arguments JSON (Object)</label>
              <div style="display: flex; gap: 8px;">
                <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.toggleBatchPlayground()">🔄 Batch Mode</button>
              </div>
            </div>
            <textarea class="form-textarea" rows="7" id="pg-args-input">${escapeHtml(initialArgs)}</textarea>

            <div style="margin-top: 12px; padding: 10px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-size: 11px; font-weight: 700; color: var(--cyan-400); margin-bottom: 6px; display: flex; align-items: center; gap: 6px;">
                <span>⚡ Context Distillation Filters</span>
              </div>
              <div class="form-group" style="margin-bottom: 6px;">
                <label class="form-label" style="font-size: 10.5px;">JSONPath Filter (e.g. $.items[*].name)</label>
                <input type="text" class="form-input" id="pg-jsonpath-input" placeholder="$.result">
              </div>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Lines</label>
                  <input type="number" class="form-input" id="pg-limit-lines-input" placeholder="e.g. 50">
                </div>
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Bytes</label>
                  <input type="number" class="form-input" id="pg-truncate-bytes-input" placeholder="e.g. 20480">
                </div>
              </div>
            </div>

            <div class="form-group" style="margin-top: 10px;">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${selectedCap && selectedCap.input_schema ? `
              <div style="margin-top: 14px;">
                <label class="form-label">Input JSON Schema</label>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${escapeHtml(JSON.stringify(selectedCap.input_schema, null, 2))}</pre>
              </div>
            ` : ''}
          </div>

          <!-- Response Inspector -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">NORMALIZED EXECUTION ENVELOPE</span>
              <span id="pg-status-badge" style="font-size: 11px; font-weight: 600; color: ${state.executionResult ? (state.executionResult.status === 200 ? 'var(--green-400)' : 'var(--red-400)') : 'var(--text-dim)'}; font-family: var(--ff-mono);">
                ${state.executionResult ? `HTTP ${state.executionResult.status} · ${state.executionResult.durationMs.toFixed(1)}ms` : 'READY'}
              </span>
            </div>
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${state.executionResult ? escapeHtml(JSON.stringify(state.executionResult.data, null, 2)) : '// Response envelope output will be formatted here'}</pre>
          </div>
        </div>
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
