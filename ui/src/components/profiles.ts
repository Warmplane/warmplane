import { store } from '../state';
import { api } from '../api';

export function renderProfiles(): string {
  const state = store.getState();
  const cfg = state.config;
  const profiles = cfg.profiles || {};
  const entries = Object.entries(profiles);
  const servers = cfg.mcpServers || {};
  const activeProf = state.activeProfile;

  let profilesListHtml = '';
  if (entries.length === 0) {
    profilesListHtml = `
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;
  } else {
    profilesListHtml = entries.map(([name, prof]) => {
      const isSelected = activeProf === name;
      const serverBadges = prof.servers.map(s => {
        const srvExists = !!servers[s];
        const colorStyle = srvExists 
          ? 'color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);'
          : 'color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);';
        return `<span class="brand-badge" style="${colorStyle}">${escapeHtml(s)}</span>`;
      }).join(' ');

      // Count matching capabilities
      const matchingCapsCount = (state.capabilities || []).filter(c => prof.servers.includes(c.server)).length;

      // Policy stats if custom policy is defined
      const hasPolicy = !!prof.policy;
      const allowCount = prof.policy?.allow?.length || 0;
      const denyCount = prof.policy?.deny?.length || 0;
      const hitlCount = (prof.policy?.require_approval || prof.policy?.requireApproval || []).length;
      const redactCount = (prof.policy?.redact_keys || prof.policy?.redactKeys || []).length;

      return `
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${isSelected ? '3px solid var(--amber-400)' : '1px solid var(--border)'};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${escapeHtml(name)}</span>
                ${isSelected ? '<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>' : ''}
                <span class="brand-badge">${prof.servers.length} server${prof.servers.length === 1 ? '' : 's'}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${matchingCapsCount} capabilities</span>
                ${hasPolicy ? '<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>' : ''}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${escapeHtml(prof.description || 'No description provided')}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${hasPolicy ? '8px' : '0'};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${serverBadges || '<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${hasPolicy ? `
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${allowCount > 0 ? `<span class="brand-badge" style="color: var(--green-400);">Allow: ${allowCount}</span>` : ''}
                  ${denyCount > 0 ? `<span class="brand-badge" style="color: var(--red-400);">Deny: ${denyCount}</span>` : ''}
                  ${hitlCount > 0 ? `<span class="brand-badge" style="color: var(--amber-400);">HITL: ${hitlCount}</span>` : ''}
                  ${redactCount > 0 ? `<span class="brand-badge" style="color: var(--text-muted);">Redact: ${redactCount}</span>` : ''}
                  ${allowCount === 0 && denyCount === 0 && hitlCount === 0 && redactCount === 0 ? `<span style="color: var(--text-dim);">Configured</span>` : ''}
                </div>
              ` : ''}
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${isSelected ? `
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              ` : `
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${escapeHtml(name)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${escapeHtml(name)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${escapeHtml(name)}')">
                Remove
              </button>
            </div>
          </div>
        </div>
      `;
    }).join('');
  }

  return `
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="font-size: 12px; color: var(--text-dim);">
        Define named subsets of servers for task-specific agent interactions, dynamic per-request switching, and scoped ETag caching.
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create Profile</button>
      </div>
    </div>

    <!-- Quick Info Box -->
    <div class="bento-card" style="margin-bottom: 18px; background: rgba(245, 158, 11, 0.03); border: 1px solid rgba(245, 158, 11, 0.15);">
      <div style="display: flex; gap: 12px; align-items: center;">
        <span style="font-size: 20px;">💡</span>
        <div style="font-size: 12px; color: var(--text-muted); line-height: 1.5;">
          HTTP clients can select profiles dynamically using the <code style="color: var(--amber-400);">X-Warmplane-Profile: &lt;name&gt;</code> header or <code style="color: var(--amber-400);">?profile=&lt;name&gt;</code> query parameter. MCP stdio clients can pass <code style="color: var(--amber-400);">--profile &lt;name&gt;</code>.
        </div>
      </div>
    </div>

    ${profilesListHtml}
  `;
}

function escapeHtml(str: string): string {
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
