export interface McpServerConfig {
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  protocolVersion?: string;
  allowStateless?: boolean;
}

export interface WebhookConfig {
  url: string;
  secret?: string;
  secretEnv?: string;
  authHeader?: string;
  headers?: Record<string, string>;
}

export interface PolicyConfig {
  allow?: string[];
  deny?: string[];
  redact_keys?: string[];
  redactKeys?: string[];
  requireApproval?: string[];
  require_approval?: string[];
  approvalTimeoutSecs?: number;
  approval_timeout_secs?: number;
  webhook?: WebhookConfig;
}

export interface PendingApproval {
  id: string;
  capability_id: string;
  server_id: string;
  args: Record<string, any>;
  sanitized_args: Record<string, any>;
  request_id?: string;
  context?: {
    operation_id?: string;
    actor_id?: string;
    grant_id?: string;
  };
  created_at: number;
  expires_at: number;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  operator?: string;
  reason?: string;
  modified_args?: Record<string, any>;
  timestamp?: number;
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
  status: 'success' | 'failed' | 'denied' | 'intercepted' | 'cancelled';
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
  total_events: number;
  by_status: {
    success: number;
    failed: number;
    denied: number;
    intercepted: number;
  };
}

export interface McpConfig {
  mcpServers?: Record<string, McpServerConfig>;
  capabilityAliases?: Record<string, string>;
  resourceAliases?: Record<string, string>;
  promptAliases?: Record<string, string>;
  policy?: PolicyConfig;
  toolTimeoutMs?: number;
}

export interface GetConfigResponse {
  ok: boolean;
  config_path: string;
  config: McpConfig;
  server_statuses?: Record<string, { transport: string; protocol_version: string; status: string }>;
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

  async listCapabilities(): Promise<ListCapabilitiesResponse> {
    const res = await fetch(`${this.baseUrl}/v1/capabilities`);
    return res.json();
  }

  async callCapability(req: CallCapabilityRequest): Promise<{ status: number; durationMs: number; data: any }> {
    const start = performance.now();
    const res = await fetch(`${this.baseUrl}/v1/tools/call`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req)
    });
    const durationMs = performance.now() - start;
    const data = await res.json();
    return { status: res.status, durationMs, data };
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

  async listAuditEvents(params?: { actor_id?: string; capability_id?: string; event_type?: string; limit?: number; offset?: number }): Promise<{ ok: boolean; events: AuditEventItem[]; total: number }> {
    const q = new URLSearchParams();
    if (params?.actor_id) q.set('actor_id', params.actor_id);
    if (params?.capability_id) q.set('capability_id', params.capability_id);
    if (params?.event_type) q.set('event_type', params.event_type);
    if (params?.limit) q.set('limit', String(params.limit));
    if (params?.offset) q.set('offset', String(params.offset));
    const qs = q.toString();
    const res = await fetch(`${this.baseUrl}/v1/audit/events${qs ? `?${qs}` : ''}`);
    return res.json();
  }

  async verifyAuditChain(): Promise<{ ok: boolean; report: VerificationReport }> {
    const res = await fetch(`${this.baseUrl}/v1/audit/verify`);
    return res.json();
  }

  async getAuditStats(): Promise<{ ok: boolean; total_events: number; by_status: { success: number; failed: number; denied: number; intercepted: number } }> {
    const res = await fetch(`${this.baseUrl}/v1/audit/stats`);
    return res.json();
  }
}

export const api = new WarmplaneClient();
