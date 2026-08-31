export function formatFilsForInput(fils: number): string {
  const whole = Math.floor(fils / 1000);
  const remainder = fils % 1000;

  return `${whole}.${remainder.toString().padStart(3, "0")}`;
}