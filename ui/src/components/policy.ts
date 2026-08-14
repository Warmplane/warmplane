import { store } from '../state';
import { api } from '../api';

export function renderPolicy(): string {
  const state = store.getState();
  const policy = state.config.policy || {};
  const allow = policy.allow || [];
  const deny = policy.deny || [];
  const redact = policy.redact_keys || [];

  const allowHtml = allow.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  ` : allow.map((a, i) => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${escapeHtml(a)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${i})">✕</button>
    </div>
  `).join('');

  const denyHtml = deny.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  ` : deny.map((d, i) => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${escapeHtml(d)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${i})">✕</button>
    </div>
  `).join('');

  const redactHtml = redact.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  ` : redact.map((r, i) => `
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${escapeHtml(r)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${i})">✕</span>
    </span>
  `).join('');

  return `
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Security Governance &amp; Data Redaction</div>
        <div style="font-size: 11px; color: var(--text-dim);">Wildcard capability access control and sensitive key masking</div>
      </div>
    </div>

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-6">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${allowHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-allow" placeholder="e.g. github.*, db.read_*">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('allow', document.getElementById('policy-new-allow').value)">Add Allow</button>
        </div>
      </div>

      <!-- Deny Rules -->
      <div class="bento-card col-6">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--red-400);">Deny List Patterns (Strict Precedence)</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${denyHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-deny" placeholder="e.g. *.drop_*, filesystem.write_*">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('deny', document.getElementById('policy-new-deny').value)">Add Deny</button>
        </div>
      </div>

      <!-- Key Redaction -->
      <div class="bento-card col-12">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--amber-300);">Sensitive Key Redaction Patterns</span>
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Keys automatically masked as &lt;redacted&gt; in logs and envelopes</span>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; margin: 12px 0;">
          ${redactHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px; max-width: 420px;">
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('redact', document.getElementById('policy-new-redact').value)">Add Key</button>
        </div>
      </div>

      <!-- Evaluation Sandbox Tester -->
      <div class="bento-card col-12">
        <div class="stat-header">
          <span class="stat-label">Policy Evaluation Sandbox</span>
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Live verification of capability access against active rules</span>
        </div>
        <div style="display: flex; gap: 12px; align-items: center; margin-top: 12px;">
          <input type="text" class="form-input" placeholder="Type capability identifier, e.g. github.create_issue or sqlite.drop_table" style="flex: 1;" oninput="window.app.testPolicySandbox(this.value)">
          <div id="policy-test-verdict" style="font-weight: 700; font-family: var(--ff-mono); font-size: 12.5px; padding: 7px 16px; border-radius: var(--radius-sm); background: var(--surface); border: 1px solid var(--border); color: var(--green-400);">
            ALLOWED
          </div>
        </div>
      </div>
    </div>
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
