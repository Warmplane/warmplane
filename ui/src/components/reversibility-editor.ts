export function renderReversibilityEditor(): string {
  return `
    <section id="reversibility-editor" data-alias-form-section="tool" style="margin-top:16px;padding:16px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface-hover);">
      <div style="display:flex;justify-content:space-between;gap:16px;align-items:flex-start;margin-bottom:12px;">
        <div>
          <h3 style="margin:0 0 4px;font-size:14px;">Execution reversibility</h3>
          <p class="text-dim" style="margin:0;font-size:11px;">Define how Warmplane handles this capability after it is called.</p>
        </div>
        <label for="reversibility-class" style="min-width:220px;font-size:11px;color:var(--text-dim);">Contract class
          <select class="form-input" id="reversibility-class" onchange="window.app.updateReversibilityEditor()" style="display:block;margin-top:4px;">
            <option value="">Not configured</option>
            <option value="delayable">Delayable · hold for cancellation</option>
            <option value="reversible">Reversible · restore prior state</option>
            <option value="compensable">Compensable · apply a compensating action</option>
            <option value="irreversible">Irreversible · explicitly no inverse</option>
          </select>
        </label>
      </div>
      <p id="reversibility-class-help" class="text-dim" style="font-size:11px;margin:0 0 12px;">No reversibility contract is currently configured.</p>

      <div data-reversibility-class="delayable" hidden>
        ${numberField('hold-window-secs', 'Hold window (seconds)', 'Duration the call waits before dispatch; must be positive.')}
      </div>

      <div data-reversibility-class="compensable" hidden>
        ${numberField('daily-limit', 'Daily limit', 'Maximum compensable calls in the rolling daily window; must be positive.')}
      </div>

      <div data-reversibility-class="reversible compensable" hidden>
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;margin-bottom:14px;">
          ${numberField('ttl-secs', 'Undo window (seconds)', 'Optional lifetime for the registered inverse; must be positive when set.')}
          ${capabilityAutocompleteField('inverse-capability', 'Inverse capability', 'Canonical capability used to reverse or compensate for the call.')}
        </div>
        ${jsonObjectField('inverse-args', 'Inverse arguments template', 'JSON object supporting argument, result, and snapshot template references.', '{\n  "id": "\${args.id}",\n  "value": "\${snapshot}"\n}')}
        ${optionalReadSection('pre-snapshot', 'Capture pre-call state', 'Read state before the forward call. Its captured value can be used through \${snapshot} in the inverse arguments.')}
        ${optionalReadSection('pre-undo-check', 'Check for state drift before undo', 'Read current state immediately before undo; a mismatch blocks the inverse.')}
        <section style="padding:12px 0 0;border-top:1px solid var(--border);margin-top:12px;">
          <label style="display:flex;gap:8px;align-items:center;font-size:12px;color:var(--text-main);">
            <input type="checkbox" id="test-target-enabled" onchange="window.app.updateReversibilityEditor()">
            Enable isolated undo contract testing
          </label>
          <div data-optional-field="test-target" hidden style="margin-top:10px;">
            ${textField('test-target-server', 'Dedicated test server', 'Server used only for the isolated forward/snapshot/inverse test.')}
            ${jsonObjectField('test-target-args', 'Fixed test arguments', 'JSON object sent to the forward call on the dedicated test server.', '{\n  "id": "UNDO-TEST-1"\n}')}
          </div>
          <p class="text-dim" style="font-size:10px;margin:8px 0 0;">Contract testing requires a pre-call snapshot and a dedicated non-production server.</p>
        </section>
      </div>
    </section>
  `;
}

function numberField(id: string, label: string, help: string): string {
  return `<label for="${id}" style="display:block;font-size:11px;color:var(--text-dim);margin-bottom:12px;">${label}
    <input class="form-input" id="${id}" type="number" min="1" step="1" style="display:block;margin-top:4px;">
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${help}</span>
  </label>`;
}

function textField(id: string, label: string, help: string): string {
  return `<label for="${id}" style="display:block;font-size:11px;color:var(--text-dim);margin-bottom:12px;">${label}
    <input class="form-input" id="${id}" type="text" style="display:block;margin-top:4px;">
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${help}</span>
  </label>`;
}

export function renderCapabilityInput(id: string, label: string, placeholder = ''): string {
  return `<div class="capability-autocomplete">
    <input class="form-input" id="${id}" type="text" role="combobox" aria-label="${label}" aria-autocomplete="list" aria-expanded="false" aria-controls="${id}-suggestions" autocomplete="off" placeholder="${placeholder}">
    <div id="${id}-suggestions" role="listbox" aria-label="${label} suggestions" class="capability-suggestions" hidden></div>
  </div>`;
}

function capabilityAutocompleteField(id: string, label: string, help: string): string {
  return `<div class="form-group">
    <label class="form-label" for="${id}">${label}</label>
    ${renderCapabilityInput(id, label)}
    <span class="text-dim" style="font-size:10px;">${help}</span>
  </div>`;
}

function jsonObjectField(id: string, label: string, help: string, placeholder: string): string {
  return `<label for="${id}" style="display:block;font-size:11px;color:var(--text-dim);margin:0 0 12px;">${label}
    <textarea class="form-input" id="${id}" rows="4" spellcheck="false" placeholder='${placeholder}' style="display:block;margin-top:4px;width:100%;font-family:var(--ff-mono);font-size:11px;resize:vertical;"></textarea>
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${help} Must be a JSON object.</span>
  </label>`;
}

function optionalReadSection(id: 'pre-snapshot' | 'pre-undo-check', title: string, help: string): string {
  const prefix = id;
  return `<section style="padding:12px 0 0;border-top:1px solid var(--border);margin-top:12px;">
    <label style="display:flex;gap:8px;align-items:center;font-size:12px;color:var(--text-main);">
      <input type="checkbox" id="${id}-enabled" onchange="window.app.updateReversibilityEditor()"> ${title}
    </label>
    <div data-optional-field="${id}" hidden style="margin-top:10px;">
      ${capabilityAutocompleteField(`${prefix}-capability`, 'Read capability', 'Canonical read capability to invoke.')}
      ${jsonObjectField(`${prefix}-args`, 'Read arguments template', help, '{\n  "id": "\${args.id}"\n}')}
      ${textField(`${prefix}-capture-json-path`, 'Captured JSONPath (optional)', 'Optional JSONPath to select the value to snapshot or hash.')}
    </div>
  </section>`;
}
