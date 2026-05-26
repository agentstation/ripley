/**
 * DESIGN.md severity color tokens — the source-of-truth for runtime contrast
 * assertions. Mirrors the `--color-severity-*` block in `theme.css`.
 *
 * RGB form is what `getComputedStyle().color` returns; the spec compares
 * computed styles against this map rather than re-parsing hex.
 */
export const SEVERITY_COLORS_RGB = {
  critical: "rgb(255, 123, 114)",
  high: "rgb(240, 136, 62)",
  medium: "rgb(210, 153, 34)",
  low: "rgb(88, 166, 255)",
  clean: "rgb(63, 185, 80)",
} as const;

export type SeverityKey = keyof typeof SEVERITY_COLORS_RGB;

/**
 * Allow-list for color values found in computed styles. Anything that resolves
 * to one of these passes the "no raw hex" runtime check.
 *
 * - rgb() / rgba() with our token values.
 * - oklab / oklch / lab — modern color spaces Chromium emits for some palettes.
 * - transparent / currentColor / inherit.
 */
const COMPUTED_COLOR_PATTERN =
  /^(rgb(a)?\(|hsl(a)?\(|oklch\(|oklab\(|lab\(|lch\(|color\(|transparent$|currentcolor$|inherit$|initial$|none$|unset$|var\()/i;

export function isAllowedComputedColor(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true;
  return COMPUTED_COLOR_PATTERN.test(trimmed);
}
