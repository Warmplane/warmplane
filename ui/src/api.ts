export interface McpServerConfig {
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  protocolVersion?: string;
  allowStateless?: boolean;
}

export interface PolicyConfig {
  allow?: string[];
  deny?: string[];
  redact_keys?: string[];
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
    const res = await fetch(`${this.baseUrl}/v1/config/policy`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(policy)
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
}

export const api = new WarmplaneClient();
