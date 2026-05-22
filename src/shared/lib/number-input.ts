export function getFiniteNumberInputValue(
  value: number,
  fallback: number,
  options: { min?: number; max?: number } = {},
) {
  const finiteFallback = Number.isFinite(fallback) ? fallback : (options.min ?? 0);
  const finiteValue = Number.isFinite(value) ? value : finiteFallback;
  const minBounded = options.min === undefined ? finiteValue : Math.max(options.min, finiteValue);
  return options.max === undefined ? minBounded : Math.min(options.max, minBounded);
}
