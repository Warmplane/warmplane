import { McpConfig, CapabilityItem, PendingApproval, AuditEventItem, VerificationReport, AuditStats, CircuitBreakerSnapshot } from './api';

export interface AppState {
  configPath: string;
  config: McpConfig;
  serverStatuses: Record<string, { transport: string; protocol_version: string; status: string }>;
  circuitBreakers: CircuitBreakerSnapshot[];
  capabilities: CapabilityItem[];
  approvals: PendingApproval[];
  auditEvents: AuditEventItem[];
  auditStats: AuditStats | null;
  auditVerification: VerificationReport | null;
  selectedCapabilityId: string | null;
  activeTab: 'overview' | 'servers' | 'playground' | 'approvals' | 'audit' | 'policy' | 'aliases';
  eventLogs: Array<{ time: string; method: string; target: string; status: string; latency: string }>;
  executionResult: { status: number; durationMs: number; data: any } | null;
  metrics: {
    totalCatalogRequests: number;
    totalEtagHits: number;
    totalToolCalls: number;
    totalToolDurationUs: number;
  };
}

type Listener = (state: AppState) => void;

class Store {
  private state: AppState = {
    configPath: 'mcp_servers.json',
    config: { mcpServers: {} },
    serverStatuses: {},
    circuitBreakers: [],
    capabilities: [],
    approvals: [],
    auditEvents: [],
    auditStats: null,
    auditVerification: null,
    selectedCapabilityId: null,
    activeTab: 'overview',
    eventLogs: [],
    executionResult: null,
    metrics: {
      totalCatalogRequests: 0,
      totalEtagHits: 0,
      totalToolCalls: 0,
      totalToolDurationUs: 0
    }
  };
  private listeners: Listener[] = [];

  getState(): AppState {
    return this.state;
  }

  setState(partial: Partial<AppState>) {
    this.state = { ...this.state, ...partial };
    this.listeners.forEach(l => l(this.state));
  }

  subscribe(listener: Listener): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter(l => l !== listener);
    };
  }

  addEventLog(method: string, target: string, status: string, latency: string) {
    const time = new Date().toLocaleTimeString();
    const newLogs = [{ time, method, target, status, latency }, ...this.state.eventLogs].slice(0, 50);
    this.setState({ eventLogs: newLogs });
  }
}

export const store = new Store();
