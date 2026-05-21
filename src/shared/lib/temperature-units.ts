export const TEMPERATURE_UNITS = ["celsius", "fahrenheit"] as const;

export type TemperatureUnit = (typeof TEMPERATURE_UNITS)[number];

export function normalizeTemperatureUnit(value: unknown): TemperatureUnit {
  return TEMPERATURE_UNITS.includes(value as TemperatureUnit) ? (value as TemperatureUnit) : "celsius";
}
