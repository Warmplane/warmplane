import { store } from './state';
import { api } from './api';
import { renderOverview } from './components/overview';
import { renderServers } from './components/servers';
import { renderPlayground } from './components/playground';
import { renderPolicy } from './components/policy';
import { renderAliases } from './components/aliases';

class WarmplaneApp {
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

      if (capsRes.ok) {
        store.setState({
          capabilities: capsRes.capabilities || []
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
    const listEl = document.querySelector('.playground-sidebar div:last-child');
    if (listEl) {
      listEl.innerHTML = filtered.map(c => `
        <div class="cap-item ${c.id === store.getState().selectedCapabilityId ? 'active' : ''}" onclick="window.app.selectCapability('${c.id}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c.id}</span>
            <span style="font-size: 10px; color: var(--green-400);">${c.mode || 'read'}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${c.server || 'local'}</div>
        </div>
      `).join('');
    }
  }

  async executePlaygroundTool() {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    if (!capId) return;

    const argsText = (document.getElementById('pg-args-input') as HTMLTextAreaElement)?.value || '{}';
    const contextVal = (document.getElementById('pg-context-input') as HTMLInputElement)?.value || undefined;
    const statusBadge = document.getElementById('pg-status-badge');
    const responseJson = document.getElementById('pg-response-json');

    let parsedArgs = {};
    try {
      parsedArgs = JSON.parse(argsText);
    } catch {
      alert('Invalid arguments JSON object');
      return;
    }

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

      if (statusBadge) {
        statusBadge.textContent = `HTTP ${res.status} · ${res.durationMs.toFixed(1)}ms`;
        statusBadge.style.color = res.status === 200 ? 'var(--green-400)' : 'var(--red-400)';
      }
      if (responseJson) {
        responseJson.textContent = JSON.stringify(res.data, null, 2);
      }

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
  async addPolicyRule(type: 'allow' | 'deny' | 'redact', val: string) {
    if (!val || !val.trim()) return;
    const state = store.getState();
    const current = state.config.policy || {};
    const allow = [...(current.allow || [])];
    const deny = [...(current.deny || [])];
    const redact = [...(current.redact_keys || [])];

    if (type === 'allow' && !allow.includes(val.trim())) allow.push(val.trim());
    if (type === 'deny' && !deny.includes(val.trim())) deny.push(val.trim());
    if (type === 'redact' && !redact.includes(val.trim())) redact.push(val.trim());

    await api.savePolicy({ allow, deny, redact_keys: redact });
    await this.refreshData();
  }

  async removePolicyRule(type: 'allow' | 'deny' | 'redact', index: number) {
    const state = store.getState();
    const current = state.config.policy || {};
    const allow = [...(current.allow || [])];
    const deny = [...(current.deny || [])];
    const redact = [...(current.redact_keys || [])];

    if (type === 'allow') allow.splice(index, 1);
    if (type === 'deny') deny.splice(index, 1);
    if (type === 'redact') redact.splice(index, 1);

    await api.savePolicy({ allow, deny, redact_keys: redact });
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

  // Ecosystem Actions
  async openImportModal() {
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

const app = new WarmplaneApp();
(window as any).app = app;
window.addEventListener('DOMContentLoaded', () => app.init());
