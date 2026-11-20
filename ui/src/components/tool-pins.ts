import { api, type ToolPinDefinition, type ToolPinReview } from '../api';
import { escapeHtml } from '../escape';

const statusLabels: Record<ToolPinReview['status'], string> = {
  missing: 'Approval needed',
  matching: 'Approved',
  changed: 'Definition changed',
  unavailable: 'Unavailable',
};

function definition(label: string, value: ToolPinDefinition | null): string {
  const rendered = value ? JSON.stringify(value.value, null, 2) : 'No definition available';
  return `
    <div class="tool-pin-definition">
      <strong>${label}</strong>
      ${value ? '<code>' + escapeHtml(value.digest) + '</code>' : ''}
      <pre tabindex="0">${escapeHtml(rendered)}</pre>
    </div>
  `;
}

function pinReview(pin: ToolPinReview, index: number): string {
  const canApprove = pin.tool && pin.current && pin.status !== 'matching';
  return `
    <details class="tool-pin-review">
      <summary><code>${escapeHtml(pin.server)}${pin.tool ? ' / ' + escapeHtml(pin.tool) : ''}</code>
        <span class="tool-pin-status" data-status="${escapeHtml(pin.status)}">${statusLabels[pin.status]}</span>
      </summary>
      <div class="tool-pin-comparison">
        ${definition('Observed definition', pin.current)}
        ${definition('Approved definition', pin.approved)}
      </div>
      ${canApprove ? '<button type="button" class="btn btn-primary" data-approve-pin="' + index + '">Approve this definition</button>' : ''}
    </details>
  `;
}

export function mountToolPinReview(panel: HTMLElement) {
  panel.innerHTML = `
    <h2 class="stat-label">Tool definition pins</h2>
    <form class="trust-auth-form">
      <div class="form-group">
        <label for="tool-pin-token" class="form-label">Trust administrator token</label>
        <input id="tool-pin-token" class="form-input" type="password" autocomplete="off">
      </div>
      <button class="btn btn-ghost" type="submit">Load definitions</button>
    </form>
    <p class="tool-pin-feedback" role="status" aria-live="polite">Load definitions for servers with tool pinning enabled.</p>
    <div class="tool-pin-list"></div>
  `;
  const form = panel.querySelector<HTMLFormElement>('form')!;
  const token = panel.querySelector<HTMLInputElement>('#tool-pin-token')!;
  const feedback = panel.querySelector<HTMLElement>('[role="status"]')!;
  const list = panel.querySelector<HTMLElement>('.tool-pin-list')!;
  let pins: ToolPinReview[] = [];
  let busy = false;

  async function refresh(message?: string) {
    const result = await api.listToolPins(token.value);
    pins = result.pins ?? [];
    list.innerHTML = pins.map(pinReview).join('');
    feedback.textContent = result.error ?? message
      ?? (pins.length ? `${pins.length} definitions and server statuses loaded.` : 'No tool definitions are awaiting review.');
  }

  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true;
    token.readOnly = true;
    panel.setAttribute('aria-busy', 'true');
    try {
      await action();
    } catch {
      feedback.textContent = 'Could not reach Warmplane. Load definitions again when the daemon is available.';
    } finally {
      panel.removeAttribute('aria-busy');
      token.readOnly = false;
      busy = false;
    }
  }

  form.addEventListener('submit', event => {
    event.preventDefault();
    list.replaceChildren();
    feedback.textContent = 'Loading definitions…';
    void run(() => refresh());
  });
  list.addEventListener('click', event => {
    if (!(event.target instanceof Element)) return;
    const button = event.target.closest<HTMLButtonElement>('[data-approve-pin]');
    if (!button) return;
    const pin = pins[Number(button.dataset.approvePin)];
    if (!pin?.tool || !pin.current) return;
    const approval = { server: pin.server, tool: pin.tool, digest: pin.current.digest };
    void run(async () => {
      feedback.textContent = 'Saving approval…';
      const result = await api.approveToolPin(token.value, approval);
      await refresh(result.error ?? 'Definition approved.');
    });
  });
}
