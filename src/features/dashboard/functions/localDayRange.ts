export type LocalDayRange = { dayStartMs: number; dayEndMs: number };

export function localDayRange(now: Date = new Date()): LocalDayRange {
  const year = now.getFullYear();
  const month = now.getMonth();
  const day = now.getDate();
  return {
    dayStartMs: new Date(year, month, day).getTime(),
    dayEndMs: new Date(year, month, day + 1).getTime(),
  };
}
