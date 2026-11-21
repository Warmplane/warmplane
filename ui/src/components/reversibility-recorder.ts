import { api } from '../api';
import type { ReversibilityConfig, ReversibilityDraft, ReversibilityDraftRequest, ReversibilityReply } from '../api';
import { store } from '../state';

export function renderReversibilityRecorder(): string {
  return `<section data-reversibility-recorder data-reversibility-class="reversible" hidden style="padding-top:12px;margin-top:12px;border-top:1px solid var(--border);">
    <h4 style="font-size:12px;margin:0 0 8px;">Test and activate a draft</h4>
    <div style="display:flex;gap:8px;flex-wrap:wrap;">
      <button type="button" class="btn btn-ghost" data-recorder="generate">Generate draft</button>
      <button type="button" class="btn btn-ghost" data-recorder="export" disabled>Export JSON</button>
      <button type="button" class="btn btn-ghost" data-recorder="record" disabled>Run isolated test</button>
      <button type="button" class="btn btn-primary" data-recorder="activate" disabled>Activate tested contract</button>
    </div>
    <div data-recorder="preview" hidden style="margin-top:10px;">
      <p data-recorder="target" class="text-dim" style="font-size:11px;margin:0 0 8px;"></p>
      <label style="display:flex;gap:8px;font-size:12px;align-items:flex-start;">
        <input type="checkbox" data-recorder="confirm">
        Confirm one forward call and cleanup on this test target.
      </label>
    </div>
    <p data-recorder="status" role="status" class="text-dim" style="font-size:11px;margin:8px 0 0;"></p>
  </section>`;
}

interface Candidate {
  alias: string;
  request: ReversibilityDraftRequest;
  profile?: string;
}

function getCandidate(form: HTMLFormElement, read: () => { contract?: ReversibilityConfig | null; errors: Map<string, string> }): Candidate {
  const { contract, errors } = read();
  const value = (id: string) => form.querySelector<HTMLInputElement>(`#${id}`)?.value.trim() || '';
  if (errors.size) throw new Error(errors.values().next().value);
  if (contract?.class !== 'reversible' || !contract.ttlSecs || !contract.preSnapshot
      || !contract.preUndoCheck || !contract.inverse || !contract.testTarget) {
    throw new Error('Set an undo window, snapshot, drift check, inverse, and isolated test target.');
  }
  return {
    alias: value('alias-name'),
    profile: store.getState().activeProfile || undefined,
    request: {
      forward: value('alias-target'), ttlSecs: contract.ttlSecs, preSnapshot: contract.preSnapshot,
      inverse: contract.inverse, preUndoCheck: contract.preUndoCheck, testTarget: contract.testTarget,
    },
  };
}

export function bindReversibilityRecorder(
  form: HTMLFormElement,
  read: () => { contract?: ReversibilityConfig | null; errors: Map<string, string> },
  onSaved: () => Promise<void>,
) {
  const container = form.querySelector<HTMLElement>('[data-reversibility-recorder]');
  if (container) new Recorder(container, () => getCandidate(form, read), onSaved).attach(form);
}

type Action = 'generate' | 'record' | 'activate';

class Recorder {
  private draft?: ReversibilityDraft;
  private proof?: string;
  private fingerprint = '';
  private confirmationToken = '';
  private testForward = '';
  private working = false;

  constructor(
    private container: HTMLElement,
    private candidate: () => Candidate,
    private onSaved: () => Promise<void>,
  ) {}

  attach(form: HTMLFormElement) {
    for (const action of ['generate', 'record', 'activate'] as const) {
      this.control(action).addEventListener('click', () => void this.run(action));
    }
    this.control('export').addEventListener('click', () => this.exportCurrent());
    this.confirmation.addEventListener('change', () => this.render());
    form.addEventListener('input', event => {
      if (event.target === this.confirmation) return;
      if (this.draft) this.setStatus('Inputs changed. Generate a fresh draft.');
      this.reset();
    });
    this.render();
  }

  private exportCurrent() {
    try {
      this.requireCurrent(this.candidate());
      if (this.draft) downloadDraft(this.draft.contract);
    } catch {
      this.setStatus('Inputs changed. Generate a fresh draft.');
    }
  }

  private control<T extends HTMLElement = HTMLElement>(name: string): T {
    return this.container.querySelector<T>(`[data-recorder="${name}"]`)!;
  }

  private get confirmation() {
    return this.control<HTMLInputElement>('confirm');
  }

  private render() {
    this.control<HTMLButtonElement>('generate').disabled = this.working;
    this.control<HTMLButtonElement>('export').disabled = this.working || !this.draft;
    this.control<HTMLButtonElement>('record').disabled = this.working || !this.draft || !this.confirmation.checked;
    this.control<HTMLButtonElement>('activate').disabled = this.working || !this.proof;
    this.control('preview').hidden = !this.draft;
  }

  private reset() {
    this.draft = undefined;
    this.proof = undefined;
    this.fingerprint = '';
    this.confirmationToken = '';
    this.testForward = '';
    this.confirmation.checked = false;
    this.render();
  }

  private setStatus(message: string) {
    this.control('status').textContent = message;
  }

  private async run(action: Action) {
    if (this.working) return;
    try {
      const candidate = this.candidate();
      if (action !== 'generate') this.requireCurrent(candidate);
      this.working = true;
      this.render();
      const response = await request(candidate, action, this.draft, this.proof, this.confirmationToken, this.confirmation.checked);
      await this.accept(action, response, candidate);
    } catch (error) {
      if (action === 'record') this.reset();
      this.setStatus(error instanceof Error ? error.message : 'The recorder request failed.');
    } finally {
      this.working = false;
      this.render();
    }
  }

  private requireCurrent(candidate: Candidate) {
    if (JSON.stringify(candidate) !== this.fingerprint) {
      this.reset();
      throw new Error('Inputs changed. Generate a fresh draft.');
    }
  }

  private async accept(action: Action, response: ReversibilityReply, candidate: Candidate) {
    if (!response.ok) throw new Error(replyError(response));
    if (action === 'activate') {
      this.reset();
      await this.onSaved();
      this.setStatus('Tested contract saved.');
      return;
    }
    this.fingerprint = JSON.stringify(candidate);
    this.requireCurrent(this.candidate());
    if (!response.draft) throw new Error('The daemon did not return a draft.');
    this.updatePreview(response, response.draft, candidate);
  }

  private updatePreview(response: ReversibilityReply, draft: ReversibilityDraft, candidate: Candidate) {
    this.draft = draft;
    this.confirmationToken = response.confirmationToken ?? this.confirmationToken;
    this.testForward = response.testForward ?? this.testForward;
    this.proof = response.reviewToken;
    this.confirmation.checked = false;
    const test = candidate.request.testTarget;
    const fields = Object.keys(test.args).map(key => `${key}: [REDACTED]`).join(', ');
    this.control('target').textContent = `Target: ${test.server} · Forward: ${this.testForward} · Arguments: ${fields || 'none'}`;
    this.setStatus(this.proof
      ? 'Test passed. Cleanup restored the starting state. Activate the reviewed contract when ready.'
      : 'Draft validated. Review the target and confirm before starting the test.');
  }
}

function request(candidate: Candidate, action: 'generate' | 'record' | 'activate', draft: ReversibilityDraft | undefined, proof: string | undefined, confirmationToken: string, confirmed: boolean): Promise<ReversibilityReply> {
  if (action === 'generate') return api.reversibility('draft', candidate.request, candidate.profile);
  if (!draft) throw new Error('Generate a draft first.');
  const payload = { draft: candidate.request, revisions: draft.revisions };
  if (action === 'record') {
    if (!confirmed) throw new Error('Confirm the isolated test first.');
    return api.reversibility('record', { ...payload, confirmationToken, confirmed }, candidate.profile);
  }
  if (!proof) throw new Error('Pass the isolated test before activating.');
  return api.reversibility('activate', { ...payload, reviewToken: proof, alias: candidate.alias }, candidate.profile);
}

function replyError(response: ReversibilityReply): string {
  const detail = typeof response.error === 'string' ? response.error : response.error?.message;
  const result = response.result?.error_code;
  if (result === 'UNDO_TEST_FORWARD_OUTCOME_UNKNOWN') {
    return 'The test action may have taken effect. Check and reset the test fixture before another run.';
  }
  return detail || (result ? `Test failed (${result}). Check the test fixture state.` : 'The recorder request failed.');
}

function downloadDraft(contract: ReversibilityConfig) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(contract, null, 2)], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = 'reversibility-draft.json';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
