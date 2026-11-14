import { invoke } from '@tauri-apps/api/core';
import type { Bootstrap, CalendarItem, Habit, List, Note, PomodoroSession, Task, WidgetState } from './types';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const api = {
  bootstrap: () => invoke<Bootstrap>('get_bootstrap'),
  calendar: (from: string, to: string) => invoke<CalendarItem[]>('get_calendar_range', { from, to }),
  saveTask: (input: Record<string, unknown>) => invoke<Task>('save_task', { input }),
  toggleTask: (id: string, occurrenceKey: string | null = null) => invoke<void>('toggle_task', { id, occurrenceKey }),
  deleteTask: (id: string) => invoke<void>('delete_task', { id }),
  reorder: (ids: string[]) => invoke<void>('reorder_tasks', { ids }),
  bulkComplete: (ids: string[], done: boolean) => invoke<void>('bulk_complete', { ids, done }),
  restore: (kind: string, ids: string[]) => invoke<void>('restore_entities', { kind, ids }),
  logPomodoro: (taskId: string | null, durationSec: number, kind: string) =>
    invoke<PomodoroSession>('log_pomodoro', { taskId, durationSec, kind }),
  saveList: (input: Record<string, unknown>) => invoke<List>('save_list', { input }),
  deleteList: (id: string) => invoke<void>('delete_list', { id }),
  saveNote: (input: Record<string, unknown>) => invoke<Note>('save_note', { input }),
  deleteNote: (id: string) => invoke<void>('delete_note', { id }),
  saveHabit: (input: Record<string, unknown>) => invoke<Habit>('save_habit', { input }),
  deleteHabit: (id: string) => invoke<void>('delete_habit', { id }),
  toggleHabit: (habitId: string, localDate: string) => invoke<void>('toggle_habit', { habitId, localDate }),
  widgets: () => invoke<WidgetState[]>('widget_states'),
  toggleWidget: (label: string) => invoke<boolean>('toggle_widget', { label }),
  hideWidget: (label: string) => invoke<void>('hide_widget', { label }),
  setPin: (label: string, mode: string) => invoke<void>('set_widget_pin', { label, mode }),
  focusMain: () => invoke<void>('focus_main')
};