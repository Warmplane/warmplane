import { store } from '../state';
import { api } from '../api';

export function renderPolicy(): string {
  const state = store.getState();
  const activeProfName = state.activeProfile;
  const activeProfile = activeProfName ? state.config.profiles?.[activeProfName] : undefined;

  // If a profile is active in UI, view & edit that profile's policy; otherwise global daemon policy
  const isProfileScope = !!activeProfile;
  const basePolicy = state.config.policy || {};
  const profPolicy = activeProfile?.policy;
  
  // Use active profile's policy if profile is selected, else global
  const policy = isProfileScope ? (profPolicy || {}) : basePolicy;
  const allow = policy.allow || [];
  const deny = policy.deny || [];
  const redact = policy.redact_keys || policy.redactKeys || [];
  const requireApproval = policy.require_approval || policy.requireApproval || [];

  const allowHtml = allow.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">${isProfileScope ? 'No profile allow list (inherits global rules)' : 'No allow list (all non-denied operations permitted)'}</div>
  ` : allow.map((a, i) => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${escapeHtml(a)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${i})">✕</button>
    </div>
  `).join('');

  const denyHtml = deny.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">${isProfileScope ? 'No profile deny rules configured' : 'No deny rules configured'}</div>
  ` : deny.map((d, i) => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${escapeHtml(d)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${i})">✕</button>
    </div>
  `).join('');

  const approvalHtml = requireApproval.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">${isProfileScope ? 'No profile human-in-the-loop triggers configured' : 'No human-in-the-loop approval rules configured'}</div>
  ` : requireApproval.map((ap, i) => `
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">🛡️ ${escapeHtml(ap)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${i})">✕</button>
    </div>
  `).join('');

  const redactHtml = redact.length === 0 ? `
    <div style="color: var(--text-dim); font-size: 12px;">${isProfileScope ? 'No profile key redaction patterns configured' : 'No key redaction patterns configured'}</div>
  ` : redact.map((r, i) => `
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${escapeHtml(r)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${i})">✕</span>
    </span>
  `).join('');

  const scopeBannerHtml = isProfileScope ? `
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">🛡️</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Viewing &amp; Editing Policy for Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${escapeHtml(activeProfName!)}</code>
          </div>
          <div style="font-size: 11.5px; color: var(--text-muted); margin-top: 2px;">
            Rules defined here apply specifically when requests target this profile. Deny and HITL rules are strictly additive with global rules.
          </div>
        </div>
      </div>
      <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.setActiveProfile(null)">Switch to Global Policy</button>
    </div>
  ` : `
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Global security policy rules governing wildcard access control, human-in-the-loop triggers, and sensitive key masking. (Select an active profile in the top bar to edit per-profile rules).
    </div>
  `;

  return `
    ${scopeBannerHtml}

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${allowHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-allow" placeholder="e.g. github.*, db.read_*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('allow')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('allow')">Add Allow</button>
        </div>
      </div>

      <!-- Deny Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--red-400);">Deny List Patterns (Strict Precedence)</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${denyHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-deny" placeholder="e.g. *.drop_*, filesystem.write_*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('deny')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('deny')">Add Deny</button>
        </div>
      </div>

      <!-- Require Approval Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--amber-400);">Human-in-the-Loop Triggers</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${approvalHtml}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-requireApproval" placeholder="e.g. docker.run*, db.write*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('requireApproval')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('requireApproval')">Add Approval</button>
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
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('redact')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('redact')">Add Key</button>
        </div>
      </div>

      <!-- Webhooks & ChatOps Alerts -->
      <div class="bento-card col-12" style="border-color: rgba(59, 130, 246, 0.3);">
        <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
          <div>
            <span class="stat-label" style="color: var(--cyan-400);">⚡ ChatOps &amp; Outbound Webhooks</span>
            <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Push actionable HITL approval cards and alerts directly to Slack, Discord, or Teams</span>
          </div>
          <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">Bidirectional</span>
        </div>

        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 14px;">
          <div>
            <label class="form-label" style="font-size: 11px;">Target Webhook URL</label>
            <input type="text" class="form-input" id="policy-webhook-url" placeholder="https://hooks.slack.com/services/... or Discord webhook URL" value="${escapeHtml(typeof policy.webhook === 'object' && policy.webhook ? policy.webhook.url || '' : '')}">
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">Payload Layout Format</label>
            <select class="form-input" id="policy-webhook-format">
              <option value="slack" ${(typeof policy.webhook === 'object' && policy.webhook?.format === 'slack') ? 'selected' : ''}>Slack Block Kit (Interactive)</option>
              <option value="discord" ${(typeof policy.webhook === 'object' && policy.webhook?.format === 'discord') ? 'selected' : ''}>Discord Embed &amp; Actions</option>
              <option value="teams" ${(typeof policy.webhook === 'object' && policy.webhook?.format === 'teams') ? 'selected' : ''}>Microsoft Teams Adaptive Cards</option>
              <option value="generic" ${(typeof policy.webhook === 'object' && policy.webhook?.format === 'generic') || !policy.webhook ? 'selected' : ''}>Generic JSON (Standard)</option>
            </select>
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">HMAC Secret (or Env Var)</label>
            <input type="text" class="form-input" id="policy-webhook-secret" placeholder="e.g. WARMPLANE_WEBHOOK_SECRET" value="${escapeHtml(typeof policy.webhook === 'object' && policy.webhook ? policy.webhook.secret_env || policy.webhook.secretEnv || policy.webhook.secret || '' : '')}">
          </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-primary" onclick="window.app.saveWebhookConfig()">Save Webhook Settings</button>
            <button class="btn btn-ghost" onclick="window.app.testWebhook()">⚡ Send Test Event</button>
          </div>
          <div id="policy-webhook-status" style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">
            ${typeof policy.webhook === 'object' && policy.webhook?.url ? `Active Target: ${escapeHtml(policy.webhook.url)}` : 'No webhook configured'}
          </div>
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
