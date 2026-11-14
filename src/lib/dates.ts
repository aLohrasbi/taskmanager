// src/lib/dates.ts
export const dateKey = (d: Date) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
export const todayKey = () => dateKey(new Date());
export const addDays = (d: Date, n: number) => { const x = new Date(d); x.setDate(x.getDate() + n); return x; };
export const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
export const startOfWeek = (d: Date) => { const x = startOfDay(d); const wd = (x.getDay() + 6) % 7; return addDays(x, -wd); };
export const iso = (d: Date) => d.toISOString();

export function isOverdue(dueAt: string | null, allDay: boolean, done: boolean) {
  if (!dueAt || done) return false;
  const d = new Date(dueAt);
  return allDay ? dateKey(d) < todayKey() : d.getTime() < Date.now();
}
export function isToday(dueAt: string | null) { return !!dueAt && dateKey(new Date(dueAt)) === todayKey(); }
export function isNext7(dueAt: string | null) {
  if (!dueAt) return false;
  const k = dateKey(new Date(dueAt));
  return k >= todayKey() && k <= dateKey(addDays(new Date(), 6));
}
export function daysLeft(dueAt: string | null): number | null {
  if (!dueAt) return null;
  return Math.round((startOfDay(new Date(dueAt)).getTime() - startOfDay(new Date()).getTime()) / 86400000);
}
export function fmtCountdown(dueAt: string | null) {
  const d = daysLeft(dueAt);
  if (d === null) return '';
  if (d === 0) return 'Today';
  if (d === 1) return '1 day left';
  if (d > 1) return `${d} days left`;
  return `${Math.abs(d)} days Overdue`;
}
export function fmtDue(dueAt: string | null, allDay: boolean) {
  if (!dueAt) return '';
  const d = new Date(dueAt);
  const k = dateKey(d);
  const t = todayKey();
  let label: string;
  if (k === t) label = 'Today';
  else if (k === dateKey(addDays(new Date(), 1))) label = 'Tomorrow';
  else if (k === dateKey(addDays(new Date(), -1))) label = 'Yesterday';
  else label = d.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' });
  return allDay ? label : `${label} · ${d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}`;
}
export function toLocalInput(isoStr: string | null) {
  if (!isoStr) return '';
  const d = new Date(isoStr);
  return `${dateKey(d)}T${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}
export function streak(dates: string[], today = todayKey()) {
  const set = new Set(dates);
  if (!set.size) return { current: 0, best: 0, doneToday: false };
  const doneToday = set.has(today);
  const shift = (k: string, n: number) => dateKey(addDays(new Date(k + 'T12:00:00'), n));
  let anchor = doneToday ? today : shift(today, -1);
  let current = 0;
  while (set.has(anchor)) { current++; anchor = shift(anchor, -1); }
  const sorted = [...set].sort();
  let best = 1, run = 1;
  for (let i = 1; i < sorted.length; i++) {
    run = shift(sorted[i - 1], 1) === sorted[i] ? run + 1 : 1;
    best = Math.max(best, run);
  }
  return { current, best, doneToday };
}