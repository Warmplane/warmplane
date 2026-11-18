// Escapes a value for safe interpolation into HTML text or an attribute.

/**
 * Escapes a value for safe interpolation into HTML text or an attribute.
 *
 * HTML escaping does not make a value safe inside JavaScript source. Dynamic
 * values used by controls should be stored in escaped data attributes and read
 * by event handlers instead of interpolated into inline `on*` handlers.
 *
 * @param value Any value. Non-strings are coerced; nullish values become an
 *   empty string.
 * @returns The escaped string.
 */
export function escapeHtml(value: unknown): string {
  if (value === null || value === undefined) return '';
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
