import { store } from './state';
import { api } from './api';
import { renderOverview } from './components/overview';
import { renderServers } from './components/servers';
import { renderPlayground } from './components/playground';
import { renderApprovals } from './components/approvals';
import { renderAudit } from './components/audit';
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

  private auditSearchTimeout: any = null;

  async refreshData() {
    try {
      const state = store.getState();
      const filters = state.auditFilters;
      const [configRes, capsRes, apprRes, auditEventsRes, auditStatsRes] = await Promise.all([
        api.getConfig(),
        api.listCapabilities(),
        api.listApprovals(),
        api.listAuditEvents({
          server_id: filters.serverId !== 'all' ? filters.serverId : undefined,
          event_type: filters.eventType !== 'all' ? filters.eventType : undefined,
          status: filters.status !== 'all' ? filters.status : undefined,
          search: filters.search.trim() ? filters.search.trim() : undefined,
          limit: filters.limit,
          offset: filters.offset,
        }),
        api.getAuditStats()
      ]);

      if (configRes.ok) {
        store.setState({
          configPath: configRes.config_path,
          config: configRes.config,
          serverStatuses: configRes.server_statuses || {},
          circuitBreakers: configRes.circuit_breakers || [],
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

      if (apprRes && Array.isArray(apprRes.approvals)) {
        store.setState({
          approvals: apprRes.approvals
        });
      }

      if (auditEventsRes && Array.isArray(auditEventsRes.events)) {
        store.setState({
          auditEvents: auditEventsRes.events,
          auditTotal: auditEventsRes.total ?? auditEventsRes.events.length,
        });
      }

      if (auditStatsRes && auditStatsRes.ok) {
        store.setState({
          auditStats: auditStatsRes
        });
      }
    } catch (e) {
      console.error('Failed to fetch daemon state:', e);
    }
  }

  async refreshAuditEvents() {
    try {
      const state = store.getState();
      const filters = state.auditFilters;
      const [auditEventsRes, auditStatsRes] = await Promise.all([
        api.listAuditEvents({
          server_id: filters.serverId !== 'all' ? filters.serverId : undefined,
          event_type: filters.eventType !== 'all' ? filters.eventType : undefined,
          status: filters.status !== 'all' ? filters.status : undefined,
          search: filters.search.trim() ? filters.search.trim() : undefined,
          limit: filters.limit,
          offset: filters.offset,
        }),
        api.getAuditStats()
      ]);
      if (auditEventsRes && Array.isArray(auditEventsRes.events)) {
        store.setState({
          auditEvents: auditEventsRes.events,
          auditTotal: auditEventsRes.total ?? auditEventsRes.events.length,
        });
      }
      if (auditStatsRes && auditStatsRes.ok) {
        store.setState({ auditStats: auditStatsRes });
      }
    } catch (e) {
      console.error('Failed to refresh audit events:', e);
    }
  }

  handleAuditSearchInput(val: string) {
    const state = store.getState();
    const newFilters = { ...state.auditFilters, search: val, offset: 0 };
    store.setState({ auditFilters: newFilters });
    clearTimeout(this.auditSearchTimeout);
    this.auditSearchTimeout = setTimeout(() => {
      this.refreshAuditEvents();
    }, 250);
  }

  handleAuditStatusFilter(status: string) {
    const state = store.getState();
    store.setState({ auditFilters: { ...state.auditFilters, status, offset: 0 } });
    this.refreshAuditEvents();
  }

  handleAuditEventTypeFilter(eventType: string) {
    const state = store.getState();
    store.setState({ auditFilters: { ...state.auditFilters, eventType, offset: 0 } });
    this.refreshAuditEvents();
  }

  handleAuditServerFilter(serverId: string) {
    const state = store.getState();
    store.setState({ auditFilters: { ...state.auditFilters, serverId, offset: 0 } });
    this.refreshAuditEvents();
  }

  handleAuditPageSize(sizeStr: string) {
    const size = parseInt(sizeStr, 10) || 25;
    const state = store.getState();
    store.setState({ auditFilters: { ...state.auditFilters, limit: size, offset: 0 } });
    this.refreshAuditEvents();
  }

  clearAuditFilters() {
    const state = store.getState();
    store.setState({
      auditFilters: {
        search: '',
        status: 'all',
        eventType: 'all',
        serverId: 'all',
        limit: state.auditFilters.limit || 25,
        offset: 0,
      }
    });
    this.refreshAuditEvents();
  }

  auditPrevPage() {
    const state = store.getState();
    const { limit, offset } = state.auditFilters;
    const newOffset = Math.max(0, offset - limit);
    if (newOffset !== offset) {
      store.setState({ auditFilters: { ...state.auditFilters, offset: newOffset } });
      this.refreshAuditEvents();
    }
  }

  auditNextPage() {
    const state = store.getState();
    const { limit, offset } = state.auditFilters;
    const total = state.auditTotal;
    if (offset + limit < total) {
      store.setState({ auditFilters: { ...state.auditFilters, offset: offset + limit } });
      this.refreshAuditEvents();
    }
  }

  auditGoToPage(pageNumber: number) {
    const state = store.getState();
    const { limit } = state.auditFilters;
    const newOffset = Math.max(0, (pageNumber - 1) * limit);
    store.setState({ auditFilters: { ...state.auditFilters, offset: newOffset } });
    this.refreshAuditEvents();
  }

  selectAuditEvent(id: string | null) {
    if (!id) {
      store.setState({ auditSelectedEvent: null });
      return;
    }
    const state = store.getState();
    const found = state.auditEvents.find(e => e.id === id) || null;
    store.setState({ auditSelectedEvent: found });
  }

  async verifyAuditChain() {
    try {
      const res = await api.verifyAuditChain();
      if (res && res.report) {
        store.setState({ auditVerification: res.report });
      }
    } catch (e) {
      console.error('Failed to verify audit chain:', e);
    }
  }

  async refreshApprovals() {
    try {
      const apprRes = await api.listApprovals();
      if (apprRes && Array.isArray(apprRes.approvals)) {
        store.setState({ approvals: apprRes.approvals });
      }
    } catch (e) {
      console.error('Failed to refresh approvals:', e);
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

  switchTab(tab: 'overview' | 'servers' | 'playground' | 'approvals' | 'audit' | 'policy' | 'aliases') {
    store.setState({ activeTab: tab });
    this.refreshData();
  }

  render() {
    const state = store.getState();
    const mainEl = document.getElementById('app-main');
    if (!mainEl) return;

    // Update nav item active states and badges
    const pendingCount = state.approvals.filter(a => a.status === 'pending').length;
    const badgeEl = document.getElementById('nav-approvals-badge');
    if (badgeEl) {
      badgeEl.textContent = pendingCount > 0 ? `${pendingCount}` : '';
      (badgeEl as HTMLElement).style.display = pendingCount > 0 ? 'inline-block' : 'none';
    }

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
      approvals: 'Human-in-the-Loop Review Queue',
      audit: 'WORM Audit & Compliance Ledger',
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
      case 'approvals':
        mainEl.innerHTML = renderApprovals(state);
        break;
      case 'audit':
        mainEl.innerHTML = renderAudit();
        break;
      case 'policy':
        mainEl.innerHTML = renderPolicy();
        break;
      case 'aliases':
        mainEl.innerHTML = renderAliases();
        break;
    }
  }

  // HITL Approval Actions
  async submitApproval(id: string) {
    const operatorInput = document.getElementById(`appr-operator-${id}`) as HTMLInputElement | null;
    const argsInput = document.getElementById(`appr-args-${id}`) as HTMLTextAreaElement | null;
    const operator = operatorInput?.value.trim() || 'security-operator';

    let modifiedArgs: any = undefined;
    if (argsInput && argsInput.value.trim()) {
      try {
        modifiedArgs = JSON.parse(argsInput.value.trim());
      } catch {
        alert('Invalid JSON in arguments editor');
        return;
      }
    }

    const res = await api.approveTicket(id, operator, modifiedArgs);
    if (res.ok) {
      await this.refreshApprovals();
    } else {
      alert(`Approval failed: ${res.error || 'Unknown error'}`);
    }
  }

  async promptReject(id: string) {
    const reason = prompt('Reason for rejection (will be returned to the calling agent):');
    if (reason === null) return;
    const operatorInput = document.getElementById(`appr-operator-${id}`) as HTMLInputElement | null;
    const operator = operatorInput?.value.trim() || 'security-operator';

    const res = await api.rejectTicket(id, operator, reason);
    if (res.ok) {
      await this.refreshApprovals();
    } else {
      alert(`Rejection failed: ${res.error || 'Unknown error'}`);
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
    const jsonpathVal = (document.getElementById('pg-jsonpath-input') as HTMLInputElement)?.value.trim() || undefined;
    const limitLinesStr = (document.getElementById('pg-limit-lines-input') as HTMLInputElement)?.value.trim() || undefined;
    const truncateBytesStr = (document.getElementById('pg-truncate-bytes-input') as HTMLInputElement)?.value.trim() || undefined;

    let parsedArgs: any = {};
    try {
      parsedArgs = JSON.parse(argsText);
    } catch {
      alert('Invalid arguments JSON object');
      return;
    }

    if (jsonpathVal) {
      parsedArgs['_jsonpath'] = jsonpathVal;
    }
    if (limitLinesStr && !isNaN(Number(limitLinesStr))) {
      parsedArgs['_limit_lines'] = Number(limitLinesStr);
    }
    if (truncateBytesStr && !isNaN(Number(truncateBytesStr))) {
      parsedArgs['_truncate_bytes'] = Number(truncateBytesStr);
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
      });

      store.setState({
        executionResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });

      store.addEventLog('POST', `/v1/tools/call → ${capId}`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);

      api.getConfig().then(cfgRes => {
        if (cfgRes.ok && cfgRes.circuit_breakers) {
          store.setState({ circuitBreakers: cfgRes.circuit_breakers });
        }
      });
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

  toggleBatchPlayground() {
    const argsInput = document.getElementById('pg-args-input') as HTMLTextAreaElement | null;
    if (!argsInput) return;

    const sampleBatch = [
      {
        "id": "step_1",
        "capability_id": "sqlite.read_query",
        "args": { "query": "SELECT * FROM users LIMIT 2" }
      },
      {
        "id": "step_2",
        "capability_id": "github.issues.search",
        "args": { "query": "label:bug" },
        "continue_on_error": true
      }
    ];

    argsInput.value = JSON.stringify(sampleBatch, null, 2);
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
    const titleEl = document.getElementById('modal-srv-title');
    const bannerEl = document.getElementById('modal-srv-template-banner');
    const nameInput = document.getElementById('modal-srv-name') as HTMLInputElement | null;
    const transportSelect = document.getElementById('modal-srv-transport') as HTMLSelectElement | null;
    const cmdInput = document.getElementById('modal-srv-command') as HTMLInputElement | null;
    const urlInput = document.getElementById('modal-srv-url') as HTMLInputElement | null;
    const ftInput = document.getElementById('modal-srv-ft') as HTMLInputElement | null;
    const cdInput = document.getElementById('modal-srv-cd') as HTMLInputElement | null;
    const autoRestartSelect = document.getElementById('modal-srv-autorestart') as HTMLSelectElement | null;
    const maxRestartsInput = document.getElementById('modal-srv-maxrestarts') as HTMLInputElement | null;

    if (titleEl) titleEl.textContent = 'Add Upstream MCP Server';
    if (bannerEl) bannerEl.style.display = 'flex';
    if (nameInput) {
      nameInput.value = '';
      nameInput.disabled = false;
    }
    if (transportSelect) transportSelect.value = 'stdio';
    if (cmdInput) cmdInput.value = '';
    if (urlInput) urlInput.value = '';
    const cmdGroup = document.getElementById('modal-group-cmd');
    const urlGroup = document.getElementById('modal-group-url');
    if (cmdGroup) cmdGroup.style.display = 'block';
    if (urlGroup) urlGroup.style.display = 'none';

    if (ftInput) ftInput.value = '3';
    if (cdInput) cdInput.value = '30000';
    if (autoRestartSelect) autoRestartSelect.value = 'true';
    if (maxRestartsInput) maxRestartsInput.value = '5';

    const modal = document.getElementById('modal-add-server');
    if (modal) modal.classList.add('active');
  }

  openEditServerModal(serverName: string) {
    this.closeModals();
    const state = store.getState();
    const serverCfg = state.config.mcpServers?.[serverName];
    if (!serverCfg) {
      alert(`Server '${serverName}' not found in configuration.`);
      return;
    }

    const titleEl = document.getElementById('modal-srv-title');
    const bannerEl = document.getElementById('modal-srv-template-banner');
    const nameInput = document.getElementById('modal-srv-name') as HTMLInputElement | null;
    const transportSelect = document.getElementById('modal-srv-transport') as HTMLSelectElement | null;
    const cmdInput = document.getElementById('modal-srv-command') as HTMLInputElement | null;
    const urlInput = document.getElementById('modal-srv-url') as HTMLInputElement | null;
    const ftInput = document.getElementById('modal-srv-ft') as HTMLInputElement | null;
    const cdInput = document.getElementById('modal-srv-cd') as HTMLInputElement | null;
    const autoRestartSelect = document.getElementById('modal-srv-autorestart') as HTMLSelectElement | null;
    const maxRestartsInput = document.getElementById('modal-srv-maxrestarts') as HTMLInputElement | null;

    if (titleEl) titleEl.textContent = `Edit Server '${serverName}'`;
    if (bannerEl) bannerEl.style.display = 'none';
    if (nameInput) {
      nameInput.value = serverName;
      nameInput.disabled = true; // Identifier remains immutable during edit
    }

    const isStdio = !!serverCfg.command;
    if (transportSelect) transportSelect.value = isStdio ? 'stdio' : 'http';

    const cmdGroup = document.getElementById('modal-group-cmd');
    const urlGroup = document.getElementById('modal-group-url');
    if (cmdGroup) cmdGroup.style.display = isStdio ? 'block' : 'none';
    if (urlGroup) urlGroup.style.display = isStdio ? 'none' : 'block';

    if (cmdInput) {
      cmdInput.value = isStdio ? `${serverCfg.command} ${(serverCfg.args || []).join(' ')}`.trim() : '';
    }
    if (urlInput) {
      urlInput.value = serverCfg.url || '';
    }

    const res = serverCfg.resilience || state.config.resilience;
    if (ftInput) ftInput.value = String(res?.failureThreshold ?? 3);
    if (cdInput) cdInput.value = String(res?.cooldownMs ?? 30000);
    if (autoRestartSelect) autoRestartSelect.value = res?.autoRestart === false ? 'false' : 'true';
    if (maxRestartsInput) maxRestartsInput.value = String(res?.maxRestarts ?? 5);

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

    const ftVal = (document.getElementById('modal-srv-ft') as HTMLInputElement)?.value.trim();
    const cdVal = (document.getElementById('modal-srv-cd') as HTMLInputElement)?.value.trim();
    const autoRestartVal = (document.getElementById('modal-srv-autorestart') as HTMLSelectElement)?.value;
    const maxRestartsVal = (document.getElementById('modal-srv-maxrestarts') as HTMLInputElement)?.value.trim();

    if (ftVal || cdVal || autoRestartVal || maxRestartsVal) {
      serverPayload.resilience = {
        failureThreshold: ftVal ? Number(ftVal) : 3,
        cooldownMs: cdVal ? Number(cdVal) : 30000,
        autoRestart: autoRestartVal !== 'false',
        maxRestarts: maxRestartsVal ? Number(maxRestartsVal) : 5
      };
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
        <details style="margin-top: 14px; background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px;">
          <summary style="font-size: 11.5px; font-weight: 600; color: var(--amber-400); cursor: pointer;">
            🛡️ Fault Tolerance &amp; Process Supervision (Optional)
          </summary>
          <div style="margin-top: 10px; display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
            <div>
              <label class="form-label" style="font-size: 10.5px;">Failure Threshold</label>
              <input type="number" class="form-input" id="cfg-srv-ft" placeholder="3" value="3">
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Cooldown (ms)</label>
              <input type="number" class="form-input" id="cfg-srv-cd" placeholder="30000" value="30000">
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Auto-Restart</label>
              <select class="form-input" id="cfg-srv-autorestart">
                <option value="true">Enabled (Default)</option>
                <option value="false">Disabled</option>
              </select>
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Max Restarts</label>
              <input type="number" class="form-input" id="cfg-srv-maxrestarts" placeholder="5" value="5">
            </div>
          </div>
        </details>
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

    const ftVal = (document.getElementById('cfg-srv-ft') as HTMLInputElement)?.value.trim();
    const cdVal = (document.getElementById('cfg-srv-cd') as HTMLInputElement)?.value.trim();
    const autoRestartVal = (document.getElementById('cfg-srv-autorestart') as HTMLSelectElement)?.value;
    const maxRestartsVal = (document.getElementById('cfg-srv-maxrestarts') as HTMLInputElement)?.value.trim();

    if (ftVal || cdVal || autoRestartVal || maxRestartsVal) {
      payload.resilience = {
        failureThreshold: ftVal ? Number(ftVal) : 3,
        cooldownMs: cdVal ? Number(cdVal) : 30000,
        autoRestart: autoRestartVal !== 'false',
        maxRestarts: maxRestartsVal ? Number(maxRestartsVal) : 5
      };
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

