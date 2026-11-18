import { escapeHtml } from '../escape';
import { store } from '../state';

function listFor(input: HTMLInputElement) {
  return document.getElementById(input.getAttribute('aria-controls') || '');
}

export function closeCapabilitySuggestions(input: HTMLInputElement) {
  const list = listFor(input);
  if (list) list.hidden = true;
  input.setAttribute('aria-expanded', 'false');
  input.removeAttribute('aria-activedescendant');
}

function showSuggestions(input: HTMLInputElement) {
  closeCapabilitySuggestions(input);
  const list = listFor(input);
  const query = input.value.trim().toLowerCase();
  if (!list || query.length < 2) return;

  const matches = store.getState().capabilities.filter(capability =>
    [capability.id, capability.summary, capability.description, capability.server]
      .some(value => value?.toLowerCase().includes(query))
  ).slice(0, 8);
  if (!matches.length) return;

  list.innerHTML = matches.map((capability, index) => `
    <div id="${input.id}-option-${index}" role="option" aria-selected="false"
         data-capability-id="${escapeHtml(capability.id)}" class="capability-option">
      <span><strong>${escapeHtml(capability.id)}</strong>
        <small>${escapeHtml(capability.summary || capability.description || '')}</small></span>
      <span class="capability-option-server">${escapeHtml(capability.server || 'local')}</span>
    </div>`).join('');
  list.hidden = false;
  input.setAttribute('aria-expanded', 'true');
}

function selectOption(input: HTMLInputElement, option: HTMLElement) {
  input.value = option.dataset.capabilityId ?? input.value;
  closeCapabilitySuggestions(input);
  input.dispatchEvent(new Event('change', { bubbles: true }));
}

export function bindCapabilityAutocomplete(form: HTMLFormElement) {
  const combobox = (target: EventTarget | null) =>
    target instanceof HTMLInputElement && target.getAttribute('role') === 'combobox' ? target : null;

  for (const type of ['input', 'focusin', 'focusout']) {
    form.addEventListener(type, event => {
      const input = combobox(event.target);
      if (!input) return;
      if (type === 'focusout') closeCapabilitySuggestions(input);
      else showSuggestions(input);
    });
  }

  form.addEventListener('mousedown', event => {
    if (event.target instanceof Element && event.target.closest('[data-capability-id]')) {
      event.preventDefault();
    }
  });
  form.addEventListener('click', event => {
    if (!(event.target instanceof Element)) return;
    const option = event.target.closest<HTMLElement>('[data-capability-id]');
    const input = combobox(option?.parentElement?.previousElementSibling ?? null);
    if (option && input) selectOption(input, option);
  });

  form.addEventListener('keydown', event => {
    const input = combobox(event.target);
    if (!input || event.isComposing) return;
    const list = listFor(input);
    if (!list) return;

    const arrow = event.key === 'ArrowDown' || event.key === 'ArrowUp';
    if (arrow && list.hidden) showSuggestions(input);
    if (list.hidden) return;
    const activeId = input.getAttribute('aria-activedescendant');
    const options = [...list.querySelectorAll<HTMLElement>('[role="option"]')];
    const index = options.findIndex(option => option.id === activeId);

    if (arrow && options.length) {
      event.preventDefault();
      const next = event.key === 'ArrowDown' ? (index + 1) % options.length
        : index <= 0 ? options.length - 1 : index - 1;
      options[index]?.setAttribute('aria-selected', 'false');
      options[next].setAttribute('aria-selected', 'true');
      input.setAttribute('aria-activedescendant', options[next].id);
      options[next].scrollIntoView({ block: 'nearest' });
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (index >= 0) selectOption(input, options[index]);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      closeCapabilitySuggestions(input);
    } else if (event.key === 'Tab') {
      closeCapabilitySuggestions(input);
    }
  });
}
