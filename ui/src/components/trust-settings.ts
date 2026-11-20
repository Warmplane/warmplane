import type { PolicyConfig, WebhookConfig } from '../api';
import { escapeHtml } from '../escape';

type ResultRules = NonNullable<PolicyConfig['resultSanitization']>;
const approvalEvent = 'approval.requested';
const quarantineEvent = 'tool_pin.quarantined';

export function renderResultSanitization(rules: ResultRules | undefined): string {
  return `
    <form id="result-sanitization-form" data-policy-key="${escapeHtml(JSON.stringify(rules ?? null))}" class="bento-card col-12 trust-settings" onsubmit="window.app.saveResultSanitization(event)">
      <h2 class="stat-label">Result sanitization</h2>
      <label class="trust-checkbox"><input name="enabled" type="checkbox" ${rules ? 'checked' : ''}> Inspect upstream results before delivery</label>
      <div class="trust-fields">
        <div class="form-group">
          <label class="form-label" for="result-fields">Structured field names · one per line</label>
          <textarea class="form-textarea" id="result-fields" name="fields" rows="3">${escapeHtml((rules?.fieldNames ?? []).join('\n'))}</textarea>
        </div>
        <div class="form-group">
          <label class="form-label" for="result-patterns">Regular expressions · one per line</label>
          <textarea class="form-textarea" id="result-patterns" name="patterns" rows="3">${escapeHtml((rules?.patterns ?? []).join('\n'))}</textarea>
        </div>
      </div>
      <div class="form-group">
        <label class="form-label" for="result-exceptions">Opaque content exceptions · JSON array</label>
        <textarea class="form-textarea" id="result-exceptions" name="exceptions" rows="3" spellcheck="false"
          placeholder='[{"server":"reports","kind":"image","mimeType":"image/png"}]'>${escapeHtml(JSON.stringify(rules?.opaqueExceptions ?? [], null, 2))}</textarea>
        <span class="tool-pin-feedback">Opaque content is withheld unless its server, kind, and MIME type match an exception.</span>
      </div>
      <div class="trust-actions">
        <p class="tool-pin-feedback" role="status"></p>
        <button type="submit" class="btn btn-primary">Save result policy</button>
      </div>
    </form>
  `;
}

function webhookField(name: string, label: string, value?: string, type = 'text'): string {
  return `
    <div class="form-group">
      <label class="form-label" for="webhook-${name}">${label}</label>
      <input class="form-input" id="webhook-${name}" name="${name}" type="${type}" value="${escapeHtml(value ?? '')}" ${type === 'password' ? 'autocomplete="new-password"' : ''}>
    </div>
  `;
}

export function renderWebhookSettings(config: PolicyConfig['webhook'], rbacEnabled: boolean): string {
  const webhook = typeof config === 'string' ? { url: config } : config;
  const formats = [['generic', 'Generic JSON'], ['slack', 'Slack'], ['teams', 'Microsoft Teams'], ['discord', 'Discord']];
  return `
    <form id="chatops-settings-form" data-policy-key="${escapeHtml(JSON.stringify([config ?? null, rbacEnabled]))}" class="bento-card col-12 trust-settings" onsubmit="window.app.saveWebhookConfig(event)">
      <h2 class="stat-label">ChatOps and outbound webhooks</h2>
      <div class="trust-fields">
        ${webhookField('url', 'Destination URL', webhook?.url, 'url')}
        <div class="form-group">
          <label class="form-label" for="webhook-format">Format</label>
          <select class="form-input" id="webhook-format" name="format">
            ${formats.map(([id, label]) => `<option value="${id}" ${(webhook?.format ?? 'generic') === id ? 'selected' : ''}>${label}</option>`).join('')}
          </select>
        </div>
        ${webhookField('secret', 'Webhook signing secret', webhook?.secret, 'password')}
        ${webhookField('secretEnv', 'Webhook signing secret environment variable', webhook?.secretEnv ?? webhook?.secret_env)}
      </div>
      <fieldset>
        <legend>Signed actions</legend>
        <label class="trust-checkbox"><input name="approvals" type="checkbox" ${webhook?.events?.includes(approvalEvent) ? 'checked' : ''}> Send signed approval cards</label>
        <label class="trust-checkbox"><input name="quarantine" type="checkbox" ${webhook?.events?.includes(quarantineEvent) ? 'checked' : ''}> Send tool quarantine notifications</label>
        <div class="trust-fields">
          ${webhookField('callbackUrl', 'Callback URL ending in /v1/webhooks/actions', webhook?.callbackUrl ?? webhook?.callback_url, 'url')}
          ${webhookField('actionRole', 'Delegated action role', webhook?.actionRole)}
          ${webhookField('approvalTenant', rbacEnabled ? 'Approval tenant · required with RBAC' : 'Approval tenant', webhook?.approvalTenant)}
          ${webhookField('slackSigningSecret', 'Slack app signing secret', webhook?.slackSigningSecret, 'password')}
          ${webhookField('slackSigningSecretEnv', 'Slack signing secret environment variable', webhook?.slackSigningSecretEnv)}
        </div>
      </fieldset>
      <div class="trust-actions">
        <p class="tool-pin-feedback" role="status"></p>
        <button type="button" class="btn btn-ghost" onclick="window.app.testWebhook()" ${webhook?.url ? '' : 'disabled'}>Test saved destination</button>
        <button type="submit" class="btn btn-primary">Save webhook settings</button>
      </div>
    </form>
  `;
}

function text(data: FormData, name: string): string {
  return String(data.get(name) ?? '').trim();
}

export function readResultRules(data: FormData): ResultRules | undefined {
  if (!data.has('enabled')) return undefined;
  const fieldNames = text(data, 'fields').split('\n').map(line => line.trim()).filter(Boolean);
  const patterns = String(data.get('patterns') ?? '').split(/\r?\n/).filter(line => line.length);
  const exceptions: unknown = JSON.parse(text(data, 'exceptions') || '[]');
  if (!Array.isArray(exceptions)) throw new Error('Enter opaque content exceptions as a JSON array.');
  return { fieldNames, patterns, opaqueExceptions: exceptions };
}

export function readWebhook(data: FormData, previous: PolicyConfig['webhook'], rbacEnabled: boolean): WebhookConfig | undefined {
  const url = text(data, 'url');
  if (!url) {
    if (data.has('approvals') || data.has('quarantine')) throw new Error('Enter a destination URL for the enabled notifications.');
    return undefined;
  }
  const current = typeof previous === 'object' ? previous : undefined;
  const events = (current?.events ?? []).filter(event => event !== approvalEvent && event !== quarantineEvent);
  if (data.has('approvals')) events.push(approvalEvent);
  if (data.has('quarantine')) events.push(quarantineEvent);
  const callbackUrl = text(data, 'callbackUrl') || undefined;
  const actionRole = text(data, 'actionRole') || undefined;
  const approvalTenant = text(data, 'approvalTenant') || undefined;
  const secret = text(data, 'secret') || undefined;
  const secretEnv = text(data, 'secretEnv') || undefined;
  const slackSigningSecret = text(data, 'slackSigningSecret') || undefined;
  const slackSigningSecretEnv = text(data, 'slackSigningSecretEnv') || undefined;
  const format = (text(data, 'format') || 'generic') as WebhookConfig['format'];
  if (data.has('approvals')) {
    if (!callbackUrl || new URL(callbackUrl).pathname !== '/v1/webhooks/actions') throw new Error('Use a callback URL ending in /v1/webhooks/actions.');
    if (!actionRole) throw new Error('Enter the delegated action role for signed approvals.');
    if (rbacEnabled && !approvalTenant) throw new Error('Enter the approval tenant for signed approvals under RBAC.');
    if (format === 'slack' && !slackSigningSecret && !slackSigningSecretEnv) throw new Error('Enter the Slack app signing secret or its environment variable.');
  }
  if ((data.has('approvals') || data.has('quarantine')) && !secret && !secretEnv) {
    throw new Error('Enter the webhook signing secret or its environment variable.');
  }
  return {
    ...current, url, format, events, callbackUrl, actionRole, approvalTenant,
    secret, secretEnv, slackSigningSecret, slackSigningSecretEnv,
    callback_url: undefined, secret_env: undefined,
  };
}
