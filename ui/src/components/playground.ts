import { store } from '../state';
import { api } from '../api';

export function renderPlayground(): string {
  const state = store.getState();
  const mode = state.playgroundMode || 'tools';

  const caps = state.capabilities || [];
  const resources = state.resources || [];
  const prompts = state.prompts || [];

  // Top Mode Switcher Bar
  const modeSwitcherHtml = `
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm);">
        <button 
          class="btn ${mode === 'tools' ? 'btn-primary' : 'btn-ghost'}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          🛠️ Tools (${caps.length})
        </button>
        <button 
          class="btn ${mode === 'resources' ? 'btn-primary' : 'btn-ghost'}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          📄 Resources (${resources.length})
        </button>
        <button 
          class="btn ${mode === 'prompts' ? 'btn-primary' : 'btn-ghost'}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          💬 Prompts (${prompts.length})
        </button>
      </div>

      <div style="font-size: 11.5px; color: var(--text-dim);">
        ${mode === 'tools' ? 'Interactive Tool Caller & Context Distillation' : mode === 'resources' ? 'Live MCP Resource Inspector & Reader' : 'Prompt Template Studio & Variable Binder'}
      </div>
    </div>
  `;

  if (mode === 'resources') {
    return `
      ${modeSwitcherHtml}
      ${renderResourcesPlayground(state)}
    `;
  }

  if (mode === 'prompts') {
    return `
      ${modeSwitcherHtml}
      ${renderPromptsPlayground(state)}
    `;
  }

  return `
    ${modeSwitcherHtml}
    ${renderToolsPlayground(state)}
    ${state.isBatchModalOpen ? renderBatchModal(state) : ''}
  `;
}

export function generateSampleArgsFromSchema(schema: any, onlyRequired: boolean = false): Record<string, any> {
  if (!schema || !schema.properties) return {};
  const props = schema.properties || {};
  const requiredList: string[] = Array.isArray(schema.required) ? schema.required : [];
  const result: Record<string, any> = {};

  for (const [key, propDef] of Object.entries<any>(props)) {
    const isRequired = requiredList.includes(key);
    if (onlyRequired && !isRequired) continue;

    if (propDef.default !== undefined) {
      result[key] = propDef.default;
    } else if (Array.isArray(propDef.enum) && propDef.enum.length > 0) {
      result[key] = propDef.enum[0];
    } else if (propDef.examples && Array.isArray(propDef.examples) && propDef.examples.length > 0) {
      result[key] = propDef.examples[0];
    } else if (propDef.example !== undefined) {
      result[key] = propDef.example;
    } else {
      const type = propDef.type || 'string';
      switch (type) {
        case 'string':
          result[key] = isRequired ? `sample_${key}` : '';
          break;
        case 'number':
        case 'integer':
          result[key] = 0;
          break;
        case 'boolean':
          result[key] = true;
          break;
        case 'array':
          result[key] = [];
          break;
        case 'object':
          result[key] = {};
          break;
        default:
          result[key] = `sample_${key}`;
      }
    }
  }

  return result;
}

function renderToolsPlayground(state: any): string {
  const caps = state.capabilities || [];
  const selectedId = state.selectedCapabilityId || (caps.length > 0 ? caps[0].id : null);
  const selectedCap = caps.find((c: any) => c.id === selectedId);
  const isExecuting = !!state.isExecuting;

  let capListHtml = '';
  if (caps.length === 0) {
    capListHtml = `
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;
  } else {
    capListHtml = caps.map((c: any) => {
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

  // Schema properties inspection & badges
  const schema = selectedCap?.input_schema;
  const properties = schema?.properties || {};
  const requiredKeys: string[] = Array.isArray(schema?.required) ? schema.required : [];
  const propEntries = Object.entries<any>(properties);

  let schemaPillsHtml = '';
  if (propEntries.length > 0) {
    schemaPillsHtml = `
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${propEntries.map(([key, def]) => {
          const isReq = requiredKeys.includes(key);
          const typeStr = def.type || (def.enum ? 'enum' : 'any');
          const pillColor = isReq ? 'rgba(239, 68, 68, 0.15)' : 'rgba(148, 163, 184, 0.1)';
          const textColor = isReq ? 'var(--red-400)' : 'var(--text-muted)';
          const borderColor = isReq ? 'rgba(239, 68, 68, 0.3)' : 'var(--border)';
          const desc = def.description ? ` - ${def.description}` : '';
          return `
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${pillColor}; color: ${textColor}; border: 1px solid ${borderColor}; border-radius: var(--radius-sm);" 
              title="Click to insert '${key}' (${typeStr}${desc})" 
              onclick="window.app.insertPlaygroundArgKey('${escapeHtml(key)}', '${escapeHtml(typeStr)}', ${escapeHtml(JSON.stringify(def.default ?? null))})"
            >
              + ${escapeHtml(key)} <span style="font-size: 9px; opacity: 0.7;">(${typeStr}${isReq ? ' *' : ''})</span>
            </button>
          `;
        }).join('')}
      </div>
    `;
  }

  let initialArgs = '{}';
  if (selectedId && state.playgroundArgs && state.playgroundArgs[selectedId] !== undefined) {
    initialArgs = state.playgroundArgs[selectedId];
  } else {
    const samplePayload = generateSampleArgsFromSchema(schema, false);
    initialArgs = JSON.stringify(samplePayload, null, 2);
  }

  return `
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
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
          
          <div style="display: flex; align-items: center; gap: 10px;">
            ${isExecuting ? `
              <div style="display: flex; gap: 8px; align-items: center;">
                <span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-400); font-family: var(--ff-mono); font-size: 11px; padding: 4px 8px; display: inline-flex; align-items: center; gap: 6px;">
                  <span style="width: 6px; height: 6px; border-radius: 50%; background: var(--amber-400); display: inline-block;"></span>
                  EXECUTING...
                </span>
                <button class="btn btn-danger" style="background: rgba(239, 68, 68, 0.2); color: var(--red-400); border: 1px solid rgba(239, 68, 68, 0.4); padding: 5px 12px; font-size: 11.5px;" onclick="window.app.cancelActiveOperation()">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect x="4" y="4" width="16" height="16" rx="2"></rect></svg>
                  Cancel Operation
                </button>
              </div>
            ` : `
              <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${selectedCap ? '' : 'disabled'}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                Execute Capability
              </button>
            `}
          </div>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <label class="form-label" style="margin: 0;">Arguments JSON</label>
              <div style="display: flex; gap: 6px;">
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill sample payload from schema" onclick="window.app.fillPlaygroundSampleArgs(false)">✨ Sample Template</button>
                ${requiredKeys.length > 0 ? `
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">🧹 Required Only</button>
                ` : ''}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">📋 Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${schemaPillsHtml}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${escapeHtml(initialArgs)}</textarea>

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

            <div style="margin-top: 10px; display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: rgba(0,0,0,0.25); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div>
                <div style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); display: flex; align-items: center; gap: 6px;">
                  <span>⚡ Async Task Mode (SEP-2663)</span>
                </div>
                <div style="font-size: 10.5px; color: var(--text-dim);">Execute tool asynchronously returning HTTP 202 Accepted Task</div>
              </div>
              <label style="position: relative; display: inline-block; width: 36px; height: 20px; margin: 0; cursor: pointer;">
                <input type="checkbox" id="pg-async-task-toggle" ${state.playgroundAsyncTask ? 'checked' : ''} onchange="window.app.togglePlaygroundAsyncTask(this.checked)" style="opacity: 0; width: 0; height: 0;">
                <span style="position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: ${state.playgroundAsyncTask ? 'var(--amber-400)' : 'var(--border)'}; transition: .3s; border-radius: 20px;">
                  <span style="position: absolute; content: ''; height: 14px; width: 14px; left: ${state.playgroundAsyncTask ? '19px' : '3px'}; bottom: 3px; background-color: white; transition: .3s; border-radius: 50%;"></span>
                </span>
              </label>
            </div>

            <div class="form-group" style="margin-top: 10px;">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${selectedCap && selectedCap.input_schema ? `
              <div style="margin-top: 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <label class="form-label" style="margin: 0;">Input JSON Schema</label>
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${propEntries.length} field${propEntries.length === 1 ? '' : 's'} (${requiredKeys.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${escapeHtml(JSON.stringify(selectedCap.input_schema, null, 2))}</pre>
              </div>
            ` : ''}
          </div>

          <!-- Response Inspector -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">
                ${state.executionResult && (state.executionResult.status === 202 || state.executionResult.data?.resultType === 'task') ? 'SEP-2663 TASK RESPONSE' : 'NORMALIZED EXECUTION ENVELOPE'}
              </span>
              <span id="pg-status-badge" style="font-size: 11px; font-weight: 600; color: ${state.executionResult ? (state.executionResult.status === 200 ? 'var(--green-400)' : state.executionResult.status === 202 ? 'var(--amber-300)' : 'var(--red-400)') : 'var(--text-dim)'}; font-family: var(--ff-mono);">
                ${state.executionResult ? `HTTP ${state.executionResult.status} · ${state.executionResult.durationMs.toFixed(1)}ms` : 'READY'}
              </span>
            </div>

            ${state.executionResult && (state.executionResult.status === 202 || state.executionResult.data?.resultType === 'task') ? `
              <div style="margin-bottom: 12px; padding: 12px 14px; background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.35); border-radius: var(--radius-sm); display: flex; justify-content: space-between; align-items: center;">
                <div>
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                      ${escapeHtml(state.executionResult.data?.task?.status || state.executionResult.data?.status || 'TASK_CREATED').toUpperCase()}
                    </span>
                    <span style="font-family: var(--ff-mono); font-size: 12px; font-weight: 700; color: var(--text-main);">${escapeHtml(state.executionResult.data?.task?.taskId || state.executionResult.data?.taskId || '')}</span>
                  </div>
                  <div style="font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                    Execution suspended for Human-in-the-Loop approval or async resolution.
                  </div>
                </div>
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11px;" onclick="window.app.switchTab('tasks')">
                  Go to Tasks &amp; Approvals →
                </button>
              </div>
            ` : ''}

            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${state.executionResult ? escapeHtml(JSON.stringify(state.executionResult.data, null, 2)) : '// Response envelope output will be formatted here'}</pre>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderResourcesPlayground(state: any): string {
  const resources = state.resources || [];
  const selectedId = state.selectedResourceId || (resources.length > 0 ? resources[0].id : null);
  const selectedRes = resources.find((r: any) => r.id === selectedId);
  const result = state.resourceReadResult;

  let resListHtml = '';
  if (resources.length === 0) {
    resListHtml = `
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;
  } else {
    resListHtml = resources.map((r: any) => {
      const active = r.id === selectedId ? 'active' : '';
      const scheme = r.uri ? r.uri.split(':')[0] : 'res';
      return `
        <div class="cap-item ${active}" onclick="window.app.selectResource('${escapeHtml(r.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${escapeHtml(r.name || r.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${escapeHtml(scheme)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${escapeHtml(r.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${escapeHtml(r.server || 'local')}</span>
            <span>${escapeHtml(r.mime_type || 'text/plain')}</span>
          </div>
        </div>
      `;
    }).join('');
  }

  return `
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Resources Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${resources.length} resources..." oninput="window.app.filterResources(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-res-list">
          ${resListHtml}
        </div>
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${escapeHtml(selectedRes ? (selectedRes.name || selectedRes.id) : 'No Resource Selected')}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${escapeHtml(selectedRes ? selectedRes.uri : 'Select a resource from the list to read live content')}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeReadResource()" ${selectedRes ? '' : 'disabled'}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
            Read Resource Content
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request / Distillation Parameters -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            ${selectedRes ? `
              <div style="background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 14px;">
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">Resource Metadata</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11.5px;">
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedRes.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${escapeHtml(selectedRes.mime_type || 'text/plain')}</strong></div>
                </div>
                ${selectedRes.description ? `
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${escapeHtml(selectedRes.description)}
                  </div>
                ` : ''}
              </div>
            ` : ''}

            <div style="padding: 12px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-size: 11px; font-weight: 700; color: var(--cyan-400); margin-bottom: 8px; display: flex; align-items: center; gap: 6px;">
                <span>⚡ Context Distillation Options</span>
              </div>
              <div class="form-group" style="margin-bottom: 8px;">
                <label class="form-label" style="font-size: 10.5px;">JSONPath Expression</label>
                <input type="text" class="form-input" id="pg-res-jsonpath-input" placeholder="$.items[*].data">
              </div>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Lines</label>
                  <input type="number" class="form-input" id="pg-res-lines-input" placeholder="e.g. 100">
                </div>
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Bytes</label>
                  <input type="number" class="form-input" id="pg-res-bytes-input" placeholder="e.g. 32768">
                </div>
              </div>
            </div>
          </div>

          <!-- Content Output Preview -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RESOURCE CONTENT ENVELOPE</span>
              <span style="font-size: 11px; font-weight: 600; color: ${result ? (result.status === 200 ? 'var(--green-400)' : 'var(--red-400)') : 'var(--text-dim)'}; font-family: var(--ff-mono);">
                ${result ? `HTTP ${result.status} · ${result.durationMs.toFixed(1)}ms` : 'READY'}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${result ? escapeHtml(JSON.stringify(result.data, null, 2)) : '// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderPromptsPlayground(state: any): string {
  const prompts = state.prompts || [];
  const selectedId = state.selectedPromptId || (prompts.length > 0 ? prompts[0].id : null);
  const selectedPrompt = prompts.find((p: any) => p.id === selectedId);
  const result = state.promptGetResult;

  let promptListHtml = '';
  if (prompts.length === 0) {
    promptListHtml = `
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;
  } else {
    promptListHtml = prompts.map((p: any) => {
      const active = p.id === selectedId ? 'active' : '';
      const argCount = p.arguments ? p.arguments.length : 0;
      return `
        <div class="cap-item ${active}" onclick="window.app.selectPrompt('${escapeHtml(p.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${escapeHtml(p.name || p.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${argCount} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${escapeHtml(p.description || p.title || 'Prompt template')}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${escapeHtml(p.server || 'local')}</div>
        </div>
      `;
    }).join('');
  }

  // Dynamic argument form fields
  let argumentFieldsHtml = '';
  if (selectedPrompt && selectedPrompt.arguments && selectedPrompt.arguments.length > 0) {
    argumentFieldsHtml = selectedPrompt.arguments.map((arg: any) => `
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${escapeHtml(arg.name)}</label>
          ${arg.required ? '<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>' : '<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${arg.description ? `<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${escapeHtml(arg.description)}</div>` : ''}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${escapeHtml(arg.name)}" placeholder="Enter ${escapeHtml(arg.name)}..." />
      </div>
    `).join('');
  } else if (selectedPrompt) {
    argumentFieldsHtml = `
      <div style="padding: 12px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11.5px; color: var(--text-dim);">
        This prompt template does not require any input arguments.
      </div>
    `;
  }

  return `
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Prompts Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${prompts.length} prompts..." oninput="window.app.filterPrompts(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-prompt-list">
          ${promptListHtml}
        </div>
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${escapeHtml(selectedPrompt ? (selectedPrompt.name || selectedPrompt.id) : 'No Prompt Selected')}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${escapeHtml(selectedPrompt ? (selectedPrompt.description || selectedPrompt.title || 'Bind variables and render messages') : 'Select a prompt from the list to test')}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeGetPrompt()" ${selectedPrompt ? '' : 'disabled'}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Render Prompt Messages
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Arguments Form Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="font-size: 12px; font-weight: 700; color: var(--text-main); margin-bottom: 12px; text-transform: uppercase;">
              Template Arguments
            </div>
            ${argumentFieldsHtml}
          </div>

          <!-- Rendered Messages Output -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RENDERED PROMPT MESSAGES</span>
              <span style="font-size: 11px; font-weight: 600; color: ${result ? (result.status === 200 ? 'var(--green-400)' : 'var(--red-400)') : 'var(--text-dim)'}; font-family: var(--ff-mono);">
                ${result ? `HTTP ${result.status} · ${result.durationMs.toFixed(1)}ms` : 'READY'}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${result ? escapeHtml(JSON.stringify(result.data, null, 2)) : '// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderBatchModal(state: any): string {
  const caps = state.capabilities || [];
  const steps = state.batchSteps || [];

  const stepsHtml = steps.map((step: any, idx: number) => {
    const stepCap = caps.find((c: any) => c.id === step.capability_id);
    const stepSchema = stepCap?.input_schema;
    const stepProps = stepSchema?.properties || {};
    const stepReqs: string[] = Array.isArray(stepSchema?.required) ? stepSchema.required : [];
    const stepPropEntries = Object.entries<any>(stepProps);

    const optionsHtml = caps.map((c: any) => `
      <option value="${escapeHtml(c.id)}" ${c.id === step.capability_id ? 'selected' : ''}>
        ${escapeHtml(c.id)} (${escapeHtml(c.server || 'local')})
      </option>
    `).join('');

    let stepSchemaBadgesHtml = '';
    if (stepPropEntries.length > 0) {
      stepSchemaBadgesHtml = `
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${stepPropEntries.map(([k, def]) => {
            const isReq = stepReqs.includes(k);
            const typeStr = def.type || (def.enum ? 'enum' : 'any');
            const pillColor = isReq ? 'rgba(239, 68, 68, 0.15)' : 'rgba(148, 163, 184, 0.1)';
            const textColor = isReq ? 'var(--red-400)' : 'var(--text-muted)';
            const borderColor = isReq ? 'rgba(239, 68, 68, 0.3)' : 'var(--border)';
            return `
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${pillColor}; color: ${textColor}; border: 1px solid ${borderColor}; border-radius: 3px;" title="${escapeHtml(def.description || '')}">
                ${escapeHtml(k)} (${typeStr}${isReq ? ' *' : ''})
              </span>
            `;
          }).join('')}
        </div>
      `;
    }

    return `
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${idx + 1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${escapeHtml(step.id)}</span>
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeBatchStep(${idx})">
            ✕ Remove
          </button>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 6px;">
          <div class="form-group" style="margin: 0;">
            <label class="form-label" style="font-size: 11px;">Target Capability</label>
            <select class="form-input" style="font-size: 11.5px;" onchange="window.app.updateBatchStepCapability(${idx}, this.value)">
              <option value="">-- Select Capability --</option>
              ${optionsHtml}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${step.continue_on_error ? 'checked' : ''} onchange="window.app.updateBatchStepContinueOnError(${idx}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${stepSchemaBadgesHtml}

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
              ${stepCap ? `
                <button type="button" class="btn btn-ghost" style="padding: 1px 6px; font-size: 9.5px;" onclick="window.app.fillBatchStepSampleArgs(${idx})">✨ Sample Args</button>
              ` : ''}
            </div>
            <div style="display: flex; gap: 6px; font-size: 10px; color: var(--cyan-400); font-family: var(--ff-mono);">
              <span>Helpers:</span>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${idx}, '\${steps[0].result.id}')">\${steps[0].result.id}</code>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${idx}, '\${steps[0].result.data}')">\${steps[0].result.data}</code>
            </div>
          </div>
          <textarea 
            id="batch-step-args-${idx}"
            class="form-textarea" 
            rows="3" 
            style="font-size: 11px; font-family: var(--ff-mono);" 
            oninput="window.app.updateBatchStepArgs(${idx}, this.value)"
          >${escapeHtml(step.argsJson)}</textarea>
        </div>
      </div>
    `;
  }).join('');

  return `
    <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.75); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 24px;" onclick="if(event.target === this) window.app.closeBatchModal()">
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); width: 840px; max-width: 95vw; max-height: 90vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
        <div style="padding: 16px 20px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 16px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
              <span>⚡ Visual Multi-Step Batch Pipeline Builder</span>
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim); margin-top: 2px;">
              Execute chained MCP tools with variable reference interpolation and fault tolerance.
            </div>
          </div>
          <button class="btn btn-ghost" style="font-size: 16px; padding: 4px 8px;" onclick="window.app.closeBatchModal()">✕</button>
        </div>

        <div style="flex: 1; overflow-y: auto; padding: 20px;">
          ${stepsHtml}

          <div style="display: flex; gap: 10px; margin-top: 14px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.addBatchStep()">
              + Add Pipeline Step
            </button>
          </div>
        </div>

        <div style="padding: 14px 20px; border-top: 1px solid var(--border); background: var(--bg-app); display: flex; justify-content: space-between; align-items: center;">
          <div style="font-size: 11.5px; color: var(--text-dim);">
            ${steps.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${steps.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}


