export function getFiniteNumberInputValue(
  value: number,
  fallback: number,
  options: { min?: number; max?: number } = {},
) {
  const finiteValue = Number.isFinite(value) ? value : fallback;
  const minBounded = options.min === undefined ? finiteValue : Math.max(options.min, finiteValue);
  return options.max === undefined ? minBounded : Math.min(options.max, minBounded);
}
