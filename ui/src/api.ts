export interface CompletionRequest {
  ref_type: 'prompt' | 'resource' | string;
  ref_name: string;
  argument_name: string;
  argument_value?: string;
}

export interface CompletionResponse {
  ok: boolean;
  trace_id: string;
  data: {
    ref_type: string;
    ref_name: string;
    argument_name: string;
    argument_value: string;
    values: string[];
    total: number;
    has_more: boolean;
  };
  error?: {
    code: string;
    message: string;
  };
}

export interface ResourceItem {
  id: string;
  server: string;
  uri: string;
  name: string;
  description?: string;
  mime_type?: string;
  tags?: string[];
}

export interface ListResourcesResponse {
  version: string;
  catalog_version: string;
  resources: ResourceItem[];
}

export interface ReadResourceRequest {
  resource_id: string;
  request_id?: string;
  context?: {
    operation_id?: string;
    actor_id?: string;
    grant_id?: string;
  };
  input_responses?: Record<string, any>;
  request_state?: string;
}

export interface PromptArgument {
  name: string;
  description?: string;
  required?: boolean;
}

export interface PromptItem {
  id: string;
  server: string;
  name: string;
  title?: string;
  description?: string;
  arguments?: PromptArgument[];
  tags?: string[];
}

export interface ListPromptsResponse {
  version: string;
  catalog_version: string;
  prompts: PromptItem[];
}

export interface GetPromptRequest {
  prompt_id: string;
  arguments?: Record<string, any>;
  request_id?: string;
  context?: {
    operation_id?: string;
    actor_id?: string;
    grant_id?: string;
  };
}

export interface CatalogEventItem {
  id?: string;
  cursor?: string;
  timestamp?: number | string;
  event_type?: string;
  type?: string;
  server?: string;
  server_id?: string;
  capability_id?: string;
  resource_id?: string;
  prompt_id?: string;
  details?: Record<string, any>;
}

export interface CatalogEventsResponse {
  catalog_version: string;
  cursor?: string;
  events: CatalogEventItem[];
}

export interface WebhookConfig {
  url: string;
  format?: 'generic' | 'slack' | 'discord' | 'teams';
  secret?: string;
  secretEnv?: string;
  secret_env?: string;
  authHeader?: string;
  auth_header?: string;
  events?: string[];
  callbackUrl?: string;
  callback_url?: string;
}

export interface PolicyConfig {
  allow?: string[];
  deny?: string[];
  redactKeys?: string[];
  redact_keys?: string[];
  requireApproval?: string[];
  require_approval?: string[];
  approvalTimeoutSecs?: number;
  approval_timeout_secs?: number;
  webhook?: WebhookConfig | string;
}

export interface ClientAppStatus {
  id: string;
  name: string;
  category: string;
  config_path: string;
  config_exists: boolean;
  app_installed: boolean;
  is_attached: boolean;
  attached_profile?: string | null;
  other_servers_count: number;
}

export interface TaskItem {
  taskId: string;
  status: 'working' | 'input_required' | 'completed' | 'cancelled' | 'failed' | string;
  progress?: number;
  total?: number;
  result?: any;
  error?: any;
  ttlSeconds?: number;
  createdAtEpochSecs?: number;
  expiresAtEpochSecs?: number;
  inputRequests?: Record<string, any>;
  capabilityId?: string;
  serverId?: string;
  context?: {
    operation_id?: string;
    actor_id?: string;
    grant_id?: string;
  };
}

export interface ListTasksResponse {
  ok: boolean;
  total: number;
  tasks: TaskItem[];
}

export interface UpdateTaskRequest {
  inputResponses: Record<string, any>;
}

export interface CancelTaskRequest {
  reason?: string;
}

export interface PendingApproval {
  id: string;
  capability_id: string;
  server_id: string;
  args?: Record<string, any>;
  sanitized_args?: Record<string, any>;
  request_id?: string;
  context?: {
    operation_id?: string;
    actor_id?: string;
    work_item_id?: string;
  };
  created_at: number;
  expires_at: number;
  status: 'pending' | 'approved' | 'rejected' | 'expired' | string;
  operator?: string;
  reason?: string;
  modified_args?: Record<string, any>;
}

export interface AuditEventItem {
  id: string;
  timestamp_ns: number;
  event_type: string;
  trace_id: string;
  request_id?: string;
  actor_id?: string;
  work_item_id?: string;
  client_ip?: string;
  server_id?: string;
  capability_id?: string;
  resource_uri?: string;
  sanitized_args?: Record<string, any>;
  sanitized_response?: Record<string, any>;
  execution_latency_us?: number;
  status: 'success' | 'failed' | 'denied' | 'intercepted' | 'cancelled' | string;
  error_code?: string;
  error_message?: string;
  operator_id?: string;
  approval_ticket_id?: string;
  prev_hash: string;
  hash: string;
}

export interface VerificationReport {
  is_valid: boolean;
  total_records: number;
  corrupted_at_index?: number;
  corrupted_record_id?: string;
  message?: string;
}

export interface AuditStats {
  ok: boolean;
  total_events: number;
  by_status: {
    success: number;
    failed: number;
    denied: number;
    intercepted: number;
  };
}

export interface AuditQueryOptions {
  actor_id?: string;
  server_id?: string;
  capability_id?: string;
  event_type?: string;
  status?: string;
  trace_id?: string;
  request_id?: string;
  search?: string;
  start_time?: number;
  end_time?: number;
  limit?: number;
  offset?: number;
}

export interface ResilienceConfig {
  failureThreshold?: number;
  cooldownMs?: number;
  consecutiveSuccesses?: number;
  autoRestart?: boolean;
  maxRestarts?: number;
  healthCheckIntervalSecs?: number;
}

export interface McpServerConfig {
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  protocolVersion?: string;
  allowStateless?: boolean;
  resilience?: ResilienceConfig;
}

export interface CircuitBreakerSnapshot {
  server_id: string;
  state: 'closed' | 'open' | 'half_open';
  consecutive_failures: number;
  consecutive_successes: number;
  open_until_epoch_ms?: number;
}

export interface RolePolicyConfig {
  description?: string;
  allow?: string[];
  deny?: string[];
  requireApproval?: string[];
  require_approval?: string[];
  redactKeys?: string[];
  redact_keys?: string[];
}

export interface TokenAssignment {
  role: string;
  tenantId?: string;
  actorId?: string;
  description?: string;
}

export interface RbacConfig {
  enabled: boolean;
  defaultRole?: string;
  tokens?: Record<string, TokenAssignment>;
  roles?: Record<string, RolePolicyConfig>;
}

export interface ProfileConfig {
  servers: string[];
  description?: string;
}

export interface McpConfig {
  mcpServers?: Record<string, McpServerConfig>;
  capabilityAliases?: Record<string, string>;
  resourceAliases?: Record<string, string>;
  promptAliases?: Record<string, string>;
  policy?: PolicyConfig;
  rbac?: RbacConfig;
  profiles?: Record<string, ProfileConfig>;
  resilience?: ResilienceConfig;
  audit?: Record<string, any>;
  toolTimeoutMs?: number;
}

export interface GetConfigResponse {
  ok: boolean;
  config_path: string;
  config: McpConfig;
  server_statuses?: Record<string, { transport: string; protocol_version: string; status: string }>;
  circuit_breakers?: CircuitBreakerSnapshot[];
  metrics?: {
    total_catalog_requests: number;
    total_etag_hits: number;
    total_tool_calls: number;
    total_tool_duration_us: number;
  };
  error?: string;
}

export interface CapabilityItem {
  id: string;
  server: string;
  summary: string;
  description: string;
  mode?: string;
  tags?: string[];
  input_schema?: Record<string, any>;
}

export interface ListCapabilitiesResponse {
  ok: boolean;
  version: string;
  catalog_version: string;
  capabilities: CapabilityItem[];
}

export interface CallCapabilityRequest {
  capability_id: string;
  args: Record<string, any>;
  request_id?: string;
  async_task?: boolean;
  context?: {
    operation_id?: string;
    actor_id?: string;
    grant_id?: string;
  };
  input_responses?: Record<string, any>;
  request_state?: string;
}

export interface EcosystemSource {
  name: string;
  path: string;
  server_count: number;
  servers: string[];
}

export class WarmplaneClient {
  private baseUrl: string;

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
  }

  async getConfig(): Promise<GetConfigResponse> {
    const res = await fetch(`${this.baseUrl}/v1/config`);
    return res.json();
  }

  async listCapabilities(profile?: string): Promise<ListCapabilitiesResponse> {
    const headers: Record<string, string> = {};
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/capabilities`, { headers });
    return res.json();
  }

  async listResources(profile?: string): Promise<ListResourcesResponse> {
    const headers: Record<string, string> = {};
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/resources`, { headers });
    return res.json();
  }

  async readResource(req: ReadResourceRequest, profile?: string): Promise<{ status: number; durationMs: number; data: any }> {
    const start = performance.now();
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/resources/read`, {
      method: 'POST',
      headers,
      body: JSON.stringify(req)
    });
    const durationMs = performance.now() - start;
    const data = await res.json();
    return { status: res.status, durationMs, data };
  }

  async listPrompts(profile?: string): Promise<ListPromptsResponse> {
    const headers: Record<string, string> = {};
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/prompts`, { headers });
    return res.json();
  }

  async getPrompt(req: GetPromptRequest, profile?: string): Promise<{ status: number; durationMs: number; data: any }> {
    const start = performance.now();
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/prompts/get`, {
      method: 'POST',
      headers,
      body: JSON.stringify(req)
    });
    const durationMs = performance.now() - start;
    const data = await res.json();
    return { status: res.status, durationMs, data };
  }

  async getCatalogEvents(after?: string): Promise<CatalogEventsResponse> {
    const q = after ? `?after=${encodeURIComponent(after)}` : '';
    const res = await fetch(`${this.baseUrl}/v1/catalog/events${q}`);
    return res.json();
  }

  async callCapability(req: CallCapabilityRequest, profile?: string): Promise<{ status: number; durationMs: number; data: any }> {
    const start = performance.now();
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/tools/call`, {
      method: 'POST',
      headers,
      body: JSON.stringify(req)
    });
    const durationMs = performance.now() - start;
    const data = await res.json();
    return { status: res.status, durationMs, data };
  }

  async batchCallCapabilities(steps: Array<{ id: string; capability_id: string; args: Record<string, any>; continue_on_error?: boolean }>, profile?: string): Promise<{ status: number; durationMs: number; data: any }> {
    const start = performance.now();
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (profile) headers['X-Warmplane-Profile'] = profile;
    const res = await fetch(`${this.baseUrl}/v1/tools/batch_call`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ steps })
    });
    const durationMs = performance.now() - start;
    const data = await res.json();
    return { status: res.status, durationMs, data };
  }

  async cancelOperation(id: string): Promise<{ ok: boolean; request_id: string; cancelled: boolean; error?: any }> {
    const res = await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(id)}/cancel`, {
      method: 'POST'
    });
    return res.json();
  }

  async completeArgument(req: CompletionRequest): Promise<CompletionResponse> {
    const res = await fetch(`${this.baseUrl}/v1/completion/complete`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req)
    });
    return res.json();
  }

  async upsertServer(name: string, server: McpServerConfig): Promise<{ ok: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/servers`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, server })
    });
    return res.json();
  }

  async deleteServer(name: string): Promise<{ ok: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(name)}`, {
      method: 'DELETE'
    });
    return res.json();
  }

  async upsertProfile(name: string, servers: string[], description?: string): Promise<{ ok: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/profiles`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, servers, description })
    });
    return res.json();
  }

  async deleteProfile(name: string): Promise<{ ok: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(name)}`, {
      method: 'DELETE'
    });
    return res.json();
  }

  async getEcosystemSources(): Promise<{ ok: boolean; sources: EcosystemSource[] }> {
    const res = await fetch(`${this.baseUrl}/v1/config/ecosystem`);
    return res.json();
  }

  async importConfig(sourcePath?: string, overwrite: boolean = false): Promise<{ ok: boolean; imported_count: number; skipped_servers: string[] }> {
    const res = await fetch(`${this.baseUrl}/v1/config/import`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source_path: sourcePath, overwrite })
    });
    return res.json();
  }

  async savePolicy(policy: PolicyConfig): Promise<{ ok: boolean; error?: string }> {
    const payload = {
      allow: policy.allow || [],
      deny: policy.deny || [],
      redactKeys: policy.redact_keys || policy.redactKeys || [],
      requireApproval: policy.require_approval || policy.requireApproval || [],
      approvalTimeoutSecs: policy.approvalTimeoutSecs || policy.approval_timeout_secs || 300,
      webhook: policy.webhook
    };
    const res = await fetch(`${this.baseUrl}/v1/config/policy`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    return res.json();
  }

  async listTasks(): Promise<ListTasksResponse> {
    const res = await fetch(`${this.baseUrl}/v1/tasks`);
    return res.json();
  }

  async getTask(id: string): Promise<{ ok: boolean; resultType?: string; task?: TaskItem; error?: any }> {
    const res = await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(id)}`);
    return res.json();
  }

  async updateTask(id: string, inputResponses: Record<string, any>): Promise<{ ok: boolean; resultType?: string; message?: string; error?: any }> {
    const res = await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(id)}/update`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ inputResponses })
    });
    return res.json();
  }

  async cancelTask(id: string, reason?: string): Promise<{ ok: boolean; resultType?: string; cancelled?: boolean; message?: string; error?: any }> {
    const res = await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(id)}/cancel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ reason })
    });
    return res.json();
  }

  async listApprovals(): Promise<{ ok: boolean; approvals: PendingApproval[]; total: number }> {
    const res = await fetch(`${this.baseUrl}/v1/approvals`);
    return res.json();
  }

  async approveTicket(id: string, operator: string, modifiedArgs?: Record<string, any>): Promise<{ ok: boolean; message?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(id)}/approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ operator, modified_args: modifiedArgs })
    });
    return res.json();
  }

  async rejectTicket(id: string, operator: string, reason?: string): Promise<{ ok: boolean; message?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(id)}/reject`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ operator, reason })
    });
    return res.json();
  }

  async updateAlias(kind: string, alias: string, target?: string): Promise<{ ok: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/alias`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ kind, alias, target })
    });
    return res.json();
  }

  async reloadConfig(): Promise<{ ok: boolean; mounted?: string[]; unmounted?: string[]; warnings?: string[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/config/reload`, {
      method: 'POST'
    });
    return res.json();
  }

  async listAuditEvents(params?: AuditQueryOptions): Promise<{ ok: boolean; events: AuditEventItem[]; total: number; limit: number; offset: number }> {
    const q = new URLSearchParams();
    if (params?.actor_id) q.set('actor_id', params.actor_id);
    if (params?.server_id && params.server_id !== 'all') q.set('server_id', params.server_id);
    if (params?.capability_id) q.set('capability_id', params.capability_id);
    if (params?.event_type && params.event_type !== 'all') q.set('event_type', params.event_type);
    if (params?.status && params.status !== 'all') q.set('status', params.status);
    if (params?.trace_id) q.set('trace_id', params.trace_id);
    if (params?.request_id) q.set('request_id', params.request_id);
    if (params?.search) q.set('search', params.search);
    if (params?.limit) q.set('limit', String(params.limit));
    if (params?.offset !== undefined) q.set('offset', String(params.offset));
    const qs = q.toString();
    const res = await fetch(`${this.baseUrl}/v1/audit/events${qs ? `?${qs}` : ''}`);
    return res.json();
  }

  getAuditExportUrl(params?: AuditQueryOptions, format: 'csv' | 'jsonl' = 'csv'): string {
    const q = new URLSearchParams();
    q.set('format', format);
    if (params?.actor_id) q.set('actor_id', params.actor_id);
    if (params?.server_id && params.server_id !== 'all') q.set('server_id', params.server_id);
    if (params?.capability_id) q.set('capability_id', params.capability_id);
    if (params?.event_type && params.event_type !== 'all') q.set('event_type', params.event_type);
    if (params?.status && params.status !== 'all') q.set('status', params.status);
    if (params?.trace_id) q.set('trace_id', params.trace_id);
    if (params?.request_id) q.set('request_id', params.request_id);
    if (params?.search) q.set('search', params.search);
    return `${this.baseUrl}/v1/audit/export?${q.toString()}`;
  }

  async verifyAuditChain(): Promise<{ ok: boolean; report: VerificationReport }> {
    const res = await fetch(`${this.baseUrl}/v1/audit/verify`);
    return res.json();
  }

  async getAuditStats(): Promise<{ ok: boolean; total_events: number; by_status: { success: number; failed: number; denied: number; intercepted: number } }> {
    const res = await fetch(`${this.baseUrl}/v1/audit/stats`);
    return res.json();
  }

  async getClients(): Promise<{ ok: boolean; clients: ClientAppStatus[] }> {
    const res = await fetch(`${this.baseUrl}/v1/clients`);
    return res.json();
  }

  async attachClient(clientId: string, profile?: string): Promise<{ ok: boolean; message: string; backup_path?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(clientId)}/attach`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ profile: profile || undefined }),
    });
    return res.json();
  }

  async detachClient(clientId: string): Promise<{ ok: boolean; message: string; was_attached?: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(clientId)}/detach`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
    return res.json();
  }

  async testWebhook(url?: string, format?: string): Promise<{ ok: boolean; message?: string; error?: string; status_code?: number }> {
    const res = await fetch(`${this.baseUrl}/v1/webhooks/test`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: url || undefined, format: format || undefined }),
    });
    return res.json();
  }
}

export const api = new WarmplaneClient();


