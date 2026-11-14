import { listen } from '@tauri-apps/api/event';
import { api, isTauri } from './api';
import { todayKey } from './dates';
import type { Bootstrap, CalendarItem, Filter, Reminder, Sort, WidgetState } from './types';

class Store {
  ready = $state(false);
  error = $state<string | null>(null);
  lists = $state<Bootstrap['lists']>([]);
  tasks = $state<Bootstrap['tasks']>([]);
  notes = $state<Bootstrap['notes']>([]);
  habits = $state<Bootstrap['habits']>([]);
  habitEntries = $state<Bootstrap['habitEntries']>([]);
  completions = $state<Bootstrap['completions']>([]);
  pomodoros = $state<Bootstrap['pomodoros']>([]);
  calendar = $state<CalendarItem[]>([]);
  widgets = $state<WidgetState[]>([]);

  query = $state('');
  filter = $state<Filter>('all');
  sort = $state<Sort>('manual');
  listFilter = $state<string | null>(null);
  selection = $state<string[]>([]);
  editingTaskId = $state<string | null>(null);
  editingNoteId = $state<string | null>(null);
  toast = $state<{ msg: string; undo?: () => void } | null>(null);

  countdowns = $derived(
    this.tasks
      .filter((t) => t.countdown && !t.completedAt && t.dueAt)
      .sort((a, b) => (a.dueAt ?? '').localeCompare(b.dueAt ?? ''))
  );

  #timer: ReturnType<typeof setTimeout> | undefined;
  #toastTimer: ReturnType<typeof setTimeout> | undefined;
  #started = false;

  async init() {
    if (this.#started) return;
    this.#started = true;
    await this.refresh();
    await this.refreshWidgets();
    if (isTauri) {
      await listen('tempo:changed', () => this.scheduleRefresh());
      await listen('tempo:widgets', () => this.refreshWidgets());
      await listen<Reminder>('tempo:reminder', (e) => {
        this.notify(`🔔 Reminder: ${e.payload.title}`);
      });
    }
  }

  scheduleRefresh() {
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => this.refresh(), 60);
  }

  async refresh() {
    try {
      const b = await api.bootstrap();
      this.lists = b.lists; this.tasks = b.tasks; this.notes = b.notes;
      this.habits = b.habits; this.habitEntries = b.habitEntries;
      this.completions = b.completions; this.pomodoros = b.pomodoros;
      this.ready = true; this.error = null;
    } catch (err) {
      this.error = String(err);
      this.ready = true;
    }
  }

  async loadCalendar(from: Date, to: Date) {
    try { this.calendar = await api.calendar(from.toISOString(), to.toISOString()); }
    catch (err) { this.error = String(err); }
  }

  async refreshWidgets() {
    if (!isTauri) return;
    try { this.widgets = await api.widgets(); } catch { /* desktop only */ }
  }

  notify(msg: string, undo?: () => void) {
    clearTimeout(this.#toastTimer);
    this.toast = { msg, undo };
    this.#toastTimer = setTimeout(() => (this.toast = null), 6000);
  }

  async toggleTask(id: string, occurrenceKey: string | null = null) {
    const t = this.tasks.find((x) => x.id === id);
    const prev = t?.completedAt ?? null;
    if (t && !occurrenceKey && !t.rrule) t.completedAt = prev ? null : new Date().toISOString();
    try { await api.toggleTask(id, occurrenceKey); }
    catch (err) { if (t) t.completedAt = prev; this.notify(`Could not save: ${err}`); }
  }

  async saveTask(input: Record<string, unknown>) {
    const t = await api.saveTask(input);
    await this.refresh();
    return t;
  }

  async quickAdd(title: string, dueAt: string | null, listId: string | null = null, countdown = false) {
    if (!title.trim()) return;
    await api.saveTask({
      title: title.trim(),
      dueAt,
      listId,
      allDay: true,
      countdown,
      dueTz: Intl.DateTimeFormat().resolvedOptions().timeZone
    });
    await this.refresh();
  }

  async deleteTask(id: string) {
    this.tasks = this.tasks.filter((t) => t.id !== id);
    if (this.editingTaskId === id) this.editingTaskId = null;
    await api.deleteTask(id);
    this.notify('Task deleted', async () => { await api.restore('tasks', [id]); await this.refresh(); });
  }

  async bulkDelete() {
    const ids = [...this.selection];
    this.selection = [];
    for (const id of ids) await api.deleteTask(id);
    await this.refresh();
    this.notify(`${ids.length} tasks deleted`, async () => { await api.restore('tasks', ids); await this.refresh(); });
  }

  async bulkComplete() {
    const ids = [...this.selection];
    this.selection = [];
    await api.bulkComplete(ids, true);
    await this.refresh();
    this.notify(`${ids.length} completed`, async () => { await api.bulkComplete(ids, false); await this.refresh(); });
  }

  async reorder(ids: string[]) { await api.reorder(ids); await this.refresh(); }

  async toggleHabit(habitId: string, localDate = todayKey()) {
    const has = this.habitEntries.some((e) => e.habitId === habitId && e.localDate === localDate);
    this.habitEntries = has
      ? this.habitEntries.filter((e) => !(e.habitId === habitId && e.localDate === localDate))
      : [...this.habitEntries, { habitId, localDate, value: 1 }];
    try { await api.toggleHabit(habitId, localDate); } catch { await this.refresh(); }
  }

  async deleteHabit(id: string) {
    await api.deleteHabit(id);
    await this.refresh();
    this.notify('Habit deleted', async () => { await api.restore('habits', [id]); await this.refresh(); });
  }

  async deleteNote(id: string) {
    this.notes = this.notes.filter((n) => n.id !== id);
    if (this.editingNoteId === id) this.editingNoteId = null;
    await api.deleteNote(id);
    this.notify('Note deleted', async () => { await api.restore('notes', [id]); await this.refresh(); });
  }

  async deleteList(id: string) {
    await api.deleteList(id);
    if (this.listFilter === id) this.listFilter = null;
    await this.refresh();
    this.notify('List removed · tasks kept in Inbox', async () => { await api.restore('lists', [id]); await this.refresh(); });
  }
}

export const store = new Store();