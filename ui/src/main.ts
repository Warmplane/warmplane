import { store } from './state';
import { api } from './api';
import { renderOverview } from './components/overview';
import { renderServers } from './components/servers';
import { renderPlayground } from './components/playground';
import { renderPolicy } from './components/policy';
import { renderAliases } from './components/aliases';
import { SERVER_TEMPLATES, ServerTemplate } from './templates';

class WarmplaneApp {
  private activeTemplateCategory: string = 'all';
  private activeTemplateFilter: string = '';
  private selectedTemplate: ServerTemplate | null = null;

  async init() {
    const port = window.location.port ? `:${window.location.port}` : '';
    const portLabel = document.getElementById('daemon-port-label');
    if (portLabel) portLabel.textContent = `Daemon ${port}`;

    await this.refreshData();
    this.initSSE();
    this.render();

    store.subscribe(() => {
      this.render();
    });
  }

  async refreshData() {
    try {
      const [configRes, capsRes] = await Promise.all([
        api.getConfig(),
        api.listCapabilities()
      ]);

      if (configRes.ok) {
        store.setState({
          configPath: configRes.config_path,
          config: configRes.config,
          serverStatuses: configRes.server_statuses || {},
          metrics: {
            totalCatalogRequests: configRes.metrics?.total_catalog_requests || 0,
            totalEtagHits: configRes.metrics?.total_etag_hits || 0,
            totalToolCalls: configRes.metrics?.total_tool_calls || 0,
            totalToolDurationUs: configRes.metrics?.total_tool_duration_us || 0,
          }
        });
      }

      if (capsRes && Array.isArray(capsRes.capabilities)) {
        store.setState({
          capabilities: capsRes.capabilities
        });
      }
    } catch (e) {
      console.error('Failed to fetch daemon state:', e);
    }
  }

  initSSE() {
    try {
      const sse = new EventSource('/v1/resources/updates');
      sse.onmessage = (e) => {
        store.addEventLog('SSE', '/v1/resources/updates', 'UPDATED', '0.1ms');
        this.refreshData();
      };
    } catch (e) {
      console.warn('SSE connection unavailable');
    }
  }

  switchTab(tab: 'overview' | 'servers' | 'playground' | 'policy' | 'aliases') {
    store.setState({ activeTab: tab });
    this.refreshData();
  }

  render() {
    const state = store.getState();
    const mainEl = document.getElementById('app-main');
    if (!mainEl) return;

    // Update nav item active states
    document.querySelectorAll('.nav-item').forEach(el => {
      const tabAttr = el.getAttribute('data-tab');
      if (tabAttr === state.activeTab) {
        el.classList.add('active');
      } else {
        el.classList.remove('active');
      }
    });

    // Update top title
    const titleEl = document.getElementById('top-title');
    const titles = {
      overview: 'Overview Cockpit',
      servers: 'Server Hub & Connections',
      playground: 'MCP Capability Playground',
      policy: 'Security Governance & Redaction',
      aliases: 'Facade & Alias Studio'
    };
    if (titleEl) titleEl.textContent = titles[state.activeTab];

    switch (state.activeTab) {
      case 'overview':
        mainEl.innerHTML = renderOverview();
        break;
      case 'servers':
        mainEl.innerHTML = renderServers();
        break;
      case 'playground':
        mainEl.innerHTML = renderPlayground();
        break;
      case 'policy':
        mainEl.innerHTML = renderPolicy();
        break;
      case 'aliases':
        mainEl.innerHTML = renderAliases();
        break;
    }
  }

  // Action Handlers
  selectCapability(id: string) {
    store.setState({ selectedCapabilityId: id });
  }

  filterCapabilities(query: string) {
    const q = query.toLowerCase().trim();
    const allCaps = store.getState().capabilities;
    const filtered = allCaps.filter(c => 
      c.id.toLowerCase().includes(q) ||
      (c.summary && c.summary.toLowerCase().includes(q)) ||
      (c.server && c.server.toLowerCase().includes(q))
    );
    const listEl = document.getElementById('pg-cap-list');
    if (listEl) {
      if (filtered.length === 0) {
        listEl.innerHTML = `
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${escapeHtml(query)}"
          </div>
        `;
      } else {
        listEl.innerHTML = filtered.map(c => `
          <div class="cap-item ${c.id === store.getState().selectedCapabilityId ? 'active' : ''}" onclick="window.app.selectCapability('${escapeHtml(c.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${escapeHtml(c.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${escapeHtml(c.mode || 'read')}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${escapeHtml(c.server || 'local')}</div>
          </div>
        `).join('');
      }
    }
  }

  async executePlaygroundTool() {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    if (!capId) return;

    const argsText = (document.getElementById('pg-args-input') as HTMLTextAreaElement)?.value || '{}';
    const contextVal = (document.getElementById('pg-context-input') as HTMLInputElement)?.value || undefined;

    let parsedArgs = {};
    try {
      parsedArgs = JSON.parse(argsText);
    } catch {
      alert('Invalid arguments JSON object');
      return;
    }

    const statusBadge = document.getElementById('pg-status-badge');
    const responseJson = document.getElementById('pg-response-json');
    if (statusBadge) {
      statusBadge.textContent = 'EXECUTING...';
      statusBadge.style.color = 'var(--amber-400)';
    }

    try {
      const res = await api.callCapability({
        capability_id: capId,
        args: parsedArgs,
        context: contextVal ? { operation_id: contextVal } : undefined,
        request_id: `ui-req-${Date.now()}`
      });

      // Update store so result is preserved across re-renders
      store.setState({
        executionResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });

      store.addEventLog('POST', `/v1/tools/call → ${capId}`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);
    } catch (e: any) {
      if (statusBadge) {
        statusBadge.textContent = 'ERROR';
        statusBadge.style.color = 'var(--red-400)';
      }
      if (responseJson) {
        responseJson.textContent = e.toString();
      }
    }
  }

  // Policy Actions
  async submitPolicyRule(type: 'allow' | 'deny' | 'redact') {
    const inputId = type === 'allow' ? 'policy-new-allow' : type === 'deny' ? 'policy-new-deny' : 'policy-new-redact';
    const inputEl = document.getElementById(inputId) as HTMLInputElement | null;
    if (!inputEl) return;
    const val = inputEl.value.trim();
    if (!val) return;
    await this.addPolicyRule(type, val);
    inputEl.value = '';
  }

  async addPolicyRule(type: 'allow' | 'deny' | 'redact', val: string) {
    const trimmed = (val || '').trim();
    if (!trimmed) return;
    const state = store.getState();
    const current = state.config.policy || {};
    const allow = [...(current.allow || [])];
    const deny = [...(current.deny || [])];
    const redact = [...(current.redact_keys || current.redactKeys || [])];

    if (type === 'allow' && !allow.includes(trimmed)) allow.push(trimmed);
    if (type === 'deny' && !deny.includes(trimmed)) deny.push(trimmed);
    if (type === 'redact' && !redact.includes(trimmed)) redact.push(trimmed);

    const res = await api.savePolicy({ allow, deny, redact_keys: redact, redactKeys: redact });
    if (!res.ok) {
      alert(`Failed to save policy rule: ${res.error || 'Unknown error'}`);
    }
    await this.refreshData();
  }

  async removePolicyRule(type: 'allow' | 'deny' | 'redact', index: number) {
    const state = store.getState();
    const current = state.config.policy || {};
    const allow = [...(current.allow || [])];
    const deny = [...(current.deny || [])];
    const redact = [...(current.redact_keys || current.redactKeys || [])];

    if (type === 'allow') allow.splice(index, 1);
    if (type === 'deny') deny.splice(index, 1);
    if (type === 'redact') redact.splice(index, 1);

    const res = await api.savePolicy({ allow, deny, redact_keys: redact, redactKeys: redact });
    if (!res.ok) {
      alert(`Failed to update policy: ${res.error || 'Unknown error'}`);
    }
    await this.refreshData();
  }

  testPolicySandbox(id: string) {
    const verdictEl = document.getElementById('policy-test-verdict');
    if (!verdictEl) return;
    const testId = id.trim();
    if (!testId) {
      verdictEl.textContent = 'ENTER ID';
      verdictEl.style.color = 'var(--text-dim)';
      return;
    }

    const state = store.getState();
    const policy = state.config.policy || {};
    const deny = policy.deny || [];
    const allow = policy.allow || [];

    const wildcard = (pat: string, v: string) => {
      if (pat === '*') return true;
      if (pat.endsWith('*')) return v.startsWith(pat.slice(0, -1));
      return pat === v;
    };

    if (deny.some(d => wildcard(d, testId))) {
      verdictEl.textContent = 'DENIED (Strict Block)';
      verdictEl.style.color = 'var(--red-400)';
      return;
    }

    if (allow.length > 0 && !allow.some(a => wildcard(a, testId))) {
      verdictEl.textContent = 'DENIED (Not in Allow List)';
      verdictEl.style.color = 'var(--red-400)';
      return;
    }

    verdictEl.textContent = 'ALLOWED';
    verdictEl.style.color = 'var(--green-400)';
  }

  // Server Actions
  async deleteServer(name: string) {
    if (!confirm(`Are you sure you want to remove server '${name}' from config?`)) return;
    await api.deleteServer(name);
    await this.refreshData();
  }

  openAddServerModal() {
    this.closeModals();
    const modal = document.getElementById('modal-add-server');
    if (modal) modal.classList.add('active');
  }

  async submitAddServer() {
    const name = (document.getElementById('modal-srv-name') as HTMLInputElement)?.value.trim();
    const transport = (document.getElementById('modal-srv-transport') as HTMLSelectElement)?.value;
    if (!name) {
      alert('Server name is required');
      return;
    }

    let serverPayload: any = {};
    if (transport === 'stdio') {
      const cmdStr = (document.getElementById('modal-srv-command') as HTMLInputElement)?.value.trim();
      const parts = cmdStr.split(/\s+/).filter(Boolean);
      if (parts.length === 0) {
        alert('Command is required');
        return;
      }
      serverPayload.command = parts[0];
      serverPayload.args = parts.slice(1);
    } else {
      const url = (document.getElementById('modal-srv-url') as HTMLInputElement)?.value.trim();
      if (!url) {
        alert('URL is required');
        return;
      }
      serverPayload.url = url;
    }

    const res = await api.upsertServer(name, serverPayload);
    if (res.ok) {
      this.closeModals();
      await this.refreshData();
    } else {
      alert(`Failed to save server: ${res.error}`);
    }
  }

  // ==========================================
  // Template Catalog Actions
  // ==========================================
  openTemplateCatalog() {
    this.closeModals();
    const modal = document.getElementById('modal-templates');
    if (modal) modal.classList.add('active');
    this.renderTemplateGrid();
  }

  setTemplateCategory(cat: string) {
    this.activeTemplateCategory = cat;
    document.querySelectorAll('.tmpl-cat-btn').forEach(btn => {
      if (btn.getAttribute('data-category') === cat) {
        btn.classList.add('active');
        (btn as HTMLElement).style.background = 'var(--surface-elevated)';
        (btn as HTMLElement).style.color = 'var(--amber-400)';
      } else {
        btn.classList.remove('active');
        (btn as HTMLElement).style.background = 'var(--surface-card)';
        (btn as HTMLElement).style.color = 'var(--text-main)';
      }
    });
    this.renderTemplateGrid();
  }

  filterTemplates(query: string) {
    this.activeTemplateFilter = query.toLowerCase().trim();
    this.renderTemplateGrid();
  }

  renderTemplateGrid() {
    const gridEl = document.getElementById('tmpl-grid');
    if (!gridEl) return;

    const filtered = SERVER_TEMPLATES.filter(t => {
      const matchesCat = this.activeTemplateCategory === 'all' || t.category === this.activeTemplateCategory;
      const matchesFilter = !this.activeTemplateFilter ||
        t.name.toLowerCase().includes(this.activeTemplateFilter) ||
        t.id.toLowerCase().includes(this.activeTemplateFilter) ||
        t.description.toLowerCase().includes(this.activeTemplateFilter) ||
        t.command.toLowerCase().includes(this.activeTemplateFilter) ||
        t.envFields.some(e => e.key.toLowerCase().includes(this.activeTemplateFilter));
      return matchesCat && matchesFilter;
    });

    if (filtered.length === 0) {
      gridEl.innerHTML = `
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;
      return;
    }

    const state = store.getState();
    const configuredServers = state.config.mcpServers || {};

    gridEl.innerHTML = filtered.map(t => {
      const isAlreadyConfigured = !!configuredServers[t.id];
      const cmdPreview = `${t.command} ${t.defaultArgs.join(' ')}`;

      return `
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main);">${escapeHtml(t.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px;">${escapeHtml(t.badge)}</span>
              </div>
              ${isAlreadyConfigured ? '<span style="font-size: 10px; color: var(--green-400); font-weight: 600;">CONNECTED</span>' : ''}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${escapeHtml(t.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${escapeHtml(cmdPreview)}</code>
            </div>
            ${t.envFields.length > 0 ? `
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px;">
                <span>⚡ Needs:</span>
                <code>${t.envFields.map(e => escapeHtml(e.key)).join(', ')}</code>
              </div>
            ` : ''}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${escapeHtml(t.id)}')">
              ${isAlreadyConfigured ? 'Configure Another' : '✨ 1-Click Setup'}
            </button>
          </div>
        </div>
      `;
    }).join('');
  }

  selectTemplate(templateId: string) {
    const tmpl = SERVER_TEMPLATES.find(t => t.id === templateId);
    if (!tmpl) return;
    this.selectedTemplate = tmpl;

    this.closeModals();
    const modal = document.getElementById('modal-configure-template');
    if (modal) modal.classList.add('active');

    const titleEl = document.getElementById('cfg-tmpl-title');
    const descEl = document.getElementById('cfg-tmpl-desc');
    const formEl = document.getElementById('cfg-tmpl-form');

    if (titleEl) titleEl.textContent = `Configure ${tmpl.name} Server`;
    if (descEl) descEl.textContent = tmpl.description;

    if (formEl) {
      let envHtml = '';
      if (tmpl.envFields.length > 0) {
        envHtml = `
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${tmpl.envFields.map(ef => `
            <div class="form-group">
              <label class="form-label">${escapeHtml(ef.label)} ${ef.required ? '<span style="color: var(--red-400);">*</span>' : '(Optional)'}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${escapeHtml(ef.key)}" placeholder="${escapeHtml(ef.placeholder || '')}">
              ${ef.description ? `<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${escapeHtml(ef.description)}</div>` : ''}
            </div>
          `).join('')}
        `;
      }

      formEl.innerHTML = `
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${escapeHtml(tmpl.id)}">
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${escapeHtml(tmpl.defaultArgs.join(' '))}" placeholder="${escapeHtml(tmpl.argsPlaceholder || '')}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${escapeHtml(tmpl.command)}</code></div>
        </div>
        ${envHtml}
      `;
    }
  }

  async submitTemplateServer() {
    if (!this.selectedTemplate) return;
    const tmpl = this.selectedTemplate;
    const serverId = (document.getElementById('cfg-srv-id') as HTMLInputElement)?.value.trim();
    const argsStr = (document.getElementById('cfg-srv-args') as HTMLInputElement)?.value.trim();

    if (!serverId) {
      alert('Server identifier is required');
      return;
    }

    const args = argsStr ? argsStr.split(/\s+/).filter(Boolean) : [];
    const env: Record<string, string> = {};

    const envInputs = document.querySelectorAll('.tmpl-env-input') as NodeListOf<HTMLInputElement>;
    for (const inp of Array.from(envInputs)) {
      const k = inp.getAttribute('data-key');
      const v = inp.value.trim();
      const def = tmpl.envFields.find(e => e.key === k);
      if (def?.required && !v) {
        alert(`Required field '${def.label}' is missing.`);
        return;
      }
      if (k && v) {
        env[k] = v;
      }
    }

    const payload: any = {
      command: tmpl.command,
      args: args
    };
    if (Object.keys(env).length > 0) {
      payload.env = env;
    }

    const res = await api.upsertServer(serverId, payload);
    if (res.ok) {
      this.closeModals();
      await this.refreshData();
    } else {
      alert(`Failed to save server: ${res.error}`);
    }
  }

  // Ecosystem Actions
  async openImportModal() {
    this.closeModals();
    const modal = document.getElementById('modal-import');
    if (modal) modal.classList.add('active');
    const container = document.getElementById('modal-eco-list');
    if (!container) return;

    container.innerHTML = '<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';

    try {
      const res = await api.getEcosystemSources();
      if (res.sources && res.sources.length > 0) {
        container.innerHTML = res.sources.map(s => `
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${s.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${s.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${s.server_count} servers (${s.servers.join(', ')})</div>
            </div>
          </label>
        `).join('');
      } else {
        container.innerHTML = '<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>';
      }
    } catch {
      container.innerHTML = '<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>';
    }
  }

  async submitImport() {
    const checkboxes = document.querySelectorAll('.eco-checkbox:checked') as NodeListOf<HTMLInputElement>;
    if (checkboxes.length === 0) {
      alert('No sources selected');
      return;
    }

    for (const cb of Array.from(checkboxes)) {
      await api.importConfig(cb.value, false);
    }

    this.closeModals();
    await this.refreshData();
  }

  // Alias Actions
  handleAliasTargetInput(val: string) {
    const dropdown = document.getElementById('alias-suggestions-dropdown');
    if (!dropdown) return;
    const query = (val || '').trim().toLowerCase();
    if (query.length < 2) {
      dropdown.style.display = 'none';
      return;
    }

    const state = store.getState();
    const matches = state.capabilities.filter(c =>
      c.id.toLowerCase().includes(query) ||
      (c.summary && c.summary.toLowerCase().includes(query)) ||
      (c.description && c.description.toLowerCase().includes(query)) ||
      (c.server && c.server.toLowerCase().includes(query))
    ).slice(0, 8);

    if (matches.length === 0) {
      dropdown.style.display = 'none';
      return;
    }

    dropdown.innerHTML = matches.map(c => `
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${escapeHtml(c.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${escapeHtml(c.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${escapeHtml(c.summary || c.description || '')}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${escapeHtml(c.server || 'local')}</span>
      </div>
    `).join('');
    dropdown.style.display = 'block';
  }

  selectAliasSuggestion(id: string) {
    const input = document.getElementById('alias-target') as HTMLInputElement | null;
    if (input) {
      input.value = id;
    }
    this.hideAliasDropdown();
  }

  hideAliasDropdown() {
    const dropdown = document.getElementById('alias-suggestions-dropdown');
    if (dropdown) dropdown.style.display = 'none';
  }

  async createAlias() {
    const kind = (document.getElementById('alias-kind') as HTMLSelectElement)?.value;
    const name = (document.getElementById('alias-name') as HTMLInputElement)?.value.trim();
    const target = (document.getElementById('alias-target') as HTMLInputElement)?.value.trim();

    if (!name || !target) {
      alert('Please provide both alias name and canonical target');
      return;
    }

    await api.updateAlias(kind, name, target);
    await this.refreshData();
  }

  async deleteAlias(kind: string, name: string) {
    await api.updateAlias(kind, name, undefined);
    await this.refreshData();
  }

  async reloadFromDisk() {
    try {
      const res = await api.reloadConfig();
      if (res.ok) {
        let msg = 'Hot-reload completed successfully!';
        if (res.mounted && res.mounted.length > 0) msg += `\nMounted: ${res.mounted.join(', ')}`;
        if (res.unmounted && res.unmounted.length > 0) msg += `\nUnmounted: ${res.unmounted.join(', ')}`;
        if (res.warnings && res.warnings.length > 0) msg += `\nWarnings:\n${res.warnings.join('\n')}`;
        alert(msg);
      } else {
        alert(`Hot-reload failed: ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error reaching daemon: ${e.message}`);
    }
    await this.refreshData();
  }

  closeModals() {
    document.querySelectorAll('.modal-backdrop').forEach(el => el.classList.remove('active'));
  }
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

const app = new WarmplaneApp();
(window as any).app = app;
window.addEventListener('DOMContentLoaded', () => app.init());

