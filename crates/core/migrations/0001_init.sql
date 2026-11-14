-- crates/core/migrations/0001_init.sql
PRAGMA foreign_keys = ON;

CREATE TABLE lists (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  color         TEXT NOT NULL DEFAULT '#7c83ff',
  sort_order    INTEGER NOT NULL DEFAULT 0,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tasks (
  id            TEXT PRIMARY KEY,
  list_id       TEXT REFERENCES lists(id) ON DELETE SET NULL,
  title         TEXT NOT NULL,
  notes         TEXT NOT NULL DEFAULT '',
  due_at        TEXT,
  due_tz        TEXT,
  all_day       INTEGER NOT NULL DEFAULT 1,
  rrule         TEXT,
  priority      INTEGER NOT NULL DEFAULT 0,
  countdown     INTEGER NOT NULL DEFAULT 0,
  completed_at  TEXT,
  sort_order    INTEGER NOT NULL DEFAULT 0,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_tasks_due  ON tasks(due_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_list ON tasks(list_id) WHERE deleted_at IS NULL;

CREATE TABLE notes (
  id            TEXT PRIMARY KEY,
  title         TEXT NOT NULL DEFAULT '',
  body          TEXT NOT NULL DEFAULT '',
  pinned        INTEGER NOT NULL DEFAULT 0,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE habits (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  emoji         TEXT NOT NULL DEFAULT '✨',
  color         TEXT NOT NULL DEFAULT '#38e1ff',
  weekdays_mask INTEGER NOT NULL DEFAULT 127,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE habit_entries (
  habit_id      TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
  local_date    TEXT NOT NULL,
  value         INTEGER NOT NULL DEFAULT 1,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (habit_id, local_date)
);

CREATE TABLE task_completions (
  task_id        TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  occurrence_key TEXT NOT NULL,
  completed_at   TEXT NOT NULL,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  deleted_at     TEXT,
  origin_device  TEXT,
  rev            INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (task_id, occurrence_key)
);

CREATE TABLE task_overrides (
  task_id        TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  occurrence_key TEXT NOT NULL,
  override_json  TEXT NOT NULL,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  deleted_at     TEXT,
  origin_device  TEXT,
  rev            INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (task_id, occurrence_key)
);

CREATE TABLE reminders (
  id             TEXT PRIMARY KEY,
  task_id        TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  occurrence_key TEXT NOT NULL DEFAULT 'base',
  fire_at        TEXT NOT NULL,
  fired_at       TEXT,
  cancelled_at   TEXT,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  deleted_at     TEXT,
  origin_device  TEXT,
  rev            INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_reminders_fire ON reminders(fire_at) WHERE fired_at IS NULL AND cancelled_at IS NULL;

CREATE TABLE pomodoro_sessions (
  id            TEXT PRIMARY KEY,
  task_id       TEXT REFERENCES tasks(id) ON DELETE SET NULL,
  started_at    TEXT NOT NULL,
  ended_at      TEXT,
  duration_sec  INTEGER NOT NULL,
  kind          TEXT NOT NULL DEFAULT 'focus',
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT,
  origin_device TEXT,
  rev           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE settings_synced (
  key TEXT PRIMARY KEY, value TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT,
  origin_device TEXT, rev INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE settings_local (
  key TEXT PRIMARY KEY, value TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);