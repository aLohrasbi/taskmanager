// src/lib/types.ts
export interface List { id: string; name: string; color: string; sortOrder: number; createdAt: string; updatedAt: string }
export interface Task {
  id: string; listId: string | null; title: string; notes: string;
  dueAt: string | null; dueTz: string | null; allDay: boolean; rrule: string | null;
  priority: number; countdown: boolean; completedAt: string | null; sortOrder: number; createdAt: string; updatedAt: string;
}
export interface Note { id: string; title: string; body: string; pinned: boolean; createdAt: string; updatedAt: string }
export interface Habit { id: string; name: string; emoji: string; color: string; weekdaysMask: number; createdAt: string; updatedAt: string }
export interface HabitEntry { habitId: string; localDate: string; value: number }
export interface Completion { taskId: string; occurrenceKey: string; completedAt: string }
export interface PomodoroSession { id: string; taskId: string | null; startedAt: string; endedAt: string; durationSec: number; kind: string }
export interface Reminder { id: string; taskId: string; title: string; fireAt: string }
export interface Bootstrap {
  lists: List[]; tasks: Task[]; notes: Note[]; habits: Habit[];
  habitEntries: HabitEntry[]; completions: Completion[]; pomodoros: PomodoroSession[];
}
export interface CalendarItem {
  taskId: string; occurrenceKey: string; title: string; startAt: string;
  allDay: boolean; priority: number; countdown: boolean; listId: string | null; completed: boolean; recurring: boolean;
}
export interface WidgetState { label: string; visible: boolean; pin: string }
export type Filter = 'all' | 'today' | 'next7' | 'overdue' | 'countdown' | 'done';
export type Sort = 'manual' | 'due' | 'priority' | 'created';