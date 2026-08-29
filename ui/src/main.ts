import { store } from './state';
import { api } from './api';
import { renderOverview } from './components/overview';
import { renderServers } from './components/servers';
import { renderPlayground, generateSampleArgsFromSchema } from './components/playground';
import { renderTasks } from './components/tasks';
import { renderApprovals } from './components/approvals';
import { renderAudit } from './components/audit';
import { renderPolicy } from './components/policy';
import { renderAliases } from './components/aliases';
import { renderProfiles } from './components/profiles';
import { renderSecrets } from './components/secrets';
import { SERVER_TEMPLATES, ServerTemplate, findTemplateForServer } from './templates';

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
      const prof = state.activeProfile || undefined;
      const [configRes, capsRes, resRes, promptsRes, eventsRes, apprRes, tasksRes, auditEventsRes, auditStatsRes, clientsRes, secretsRes] = await Promise.all([
        api.getConfig(),
        api.listCapabilities(prof),
        api.listResources(prof),
        api.listPrompts(prof),
        api.getCatalogEvents(),
        api.listApprovals(),
        api.listTasks(),
        api.listAuditEvents({
          server_id: filters.serverId !== 'all' ? filters.serverId : undefined,
          event_type: filters.eventType !== 'all' ? filters.eventType : undefined,
          status: filters.status !== 'all' ? filters.status : undefined,
          search: filters.search.trim() ? filters.search.trim() : undefined,
          limit: filters.limit,
          offset: filters.offset,
        }),
        api.getAuditStats(),
        api.getClients().catch(() => ({ ok: false, clients: [] })),
        api.getSecrets().catch(() => ({ ok: false, secrets: [], keychain_service: 'warmplane' })),
      ]);

      if (clientsRes && clientsRes.ok && Array.isArray(clientsRes.clients)) {
        store.setState({ clients: clientsRes.clients });
      }

      if (secretsRes && secretsRes.ok && Array.isArray(secretsRes.secrets)) {
        store.setState({ secrets: secretsRes.secrets });
      }

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
        const currentSelectedCapId = store.getState().selectedCapabilityId;
        const capExists = capsRes.capabilities.some(c => c.id === currentSelectedCapId);
        const newSelectedCapId = capExists ? currentSelectedCapId : (capsRes.capabilities.length > 0 ? capsRes.capabilities[0].id : null);

        store.setState({
          capabilities: capsRes.capabilities,
          capabilitiesHiddenByPolicy: capsRes.hidden_by_policy || 0,
          selectedCapabilityId: newSelectedCapId,
        });
      }

      if (resRes && Array.isArray(resRes.resources)) {
        const currentSelectedId = store.getState().selectedResourceId;
        const resExists = resRes.resources.some(r => (r.uri === currentSelectedId || r.id === currentSelectedId));
        const newSelectedId = resExists ? currentSelectedId : (resRes.resources.length > 0 ? (resRes.resources[0].uri || resRes.resources[0].id || null) : null);

        store.setState({
          resources: resRes.resources,
          resourcesHiddenByPolicy: resRes.hidden_by_policy || 0,
          selectedResourceId: newSelectedId,
        });
      }

      if (promptsRes && Array.isArray(promptsRes.prompts)) {
        const currentSelectedId = store.getState().selectedPromptId;
        const promptExists = promptsRes.prompts.some(p => (p.name === currentSelectedId || p.id === currentSelectedId));
        const newSelectedId = promptExists ? currentSelectedId : (promptsRes.prompts.length > 0 ? (promptsRes.prompts[0].name || promptsRes.prompts[0].id || null) : null);

        store.setState({
          prompts: promptsRes.prompts,
          promptsHiddenByPolicy: promptsRes.hidden_by_policy || 0,
          selectedPromptId: newSelectedId,
        });
      }

      if (eventsRes && Array.isArray(eventsRes.events)) {
        store.setState({
          catalogEvents: eventsRes.events
        });
      }

      if (apprRes && Array.isArray(apprRes.approvals)) {
        store.setState({
          approvals: apprRes.approvals
        });
      }

      if (tasksRes && Array.isArray(tasksRes.tasks)) {
        store.setState({
          tasks: tasksRes.tasks
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

  switchTab(tab: 'overview' | 'servers' | 'playground' | 'tasks' | 'approvals' | 'audit' | 'policy' | 'secrets' | 'aliases' | 'profiles') {
    store.setState({ activeTab: tab });
    this.refreshData();
  }

  render() {
    const state = store.getState();
    const mainEl = document.getElementById('app-main');
    if (!mainEl) return;

    // Update nav item active states and badges
    const inputReqTasks = (state.tasks || []).filter(t => t.status === 'input_required').length;
    const legacyPendingApprs = (state.approvals || []).filter(a => a.status === 'pending').length;
    const totalActionNeeded = Math.max(inputReqTasks, legacyPendingApprs);

    const badgeEl = document.getElementById('nav-approvals-badge');
    if (badgeEl) {
      badgeEl.textContent = totalActionNeeded > 0 ? `${totalActionNeeded}` : '';
      (badgeEl as HTMLElement).style.display = totalActionNeeded > 0 ? 'inline-block' : 'none';
    }

    document.querySelectorAll('.nav-item').forEach(el => {
      const tabAttr = el.getAttribute('data-tab');
      if (tabAttr === state.activeTab || (state.activeTab === 'tasks' && tabAttr === 'approvals') || (state.activeTab === 'approvals' && tabAttr === 'tasks')) {
        el.classList.add('active');
      } else {
        el.classList.remove('active');
      }
    });

    // Update top title
    const titleEl = document.getElementById('top-title');
    const titles: Record<string, string> = {
      overview: 'Overview Cockpit',
      servers: 'Server Hub & Connections',
      playground: 'MCP Capability Playground',
      tasks: 'SEP-2663 Tasks & HITL Review',
      approvals: 'SEP-2663 Tasks & HITL Review',
      audit: 'WORM Audit & Compliance Ledger',
      policy: 'Security Governance & Redaction',
      secrets: 'Native OS Keychain & Secrets Vault',
      aliases: 'Facade & Alias Studio',
      profiles: 'Server Constellation Profiles'
    };
    if (titleEl) titleEl.textContent = titles[state.activeTab] || 'Control Deck';

    // Update top bar profile selector
    this.renderTopProfileSelector();

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
      case 'tasks':
      case 'approvals':
        mainEl.innerHTML = renderTasks(state);
        break;
      case 'audit':
        mainEl.innerHTML = renderAudit();
        break;
      case 'policy':
        mainEl.innerHTML = renderPolicy();
        break;
      case 'secrets':
        mainEl.innerHTML = renderSecrets();
        break;
      case 'aliases':
        mainEl.innerHTML = renderAliases();
        break;
      case 'profiles':
        mainEl.innerHTML = renderProfiles();
        break;
    }
  }

  toggleClientsCollapse() {
    const current = store.getState().clientsCollapsed;
    store.setState({ clientsCollapsed: !current });
    this.render();
  }

  async saveNewVaultSecret() {
    const keyEl = document.getElementById('vault-new-key') as HTMLInputElement;
    const valEl = document.getElementById('vault-new-val') as HTMLInputElement;
    const srvEl = document.getElementById('vault-new-service') as HTMLInputElement;

    const key = keyEl?.value.trim();
    const val = valEl?.value.trim();
    const service = srvEl?.value.trim() || 'warmplane';

    if (!key || !val) {
      alert('Key and secret value are required');
      return;
    }

    try {
      const res = await api.saveSecret(key, val, service);
      if (res.ok) {
        alert(`Secret '${key}' saved securely into OS Keychain!\nReference: ${res.uri}`);
        if (keyEl) keyEl.value = '';
        if (valEl) valEl.value = '';
        await this.refreshData();
      } else {
        alert(`Failed to save secret: ${res.error}`);
      }
    } catch (e: any) {
      alert(`Error saving secret: ${e.message}`);
    }
  }

  async deleteVaultSecret(key: string, serverName?: string) {
    const confirmMsg = serverName 
      ? `Are you sure you want to remove secret '${key}' from OS Keychain and server '${serverName}'?`
      : `Are you sure you want to remove secret '${key}' from OS Keychain?`;
    if (!confirm(confirmMsg)) return;

    try {
      const res = await api.deleteSecret(key);
      if (res.ok || res.error?.includes('not found')) {
        // If associated with a server, remove from server env
        if (serverName) {
          const state = store.getState();
          const servers = state.config.mcpServers || {};
          const srv = servers[serverName];
          if (srv && srv.env && srv.env[key]) {
            const updatedEnv = { ...srv.env };
            delete updatedEnv[key];
            const updatedSrv = { ...srv, env: updatedEnv };
            await api.upsertServer(serverName, updatedSrv);
          }
        }
        await this.refreshData();
      } else {
        alert(`Failed to delete secret: ${res.error}`);
      }
    } catch (e: any) {
      alert(`Error deleting secret: ${e.message}`);
    }
  }

  async removeSecretFromConfig(serverName: string, envKey: string) {
    if (!confirm(`Remove environment variable '${envKey}' from '${serverName}' configuration?`)) return;
    try {
      const state = store.getState();
      const servers = state.config.mcpServers || {};
      const srv = servers[serverName];
      if (srv && srv.env && srv.env[envKey]) {
        const updatedEnv = { ...srv.env };
        delete updatedEnv[envKey];
        const updatedSrv = { ...srv, env: updatedEnv };
        const upsertRes = await api.upsertServer(serverName, updatedSrv);
        if (upsertRes.ok) {
          await this.refreshData();
        } else {
          alert(`Failed to update server config: ${upsertRes.error}`);
        }
      }
    } catch (e: any) {
      alert(`Error removing secret from config: ${e.message}`);
    }
  }

  async quickVaultEnv(serverName: string, envKey: string) {
    const secretVal = prompt(`Enter secret value to store in OS Keychain for ${serverName}.${envKey}:`);
    if (!secretVal) return;

    try {
      const saveRes = await api.saveSecret(envKey, secretVal, 'warmplane');
      if (!saveRes.ok) {
        alert(`Failed to save to Keychain: ${saveRes.error}`);
        return;
      }

      // Update server config to use keychain URI
      const state = store.getState();
      const servers = state.config.mcpServers || {};
      const srv = servers[serverName];
      if (srv) {
        const updatedEnv = { ...(srv.env || {}), [envKey]: `keychain://warmplane/${envKey}` };
        const updatedSrv = { ...srv, env: updatedEnv };
        const upsertRes = await api.upsertServer(serverName, updatedSrv);
        if (upsertRes.ok) {
          await this.refreshData();
          alert(`Successfully configured ${serverName}.${envKey} in OS Keychain!`);
        } else {
          alert(`Failed to update server config: ${upsertRes.error}`);
        }
      }
    } catch (e: any) {
      alert(`Error during migration: ${e.message}`);
    }
  }

  // SEP-2663 Tasks & HITL Actions
  async refreshTasks() {
    try {
      const tasksRes = await api.listTasks();
      if (tasksRes && Array.isArray(tasksRes.tasks)) {
        store.setState({ tasks: tasksRes.tasks });
      }
    } catch (e) {
      console.error('Failed to refresh tasks:', e);
    }
  }

  filterTasksByStatus(status: string) {
    store.setState({ taskFilterStatus: status });
  }

  togglePlaygroundAsyncTask(enabled: boolean) {
    store.setState({ playgroundAsyncTask: enabled });
  }

  async submitTaskInputResponses(taskId: string) {
    const task = store.getState().tasks.find(t => t.taskId === taskId);
    const inputReqs = task?.inputRequests || {};
    const inputKeys = Object.keys(inputReqs);
    const responses: Record<string, any> = {};

    if (inputKeys.length > 0) {
      for (const k of inputKeys) {
        const reqDef = inputReqs[k];
        if (reqDef && reqDef.type === 'approval_review') {
          const decisionEl = document.getElementById(`task-input-${taskId}-${k}-decision`) as HTMLSelectElement | null;
          const argsEl = document.getElementById(`task-input-${taskId}-${k}`) as HTMLTextAreaElement | null;
          const isApproved = decisionEl ? decisionEl.value === 'true' : true;
          let modArgs: any = undefined;
          if (argsEl && argsEl.value.trim()) {
            try {
              modArgs = JSON.parse(argsEl.value.trim());
            } catch {
              alert('Invalid JSON in parameters editor');
              return;
            }
          }
          responses[k] = {
            approved: isApproved,
            modified_args: modArgs,
            reason: isApproved ? undefined : 'Operator rejected execution via Tasks review',
          };
        } else {
          const el = document.getElementById(`task-input-${taskId}-${k}`) as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement | null;
          if (el) {
            const val = el.value.trim();
            try {
              responses[k] = JSON.parse(val);
            } catch {
              responses[k] = val;
            }
          }
        }
      }
    } else {
      const rawEl = document.getElementById(`task-raw-input-${taskId}`) as HTMLTextAreaElement | null;
      if (rawEl && rawEl.value.trim()) {
        try {
          Object.assign(responses, JSON.parse(rawEl.value.trim()));
        } catch {
          alert('Invalid JSON in raw input responses');
          return;
        }
      }
    }

    try {
      const res = await api.updateTask(taskId, responses);
      if (res.ok) {
        await this.refreshTasks();
      } else {
        alert(`Task update failed: ${res.error?.message || res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error updating task: ${e.message}`);
    }
  }

  async promptCancelTask(taskId: string) {
    const reason = prompt('Reason for cancelling task:');
    if (reason === null) return;

    try {
      const res = await api.cancelTask(taskId, reason || undefined);
      if (res.ok) {
        await this.refreshTasks();
      } else {
        alert(`Task cancellation failed: ${res.error?.message || res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error cancelling task: ${e.message}`);
    }
  }

  async openTaskInspectorModal(taskId: string) {
    this.closeModals();
    const modal = document.getElementById('modal-task-inspector');
    if (!modal) return;

    const titleEl = document.getElementById('modal-task-title');
    const bodyEl = document.getElementById('modal-task-body');
    const footerEl = document.getElementById('modal-task-footer');

    if (titleEl) titleEl.textContent = `Task Inspector: ${taskId}`;
    if (bodyEl) {
      bodyEl.innerHTML = `
        <div style="padding: 30px; text-align: center; color: var(--text-dim); font-family: var(--ff-mono); font-size: 12px;">
          ⏳ Fetching task execution record...
        </div>
      `;
    }
    modal.classList.add('active');

    try {
      const res = await api.getTask(taskId);
      if (!res.ok || !res.task) {
        if (bodyEl) {
          bodyEl.innerHTML = `
            <div style="background: rgba(248, 113, 113, 0.12); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 16px; color: var(--red-400);">
              <div style="font-weight: 700; margin-bottom: 6px;">Failed to load task snapshot</div>
              <div style="font-family: var(--ff-mono); font-size: 11.5px;">${escapeHtml(res.error?.message || 'Task not found in runtime registry')}</div>
            </div>
          `;
        }
        return;
      }

      const t = res.task;
      const progressPercent = t.progress !== undefined ? Math.round(t.progress * 100) : (t.status === 'completed' ? 100 : t.status === 'working' ? 50 : 0);
      const now = Math.floor(Date.now() / 1000);
      const ttlLeft = t.expiresAtEpochSecs ? Math.max(0, t.expiresAtEpochSecs - now) : (t.ttlSeconds || 300);
      const isEnded = t.status === 'completed' || t.status === 'cancelled' || t.status === 'failed';

      const statusColor = 
        t.status === 'completed' ? 'var(--green-400)' :
        t.status === 'working' ? 'var(--cyan-400)' :
        t.status === 'input_required' ? 'var(--amber-300)' :
        t.status === 'cancelled' ? 'var(--text-muted)' : 'var(--red-400)';

      const statusBg = 
        t.status === 'completed' ? 'rgba(52, 211, 153, 0.15)' :
        t.status === 'working' ? 'rgba(56, 189, 248, 0.15)' :
        t.status === 'input_required' ? 'rgba(245, 158, 11, 0.18)' :
        t.status === 'cancelled' ? 'rgba(148, 163, 184, 0.15)' : 'rgba(248, 113, 113, 0.15)';

      const hasError = !!t.error;
      const hasResult = t.result !== undefined && t.result !== null;
      const hasInputReqs = t.inputRequests && Object.keys(t.inputRequests).length > 0;

      if (bodyEl) {
        bodyEl.innerHTML = `
          <!-- Status Banner & Top Metrics -->
          <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px 16px; margin-bottom: 14px;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="brand-badge" style="background: ${statusBg}; color: ${statusColor}; border-color: ${statusColor}; font-size: 11px;">
                  ${escapeHtml(t.status.toUpperCase())}
                </span>
                <span style="font-family: var(--ff-mono); font-size: 13px; font-weight: 700; color: var(--text-main);">
                  ${escapeHtml(t.capabilityId || 'Tool Execution')}
                </span>
                ${t.serverId ? `<span style="font-size: 11px; color: var(--text-dim);">via <code style="color: var(--cyan-400);">${escapeHtml(t.serverId)}</code></span>` : ''}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                Task ID: <span style="color: var(--text-muted);">${escapeHtml(t.taskId)}</span>
              </div>
            </div>
            <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px;">
              <div style="color: var(--text-dim);">Created: <span style="color: var(--text-muted);">${t.createdAtEpochSecs ? new Date(t.createdAtEpochSecs * 1000).toLocaleString() : (t.createdAt ? new Date(t.createdAt).toLocaleString() : '—')}</span></div>
              ${!isEnded ? `<div style="color: var(--amber-400); margin-top: 2px;">TTL: ${ttlLeft}s remaining</div>` : ''}
            </div>
          </div>

          <!-- Progress Bar -->
          <div style="margin-bottom: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; font-family: var(--ff-mono); font-size: 11px;">
              <span style="color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Execution Progress</span>
              <span style="color: ${statusColor}; font-weight: 700;">${progressPercent}% ${t.total ? `(step ${Math.round((t.progress || 0) * t.total)} of ${t.total})` : ''}</span>
            </div>
            <div style="height: 8px; background: var(--surface-card); border-radius: 4px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${progressPercent}%; background: ${statusColor}; transition: width 0.3s;"></div>
            </div>
          </div>

          <!-- Caller Context Envelope -->
          ${t.context ? `
            <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 14px; color: var(--text-muted);">
              ${t.context.actor_id ? `<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${escapeHtml(t.context.actor_id)}</span></div>` : ''}
              ${t.context.operation_id ? `<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${escapeHtml(t.context.operation_id)}</span></div>` : ''}
              ${t.context.grant_id ? `<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${escapeHtml(t.context.grant_id)}</span></div>` : ''}
            </div>
          ` : ''}

          <!-- Diagnostic / Error Message -->
          ${hasError ? `
            <div style="background: rgba(248, 113, 113, 0.1); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--red-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚠️ Failure / Cancellation Trace
              </div>
              <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--red-300); white-space: pre-wrap; word-break: break-word; margin: 0;">${escapeHtml(typeof t.error === 'string' ? t.error : JSON.stringify(t.error, null, 2))}</pre>
            </div>
          ` : ''}

          <!-- Input Requests (if awaiting input) -->
          ${hasInputReqs ? `
            <div style="margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚡ Pending Input Requests (MRTR / HITL)
              </div>
              <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); white-space: pre-wrap; word-break: break-word; max-height: 180px; overflow-y: auto; margin: 0;">${escapeHtml(JSON.stringify(t.inputRequests, null, 2))}</pre>
            </div>
          ` : ''}

          <!-- Result Payload -->
          <div style="margin-bottom: 8px;">
            <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">
              ${hasResult ? '📦 Output Result Payload' : 'State Details'}
            </div>
            <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-main); white-space: pre-wrap; word-break: break-word; max-height: 220px; overflow-y: auto; margin: 0;">${escapeHtml(hasResult ? JSON.stringify(t.result, null, 2) : (t.status === 'working' ? 'Task execution is currently in-flight in background worker pool.' : 'No output payload recorded.'))}</pre>
          </div>
        `;
      }

      if (footerEl) {
        footerEl.innerHTML = `
          <div>
            ${!isEnded ? `
              <button class="btn btn-danger" style="font-size: 11.5px;" onclick="window.app.promptCancelTask('${escapeHtml(t.taskId)}'); window.app.closeModals();">
                ⛔ Cancel Task
              </button>
            ` : `
              <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">Task is terminal (${escapeHtml(t.status)})</span>
            `}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="navigator.clipboard.writeText('${escapeHtml(t.taskId)}'); alert('Task ID copied to clipboard');">
              📋 Copy Task ID
            </button>
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.closeModals()">
              Close
            </button>
          </div>
        `;
      }
    } catch (e: any) {
      if (bodyEl) {
        bodyEl.innerHTML = `
          <div style="color: var(--red-400); font-family: var(--ff-mono); font-size: 11.5px;">
            Failed to inspect task: ${escapeHtml(e.message)}
          </div>
        `;
      }
    }
  }

  // HITL Approval Actions (Legacy compatibility)
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
      await this.refreshTasks();
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
      await this.refreshTasks();
    } else {
      alert(`Rejection failed: ${res.error || 'Unknown error'}`);
    }
  }

  // Action Handlers
  setPlaygroundMode(mode: 'tools' | 'resources' | 'prompts') {
    store.setState({ playgroundMode: mode });
  }

  selectCapability(id: string) {
    store.setState({ selectedCapabilityId: id });
    const cap = store.getState().capabilities.find(c => c.id === id);
    const textarea = document.getElementById('pg-args-input') as HTMLTextAreaElement | null;
    if (cap) {
      const sample = generateSampleArgsFromSchema(cap.input_schema, false);
      const jsonStr = JSON.stringify(sample, null, 2);
      if (textarea) textarea.value = jsonStr;
      const pgArgs = { ...(store.getState().playgroundArgs || {}) };
      pgArgs[id] = jsonStr;
      store.getState().playgroundArgs = pgArgs;
    }
  }

  selectResource(id: string) {
    store.setState({ selectedResourceId: id });
  }

  selectPrompt(id: string) {
    store.setState({ selectedPromptId: id });
  }

  filterResources(query: string) {
    const q = query.toLowerCase().trim();
    const allRes = store.getState().resources || [];
    const filtered = allRes.filter(r =>
      r.id.toLowerCase().includes(q) ||
      (r.name && r.name.toLowerCase().includes(q)) ||
      (r.uri && r.uri.toLowerCase().includes(q)) ||
      (r.server && r.server.toLowerCase().includes(q))
    );
    const listEl = document.getElementById('pg-res-list');
    if (listEl) {
      if (filtered.length === 0) {
        listEl.innerHTML = `
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${escapeHtml(query)}"
          </div>
        `;
      } else {
        listEl.innerHTML = filtered.map(r => {
          const active = r.id === store.getState().selectedResourceId ? 'active' : '';
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
    }
  }

  filterPrompts(query: string) {
    const q = query.toLowerCase().trim();
    const allPrompts = store.getState().prompts || [];
    const filtered = allPrompts.filter(p =>
      p.id.toLowerCase().includes(q) ||
      (p.name && p.name.toLowerCase().includes(q)) ||
      (p.description && p.description.toLowerCase().includes(q)) ||
      (p.server && p.server.toLowerCase().includes(q))
    );
    const listEl = document.getElementById('pg-prompt-list');
    if (listEl) {
      if (filtered.length === 0) {
        listEl.innerHTML = `
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${escapeHtml(query)}"
          </div>
        `;
      } else {
        listEl.innerHTML = filtered.map(p => {
          const active = p.id === store.getState().selectedPromptId ? 'active' : '';
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
    }
  }

  updatePlaygroundArgs(val: string) {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    if (!capId) return;
    const pgArgs = { ...(state.playgroundArgs || {}) };
    pgArgs[capId] = val;
    // Update store state directly without triggering full re-render on each keystroke
    state.playgroundArgs = pgArgs;
  }

  fillPlaygroundSampleArgs(onlyRequired: boolean = false) {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    const cap = state.capabilities.find(c => c.id === capId);
    const textarea = document.getElementById('pg-args-input') as HTMLTextAreaElement | null;
    if (!textarea) return;

    if (!cap || !cap.input_schema) {
      textarea.value = '{}';
      if (capId) {
        const pgArgs = { ...(state.playgroundArgs || {}) };
        pgArgs[capId] = '{}';
        state.playgroundArgs = pgArgs;
      }
      return;
    }

    const sample = generateSampleArgsFromSchema(cap.input_schema, onlyRequired);
    const jsonStr = JSON.stringify(sample, null, 2);
    textarea.value = jsonStr;
    if (capId) {
      const pgArgs = { ...(state.playgroundArgs || {}) };
      pgArgs[capId] = jsonStr;
      state.playgroundArgs = pgArgs;
    }
  }

  formatPlaygroundArgs() {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    const textarea = document.getElementById('pg-args-input') as HTMLTextAreaElement | null;
    if (textarea) {
      try {
        const parsed = JSON.parse(textarea.value || '{}');
        const formatted = JSON.stringify(parsed, null, 2);
        textarea.value = formatted;
        if (capId) {
          const pgArgs = { ...(state.playgroundArgs || {}) };
          pgArgs[capId] = formatted;
          state.playgroundArgs = pgArgs;
        }
      } catch (e: any) {
        alert(`Cannot format JSON: ${e.message}`);
      }
    }
  }

  insertPlaygroundArgKey(key: string, type: string, defaultVal: any) {
    const state = store.getState();
    const capId = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : null);
    const textarea = document.getElementById('pg-args-input') as HTMLTextAreaElement | null;
    if (textarea) {
      let currentObj: Record<string, any> = {};
      try {
        currentObj = JSON.parse(textarea.value || '{}');
      } catch {
        currentObj = {};
      }

      if (currentObj[key] === undefined) {
        if (defaultVal !== null && defaultVal !== undefined) {
          currentObj[key] = defaultVal;
        } else {
          switch (type) {
            case 'string':
              currentObj[key] = `sample_${key}`;
              break;
            case 'number':
            case 'integer':
              currentObj[key] = 0;
              break;
            case 'boolean':
              currentObj[key] = true;
              break;
            case 'array':
              currentObj[key] = [];
              break;
            case 'object':
              currentObj[key] = {};
              break;
            default:
              currentObj[key] = `sample_${key}`;
          }
        }
      }
      const newVal = JSON.stringify(currentObj, null, 2);
      textarea.value = newVal;
      if (capId) {
        const pgArgs = { ...(state.playgroundArgs || {}) };
        pgArgs[capId] = newVal;
        state.playgroundArgs = pgArgs;
      }
    }
  }

  fillBatchStepSampleArgs(idx: number) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    const step = steps[idx];
    if (!step || !step.capability_id) return;
    const cap = state.capabilities.find(c => c.id === step.capability_id);
    if (!cap || !cap.input_schema) return;

    const props = cap.input_schema.properties || {};
    const sample: Record<string, any> = {};
    for (const [key, propDef] of Object.entries<any>(props)) {
      if (propDef.default !== undefined) {
        sample[key] = propDef.default;
      } else if (Array.isArray(propDef.enum) && propDef.enum.length > 0) {
        sample[key] = propDef.enum[0];
      } else {
        const type = propDef.type || 'string';
        switch (type) {
          case 'string':
            sample[key] = `sample_${key}`;
            break;
          case 'number':
          case 'integer':
            sample[key] = 0;
            break;
          case 'boolean':
            sample[key] = true;
            break;
          case 'array':
            sample[key] = [];
            break;
          case 'object':
            sample[key] = {};
            break;
          default:
            sample[key] = `sample_${key}`;
        }
      }
    }
    const sampleStr = JSON.stringify(sample, null, 2);
    steps[idx] = { ...steps[idx], argsJson: sampleStr };
    store.setState({ batchSteps: steps });
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

    const opReqId = `op-${Date.now()}`;
    store.setState({ isExecuting: true, activeRequestId: opReqId });

    const prof = state.activeProfile || undefined;
    const isAsync = state.playgroundAsyncTask || false;

    try {
      const res = await api.callCapability({
        capability_id: capId,
        args: parsedArgs,
        request_id: opReqId,
        async_task: isAsync ? true : undefined,
        context: {
          operation_id: contextVal || opReqId,
        },
      }, prof);

      store.setState({
        isExecuting: false,
        activeRequestId: null,
        executionResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });

      if (res.status === 202 || res.data?.resultType === 'task') {
        this.refreshTasks();
      }

      store.addEventLog('POST', `/v1/tools/call → ${capId}`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);

      api.getConfig().then(cfgRes => {
        if (cfgRes.ok && cfgRes.circuit_breakers) {
          store.setState({ circuitBreakers: cfgRes.circuit_breakers });
        }
      });
    } catch (e: any) {
      store.setState({
        isExecuting: false,
        activeRequestId: null,
        executionResult: {
          status: 500,
          durationMs: 0,
          data: { error: e.toString() }
        }
      });
    }
  }

  async cancelActiveOperation() {
    const state = store.getState();
    const reqId = state.activeRequestId;
    if (reqId) {
      try {
        await api.cancelOperation(reqId);
      } catch (e) {
        console.warn('Failed to send cancel signal:', e);
      }
    }
    store.setState({
      isExecuting: false,
      activeRequestId: null,
      executionResult: {
        status: 499,
        durationMs: 0,
        data: {
          ok: false,
          error: {
            code: 'CANCELLED',
            message: 'Operation cancelled by operator'
          }
        }
      }
    });
  }

  openBatchModal() {
    const state = store.getState();
    let steps = state.batchSteps;
    // If no steps exist or first step is unconfigured, pre-fill with selected capability
    if (!steps || steps.length === 0) {
      const defaultCap = state.selectedCapabilityId || (state.capabilities[0] ? state.capabilities[0].id : '');
      steps = [
        { id: 'step_1', capability_id: defaultCap, argsJson: '{}', continue_on_error: false },
        { id: 'step_2', capability_id: '', argsJson: '{}', continue_on_error: true }
      ];
      store.setState({ batchSteps: steps });
    }
    store.setState({ isBatchModalOpen: true });
  }

  closeBatchModal() {
    store.setState({ isBatchModalOpen: false });
  }

  addBatchStep() {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    const newIdx = steps.length + 1;
    steps.push({
      id: `step_${newIdx}`,
      capability_id: '',
      argsJson: '{}',
      continue_on_error: false
    });
    store.setState({ batchSteps: steps });
  }

  removeBatchStep(idx: number) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    if (steps.length <= 1) {
      alert('Pipeline must contain at least one execution step.');
      return;
    }
    steps.splice(idx, 1);
    // Re-index step IDs
    const reindexed = steps.map((s, i) => ({
      ...s,
      id: `step_${i + 1}`
    }));
    store.setState({ batchSteps: reindexed });
  }

  updateBatchStepCapability(idx: number, capId: string) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    if (steps[idx]) {
      steps[idx] = { ...steps[idx], capability_id: capId };
      store.setState({ batchSteps: steps });
    }
  }

  updateBatchStepContinueOnError(idx: number, continueOnError: boolean) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    if (steps[idx]) {
      steps[idx] = { ...steps[idx], continue_on_error: continueOnError };
      store.setState({ batchSteps: steps });
    }
  }

  updateBatchStepArgs(idx: number, argsJson: string) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    if (steps[idx]) {
      steps[idx] = { ...steps[idx], argsJson };
      // Update in-place without triggering full re-render on each keystroke
      state.batchSteps[idx].argsJson = argsJson;
    }
  }

  appendBatchVariable(idx: number, varStr: string) {
    const state = store.getState();
    const steps = [...(state.batchSteps || [])];
    const textarea = document.getElementById(`batch-step-args-${idx}`) as HTMLTextAreaElement | null;
    if (textarea) {
      const currentVal = textarea.value;
      const start = textarea.selectionStart || currentVal.length;
      const end = textarea.selectionEnd || currentVal.length;
      const newVal = currentVal.substring(0, start) + varStr + currentVal.substring(end);
      textarea.value = newVal;
      if (steps[idx]) {
        steps[idx] = { ...steps[idx], argsJson: newVal };
        store.setState({ batchSteps: steps });
      }
    }
  }

  async executeBatchPipeline() {
    const state = store.getState();
    const steps = state.batchSteps || [];
    const formattedSteps: Array<{ id: string; capability_id: string; args: Record<string, any>; continue_on_error?: boolean }> = [];

    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      if (!step.capability_id) {
        alert(`Please select a capability for Step ${i + 1}`);
        return;
      }
      let args = {};
      try {
        args = JSON.parse(step.argsJson || '{}');
      } catch {
        alert(`Invalid JSON in Step ${i + 1} arguments`);
        return;
      }
      formattedSteps.push({
        id: step.id || `step_${i + 1}`,
        capability_id: step.capability_id,
        args,
        continue_on_error: step.continue_on_error
      });
    }

    store.setState({ isBatchModalOpen: false });

    const prof = state.activeProfile || undefined;
    try {
      const res = await api.batchCallCapabilities(formattedSteps, prof);
      store.setState({
        executionResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });
      store.addEventLog('POST', `/v1/tools/batch_call (${steps.length} steps)`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);
    } catch (e: any) {
      store.setState({
        executionResult: {
          status: 500,
          durationMs: 0,
          data: { error: e.toString() }
        }
      });
    }
  }

  async executeReadResource() {
    const state = store.getState();
    const resId = state.selectedResourceId || (state.resources[0] ? state.resources[0].id : null);
    if (!resId) return;

    const jsonpath = (document.getElementById('pg-res-jsonpath-input') as HTMLInputElement)?.value.trim() || undefined;
    const lines = (document.getElementById('pg-res-lines-input') as HTMLInputElement)?.value.trim() || undefined;
    const bytes = (document.getElementById('pg-res-bytes-input') as HTMLInputElement)?.value.trim() || undefined;

    const reqPayload: any = { resource_id: resId };
    if (jsonpath) reqPayload['_jsonpath'] = jsonpath;
    if (lines && !isNaN(Number(lines))) reqPayload['_limit_lines'] = Number(lines);
    if (bytes && !isNaN(Number(bytes))) reqPayload['_truncate_bytes'] = Number(bytes);

    const prof = state.activeProfile || undefined;
    try {
      const res = await api.readResource({
        resource_id: resId,
        input_responses: reqPayload
      }, prof);
      store.setState({
        resourceReadResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });
      store.addEventLog('POST', `/v1/resources/read → ${resId}`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);
    } catch (e: any) {
      store.setState({
        resourceReadResult: {
          status: 500,
          durationMs: 0,
          data: { error: e.toString() }
        }
      });
    }
  }

  async executeGetPrompt() {
    const state = store.getState();
    const promptId = state.selectedPromptId || (state.prompts[0] ? state.prompts[0].id : null);
    if (!promptId) return;

    const argInputs = document.querySelectorAll('.prompt-arg-input');
    const args: Record<string, any> = {};
    argInputs.forEach((el) => {
      const input = el as HTMLInputElement;
      const key = input.getAttribute('data-arg-name');
      if (key && input.value.trim()) {
        args[key] = input.value.trim();
      }
    });

    const prof = state.activeProfile || undefined;
    try {
      const res = await api.getPrompt({
        prompt_id: promptId,
        arguments: args
      }, prof);
      store.setState({
        promptGetResult: {
          status: res.status,
          durationMs: res.durationMs,
          data: res.data
        }
      });
      store.addEventLog('POST', `/v1/prompts/get → ${promptId}`, res.status === 200 ? '200 OK' : `HTTP ${res.status}`, `${res.durationMs.toFixed(1)}ms`);
    } catch (e: any) {
      store.setState({
        promptGetResult: {
          status: 500,
          durationMs: 0,
          data: { error: e.toString() }
        }
      });
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
  async submitPolicyRule(type: 'allow' | 'deny' | 'redact' | 'requireApproval') {
    const inputId = type === 'allow' ? 'policy-new-allow' : type === 'deny' ? 'policy-new-deny' : type === 'redact' ? 'policy-new-redact' : 'policy-new-requireApproval';
    const inputEl = document.getElementById(inputId) as HTMLInputElement | null;
    if (!inputEl) return;
    const val = inputEl.value.trim();
    if (!val) return;
    await this.addPolicyRule(type, val);
    inputEl.value = '';
  }

  async addPolicyRule(type: 'allow' | 'deny' | 'redact' | 'requireApproval', val: string) {
    const trimmed = (val || '').trim();
    if (!trimmed) return;
    const state = store.getState();
    const activeProfName = state.activeProfile;
    const activeProf = activeProfName ? state.config.profiles?.[activeProfName] : undefined;

    if (activeProf && activeProfName) {
      // Add rule to active profile's policy
      const current = activeProf.policy || {};
      const allow = [...(current.allow || [])];
      const deny = [...(current.deny || [])];
      const redact = [...(current.redact_keys || current.redactKeys || [])];
      const requireApproval = [...(current.require_approval || current.requireApproval || [])];

      if (type === 'allow' && !allow.includes(trimmed)) allow.push(trimmed);
      if (type === 'deny' && !deny.includes(trimmed)) deny.push(trimmed);
      if (type === 'redact' && !redact.includes(trimmed)) redact.push(trimmed);
      if (type === 'requireApproval' && !requireApproval.includes(trimmed)) requireApproval.push(trimmed);

      const updatedPolicy = {
        ...current,
        allow,
        deny,
        redactKeys: redact,
        requireApproval,
      };

      const res = await api.upsertProfile(activeProfName, activeProf.servers, activeProf.description, updatedPolicy);
      if (!res.ok) {
        alert(`Failed to save profile policy rule: ${res.error || 'Unknown error'}`);
      }
    } else {
      // Add rule to global daemon policy
      const current = state.config.policy || {};
      const allow = [...(current.allow || [])];
      const deny = [...(current.deny || [])];
      const redact = [...(current.redact_keys || current.redactKeys || [])];
      const requireApproval = [...(current.require_approval || current.requireApproval || [])];

      if (type === 'allow' && !allow.includes(trimmed)) allow.push(trimmed);
      if (type === 'deny' && !deny.includes(trimmed)) deny.push(trimmed);
      if (type === 'redact' && !redact.includes(trimmed)) redact.push(trimmed);
      if (type === 'requireApproval' && !requireApproval.includes(trimmed)) requireApproval.push(trimmed);

      const res = await api.savePolicy({
        ...current,
        allow,
        deny,
        redact_keys: redact,
        redactKeys: redact,
        require_approval: requireApproval,
        requireApproval,
      });
      if (!res.ok) {
        alert(`Failed to save policy rule: ${res.error || 'Unknown error'}`);
      }
    }
    await this.refreshData();
  }

  async removePolicyRule(type: 'allow' | 'deny' | 'redact' | 'requireApproval', index: number) {
    const state = store.getState();
    const activeProfName = state.activeProfile;
    const activeProf = activeProfName ? state.config.profiles?.[activeProfName] : undefined;

    if (activeProf && activeProfName) {
      // Remove rule from active profile's policy
      const current = activeProf.policy || {};
      const allow = [...(current.allow || [])];
      const deny = [...(current.deny || [])];
      const redact = [...(current.redact_keys || current.redactKeys || [])];
      const requireApproval = [...(current.require_approval || current.requireApproval || [])];

      if (type === 'allow') allow.splice(index, 1);
      if (type === 'deny') deny.splice(index, 1);
      if (type === 'redact') redact.splice(index, 1);
      if (type === 'requireApproval') requireApproval.splice(index, 1);

      const updatedPolicy = {
        ...current,
        allow,
        deny,
        redactKeys: redact,
        requireApproval,
      };

      const res = await api.upsertProfile(activeProfName, activeProf.servers, activeProf.description, updatedPolicy);
      if (!res.ok) {
        alert(`Failed to update profile policy: ${res.error || 'Unknown error'}`);
      }
    } else {
      // Remove rule from global policy
      const current = state.config.policy || {};
      const allow = [...(current.allow || [])];
      const deny = [...(current.deny || [])];
      const redact = [...(current.redact_keys || current.redactKeys || [])];
      const requireApproval = [...(current.require_approval || current.requireApproval || [])];

      if (type === 'allow') allow.splice(index, 1);
      if (type === 'deny') deny.splice(index, 1);
      if (type === 'redact') redact.splice(index, 1);
      if (type === 'requireApproval') requireApproval.splice(index, 1);

      const res = await api.savePolicy({
        ...current,
        allow,
        deny,
        redact_keys: redact,
        redactKeys: redact,
        require_approval: requireApproval,
        requireApproval,
      });
      if (!res.ok) {
        alert(`Failed to update policy: ${res.error || 'Unknown error'}`);
      }
    }
    await this.refreshData();
  }

  async saveWebhookConfig() {
    const urlEl = document.getElementById('policy-webhook-url') as HTMLInputElement | null;
    const formatEl = document.getElementById('policy-webhook-format') as HTMLSelectElement | null;
    const secretEl = document.getElementById('policy-webhook-secret') as HTMLInputElement | null;

    const url = urlEl ? urlEl.value.trim() : '';
    const format = formatEl ? (formatEl.value as any) : 'generic';
    const secret = secretEl ? secretEl.value.trim() : '';

    const state = store.getState();
    const current = state.config.policy || {};

    const webhook = url ? {
      url,
      format,
      secret: secret && !secret.startsWith('WARMPLANE_') && !secret.includes('_') ? secret : undefined,
      secret_env: secret && (secret.startsWith('WARMPLANE_') || secret.includes('_')) ? secret : undefined,
      events: ['approval.requested', 'circuit_breaker.tripped', 'policy.violation'],
    } : undefined;

    const res = await api.savePolicy({
      ...current,
      webhook,
    });

    if (res.ok) {
      alert('Webhook settings saved successfully');
    } else {
      alert(`Failed to save webhook settings: ${res.error || 'Unknown error'}`);
    }
    await this.refreshData();
  }

  async testWebhook() {
    const urlEl = document.getElementById('policy-webhook-url') as HTMLInputElement | null;
    const formatEl = document.getElementById('policy-webhook-format') as HTMLSelectElement | null;
    const url = urlEl ? urlEl.value.trim() : undefined;
    const format = formatEl ? formatEl.value : undefined;

    const statusEl = document.getElementById('policy-webhook-status');
    if (statusEl) {
      statusEl.textContent = 'Sending test event...';
      statusEl.style.color = 'var(--cyan-400)';
    }

    try {
      const res = await api.testWebhook(url, format);
      if (res.ok) {
        alert(`Test webhook sent successfully! (${res.message})`);
        if (statusEl) {
          statusEl.textContent = `✔ Test sent (HTTP ${res.status_code || 200})`;
          statusEl.style.color = 'var(--green-400)';
        }
      } else {
        alert(`Test webhook failed: ${res.error || 'Unknown error'}`);
        if (statusEl) {
          statusEl.textContent = `✖ Failed: ${res.error}`;
          statusEl.style.color = 'var(--red-400)';
        }
      }
    } catch (e: any) {
      alert(`Error sending test webhook: ${e.message}`);
    }
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
    const activeProfName = state.activeProfile;
    const activeProf = activeProfName ? state.config.profiles?.[activeProfName] : undefined;

    const basePolicy = state.config.policy || {};
    const profPolicy = activeProf?.policy;

    // Merge policy: deny union, hitl union, allow scoped to profile if present
    const baseDeny = basePolicy.deny || [];
    const profDeny = profPolicy?.deny || [];
    const deny = Array.from(new Set([...baseDeny, ...profDeny]));

    const baseHitl = basePolicy.require_approval || basePolicy.requireApproval || [];
    const profHitl = profPolicy?.require_approval || profPolicy?.requireApproval || [];
    const requireApproval = Array.from(new Set([...baseHitl, ...profHitl]));

    const allow = profPolicy && profPolicy.allow && profPolicy.allow.length > 0 
      ? profPolicy.allow 
      : (basePolicy.allow || []);

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

    if (requireApproval.some(r => wildcard(r, testId))) {
      verdictEl.textContent = 'REQUIRE APPROVAL (HITL Gate)';
      verdictEl.style.color = 'var(--amber-400)';
      return;
    }

    verdictEl.textContent = 'ALLOWED';
    verdictEl.style.color = 'var(--green-400)';
  }

  // Server Actions
  async deleteServer(name: string) {
    const activeProf = store.getState().activeProfile;
    const confirmMsg = activeProf
      ? `Permanently delete server '${name}' globally from warmplane configuration? (This will also unbind it from profile '${activeProf}')`
      : `Are you sure you want to permanently remove server '${name}' from configuration?`;

    if (!confirm(confirmMsg)) return;

    try {
      const res = await api.deleteServer(name);
      if (res.ok) {
        await this.refreshData();
      } else {
        alert(`Failed to remove server '${name}': ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error removing server '${name}': ${e.message}`);
    }
  }

  async restartServer(name: string) {
    try {
      const res = await api.restartServer(name);
      if (res.ok) {
        await this.refreshData();
      } else {
        alert(`Failed to restart server '${name}': ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error restarting server '${name}': ${e.message}`);
    }
  }

  openServerDiagnosticsModal(name: string) {
    this.closeModals();
    const state = store.getState();
    const serverCfg = state.config.mcpServers?.[name];
    const statusInfo = state.serverStatuses?.[name];
    const cb = (state.circuitBreakers || []).find(c => c.server_id === name);

    const modal = document.getElementById('modal-server-diagnostics');
    if (!modal) return;

    const titleEl = document.getElementById('modal-diag-title');
    const bodyEl = document.getElementById('modal-diag-body');
    if (titleEl) titleEl.textContent = `Live Diagnostics: ${name}`;

    if (bodyEl) {
      const isDegraded = statusInfo?.status === 'degraded';
      const statusColor = isDegraded ? 'var(--amber-400)' : statusInfo?.status === 'connected' ? 'var(--green-400)' : 'var(--red-400)';
      const errorMsg = statusInfo?.error || 'No active crash or error reported. Server is healthy.';

      const template = findTemplateForServer(name, serverCfg?.command, serverCfg?.args);
      const configuredEnv = serverCfg?.env || {};
      const configuredEnvKeys = Object.keys(configuredEnv);
      const requiredFields = (template?.envFields || []).filter(f => f.required);
      const missingRequired = requiredFields.filter(f => !configuredEnvKeys.includes(f.key));

      let envSectionHtml = '';
      if (template || configuredEnvKeys.length > 0) {
        const envItemsHtml = (template?.envFields || []).map(f => {
          const val = configuredEnv[f.key];
          const isSet = val !== undefined;
          let statusBadge = `<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Configured</span>`;
          if (!isSet && f.required) {
            statusBadge = `<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Required / Missing</span>`;
          } else if (!isSet) {
            statusBadge = `<span class="brand-badge" style="color: var(--text-dim); border-color: var(--border);">Optional / Not Set</span>`;
          }

          return `
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <div>
                <span style="font-family: var(--ff-mono); font-weight: 700; color: ${!isSet && f.required ? 'var(--amber-300)' : 'var(--text-main)'};">${escapeHtml(f.key)}</span>
                ${f.label ? `<span style="font-size: 10.5px; color: var(--text-dim); margin-left: 6px;">(${escapeHtml(f.label)})</span>` : ''}
              </div>
              <div style="display: flex; align-items: center; gap: 8px;">
                ${statusBadge}
                ${!isSet ? `
                  <button class="btn btn-primary" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.quickVaultEnv('${escapeHtml(name)}', '${escapeHtml(f.key)}')">➕ Configure in Keychain</button>
                ` : ''}
              </div>
            </div>
          `;
        }).join('');

        // Also list any custom env vars not in template
        const templateKeys = (template?.envFields || []).map(f => f.key);
        const extraEnvHtml = Object.entries(configuredEnv)
          .filter(([k]) => !templateKeys.includes(k))
          .map(([k, v]) => `
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <span style="font-family: var(--ff-mono); font-weight: 700; color: var(--text-main);">${escapeHtml(k)}</span>
              <span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Custom Configured</span>
            </div>
          `).join('');

        envSectionHtml = `
          <div style="background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
            <div style="font-size: 11px; font-weight: 700; color: ${missingRequired.length > 0 ? 'var(--amber-400)' : 'var(--text-main)'}; text-transform: uppercase; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <span>🔑 Environment Variables &amp; Secrets</span>
              ${missingRequired.length > 0 ? `<span style="color: var(--red-400); font-size: 10.5px;">⚠️ ${missingRequired.length} required key(s) missing</span>` : ''}
            </div>
            ${envItemsHtml}
            ${extraEnvHtml}
          </div>
        `;
      }

      bodyEl.innerHTML = `
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${statusColor};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${statusColor}; text-transform: uppercase;">${escapeHtml(statusInfo?.status || 'unknown')}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${escapeHtml(statusInfo?.protocol_version || '2026-07-28')}</span>
          ${missingRequired.length > 0 ? `<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Missing Required Keys</span>` : ''}
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${isDegraded ? 'var(--red-300)' : 'var(--text-dim)'}; white-space: pre-wrap; word-break: break-word; margin: 0;">${escapeHtml(errorMsg)}</pre>
        </div>

        ${envSectionHtml}

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Circuit Breaker State</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              ${cb ? `${cb.state.toUpperCase()} (${cb.consecutive_failures} failures)` : 'CLOSED (Healthy)'}
            </div>
          </div>
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Process Supervision</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              Auto-Restart: ${serverCfg?.resilience?.autoRestart !== false ? 'ENABLED' : 'DISABLED'}
            </div>
          </div>
        </div>

        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px; margin-bottom: 16px;">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-bottom: 4px;">Configured Execution Target</div>
          <code style="font-family: var(--ff-mono); font-size: 11px; color: var(--cyan-400); display: block; word-break: break-all;">
            ${serverCfg?.command ? `${escapeHtml(serverCfg.command)} ${escapeHtml((serverCfg.args || []).join(' '))}` : escapeHtml(serverCfg?.url || '')}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${escapeHtml(name)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `;
    }

    modal.classList.add('active');
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
    const nameInput = document.getElementById('modal-srv-name') as HTMLInputElement | null;
    const name = nameInput?.value.trim();
    const transport = (document.getElementById('modal-srv-transport') as HTMLSelectElement)?.value;
    if (!name) {
      alert('Server name is required');
      return;
    }

    // Only prompt overwrite check if user is adding a new server (input not disabled for edit)
    if (nameInput && !nameInput.disabled) {
      const state = store.getState();
      const existingServers = state.config.mcpServers || {};
      if (existingServers[name]) {
        if (!confirm(`Server '${name}' already exists in configuration. Do you want to overwrite it?`)) {
          return;
        }
      }
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
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${escapeHtml(t.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${escapeHtml(t.badge)}</span>
              </div>
              ${isAlreadyConfigured ? '<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>' : ''}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${escapeHtml(t.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${escapeHtml(cmdPreview)}</code>
            </div>
            ${t.envFields.length > 0 ? `
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${t.envFields.map(e => escapeHtml(e.key)).join(', ')}</code>
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

    // Auto-derive unique identifier if server already exists
    const configuredServers = store.getState().config.mcpServers || {};
    let initialServerId = tmpl.id;
    if (configuredServers[initialServerId]) {
      let counter = 2;
      while (configuredServers[`${tmpl.id}-${counter}`]) {
        counter++;
      }
      initialServerId = `${tmpl.id}-${counter}`;
    }

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
          <input type="text" class="form-input" id="cfg-srv-id" value="${escapeHtml(initialServerId)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
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

    const state = store.getState();
    const existingServers = state.config.mcpServers || {};
    if (existingServers[serverId]) {
      if (!confirm(`Server '${serverId}' already exists. Do you want to overwrite its configuration?`)) {
        return;
      }
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

  // Client Actions
  async refreshClients() {
    try {
      const res = await api.getClients();
      if (res.ok && Array.isArray(res.clients)) {
        store.setState({ clients: res.clients });
      }
    } catch (e) {
      console.error('Failed to scan clients:', e);
    }
  }

  setClientCategoryFilter(category: string) {
    store.setState({ clientFilterCategory: category });
  }

  setClientSearchQuery(query: string) {
    store.setState({ clientSearchQuery: query });
  }

  async attachClient(clientId: string, explicitProfile?: string) {
    let profile = explicitProfile;
    if (!profile) {
      const profSelect = (document.getElementById(`client-prof-${clientId}`) || document.getElementById(`overview-client-prof-${clientId}`)) as HTMLSelectElement | null;
      if (profSelect) {
        profile = profSelect.value || undefined;
      } else {
        profile = store.getState().activeProfile || undefined;
      }
    }
    const res = await api.attachClient(clientId, profile);
    if (!res.ok) {
      alert(`Failed to attach client: ${res.error || res.message || 'Unknown error'}`);
    } else {
      await this.refreshData();
    }
  }

  async detachClient(clientId: string) {
    if (!confirm(`Disconnect Warmplane from this client?`)) return;
    const res = await api.detachClient(clientId);
    if (!res.ok) {
      alert(`Failed to detach client: ${res.error || res.message || 'Unknown error'}`);
    } else {
      await this.refreshData();
    }
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
    const summary = (document.getElementById('alias-summary') as HTMLInputElement)?.value.trim() || undefined;
    const passthrough = (document.getElementById('alias-passthrough') as HTMLInputElement)?.checked || false;

    if (!name || !target) {
      alert('Please provide both alias name and canonical target');
      return;
    }

    await api.updateAlias(kind, name, target, summary, undefined, passthrough);
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

  renderTopProfileSelector() {
    const selectorEl = document.getElementById('top-profile-selector');
    if (!selectorEl) return;

    const state = store.getState();
    const profiles = state.config.profiles || {};
    const profKeys = Object.keys(profiles);
    const activeProf = state.activeProfile;

    let optionsHtml = `<option value="">All Servers (Unrestricted)</option>`;
    for (const k of profKeys) {
      const selected = activeProf === k ? 'selected' : '';
      optionsHtml += `<option value="${escapeHtml(k)}" ${selected}>Profile: ${escapeHtml(k)}</option>`;
    }

    selectorEl.innerHTML = optionsHtml;
  }

  async setActiveProfile(profileId: string | null) {
    store.setState({ activeProfile: profileId || null });
    await this.refreshData();
  }

  openAddProfileModal() {
    const titleEl = document.getElementById('modal-prof-title');
    if (titleEl) titleEl.textContent = 'Create Server Constellation Profile';

    const nameInput = document.getElementById('modal-prof-name') as HTMLInputElement | null;
    const descInput = document.getElementById('modal-prof-desc') as HTMLInputElement | null;
    const modeInput = document.getElementById('modal-prof-mode') as HTMLInputElement | null;

    if (nameInput) {
      nameInput.value = '';
      nameInput.disabled = false;
    }
    if (descInput) descInput.value = '';
    if (modeInput) modeInput.value = 'create';

    const allowInput = document.getElementById('modal-prof-allow') as HTMLInputElement | null;
    const denyInput = document.getElementById('modal-prof-deny') as HTMLInputElement | null;
    const hitlInput = document.getElementById('modal-prof-hitl') as HTMLInputElement | null;
    const redactInput = document.getElementById('modal-prof-redact') as HTMLInputElement | null;
    if (allowInput) allowInput.value = '';
    if (denyInput) denyInput.value = '';
    if (hitlInput) hitlInput.value = '';
    if (redactInput) redactInput.value = '';

    this.renderProfileServerCheckboxes([]);

    const modal = document.getElementById('modal-add-profile');
    if (modal) modal.classList.add('active');
  }

  openEditProfileModal(profileName: string) {
    const state = store.getState();
    const prof = state.config.profiles?.[profileName];
    if (!prof) return;

    const titleEl = document.getElementById('modal-prof-title');
    if (titleEl) titleEl.textContent = `Edit Profile: ${profileName}`;

    const nameInput = document.getElementById('modal-prof-name') as HTMLInputElement | null;
    const descInput = document.getElementById('modal-prof-desc') as HTMLInputElement | null;
    const modeInput = document.getElementById('modal-prof-mode') as HTMLInputElement | null;

    if (nameInput) {
      nameInput.value = profileName;
      nameInput.disabled = true;
    }
    if (descInput) descInput.value = prof.description || '';
    if (modeInput) modeInput.value = 'edit';

    const allowInput = document.getElementById('modal-prof-allow') as HTMLInputElement | null;
    const denyInput = document.getElementById('modal-prof-deny') as HTMLInputElement | null;
    const hitlInput = document.getElementById('modal-prof-hitl') as HTMLInputElement | null;
    const redactInput = document.getElementById('modal-prof-redact') as HTMLInputElement | null;

    const pol = prof.policy;
    if (allowInput) allowInput.value = (pol?.allow || []).join(', ');
    if (denyInput) denyInput.value = (pol?.deny || []).join(', ');
    if (hitlInput) hitlInput.value = (pol?.require_approval || pol?.requireApproval || []).join(', ');
    if (redactInput) redactInput.value = (pol?.redact_keys || pol?.redactKeys || []).join(', ');

    this.renderProfileServerCheckboxes(prof.servers || []);

    const modal = document.getElementById('modal-add-profile');
    if (modal) modal.classList.add('active');
  }

  renderProfileServerCheckboxes(selectedServers: string[]) {
    const container = document.getElementById('modal-prof-servers-list');
    if (!container) return;

    const state = store.getState();
    const allServers = Object.keys(state.config.mcpServers || {});

    if (allServers.length === 0) {
      container.innerHTML = `<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>`;
      return;
    }

    container.innerHTML = allServers.map(s => {
      const isChecked = selectedServers.includes(s) ? 'checked' : '';
      return `
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${escapeHtml(s)}" ${isChecked} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${escapeHtml(s)}</span>
        </label>
      `;
    }).join('');
  }

  async saveProfile() {
    const nameInput = document.getElementById('modal-prof-name') as HTMLInputElement | null;
    const descInput = document.getElementById('modal-prof-desc') as HTMLInputElement | null;
    const name = nameInput?.value.trim();
    const desc = descInput?.value.trim();

    if (!name) {
      alert('Please enter a profile name');
      return;
    }

    const checkboxes = document.querySelectorAll('.prof-server-checkbox:checked');
    const servers: string[] = [];
    checkboxes.forEach((cb) => {
      servers.push((cb as HTMLInputElement).value);
    });

    if (servers.length === 0) {
      alert('Please select at least one server to include in this constellation');
      return;
    }

    const parseCsv = (val: string | undefined): string[] => {
      if (!val) return [];
      return val.split(',').map(s => s.trim()).filter(s => s.length > 0);
    };

    const allowInput = document.getElementById('modal-prof-allow') as HTMLInputElement | null;
    const denyInput = document.getElementById('modal-prof-deny') as HTMLInputElement | null;
    const hitlInput = document.getElementById('modal-prof-hitl') as HTMLInputElement | null;
    const redactInput = document.getElementById('modal-prof-redact') as HTMLInputElement | null;

    const allow = parseCsv(allowInput?.value);
    const deny = parseCsv(denyInput?.value);
    const requireApproval = parseCsv(hitlInput?.value);
    const redactKeys = parseCsv(redactInput?.value);

    let policy: any = undefined;
    if (allow.length > 0 || deny.length > 0 || requireApproval.length > 0 || redactKeys.length > 0) {
      policy = {
        allow,
        deny,
        requireApproval,
        redactKeys,
      };
    }

    try {
      const res = await api.upsertProfile(name, servers, desc || undefined, policy);
      if (res.ok) {
        this.closeModals();
        await this.refreshData();
      } else {
        alert(`Failed to save profile: ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error saving profile: ${e.message}`);
    }
  }

  async deleteProfile(name: string) {
    if (!confirm(`Are you sure you want to delete profile '${name}'?`)) return;

    try {
      const res = await api.deleteProfile(name);
      if (res.ok) {
        if (store.getState().activeProfile === name) {
          store.setState({ activeProfile: null });
        }
        await this.refreshData();
      } else {
        alert(`Failed to delete profile: ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error deleting profile: ${e.message}`);
    }
  }

  async toggleServerInProfile(profileName: string, serverId: string, include: boolean) {
    const state = store.getState();
    const prof = state.config.profiles?.[profileName];
    if (!prof) return;

    let servers = [...(prof.servers || [])];
    if (include) {
      if (!servers.includes(serverId)) servers.push(serverId);
    } else {
      servers = servers.filter(s => s !== serverId);
      if (servers.length === 0) {
        alert('A profile must contain at least one server. To remove the profile, delete it in the Profiles tab.');
        return;
      }
    }

    try {
      const res = await api.upsertProfile(profileName, servers, prof.description, prof.policy);
      if (res.ok) {
        await this.refreshData();
      } else {
        alert(`Failed to update profile constellation: ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      alert(`Error updating profile constellation: ${e.message}`);
    }
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

