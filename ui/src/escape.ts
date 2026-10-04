// Escapes a value for safe interpolation into HTML text or an attribute.

/**
 * Escapes a value for safe interpolation into HTML text or an attribute.
 *
 * Also escapes the single quote, so the value is safe inside the single-quoted
 * JavaScript string literals used by inline `on*` handlers.
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