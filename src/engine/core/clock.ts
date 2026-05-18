export interface Clock {
  now(): Date;
  nowIso(): string;
  nowMs(): number;
}

export const systemClock: Clock = {
  now: () => new Date(),
  nowIso: () => new Date().toISOString(),
  nowMs: () => Date.now(),
};

export function createFixedClock(date: Date | string | number): Clock {
  const fixed = new Date(date);
  return {
    now: () => new Date(fixed),
    nowIso: () => fixed.toISOString(),
    nowMs: () => fixed.getTime(),
  };
}
