<!-- src/windows/MainWindow.svelte -->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { api } from '../lib/api';
  import { dateKey, fmtCountdown, fmtDue, isNext7, isOverdue, isToday, startOfDay, startOfWeek, streak, todayKey, addDays, toLocalInput } from '../lib/dates';
  import type { Task } from '../lib/types';

  let tab = $state<'tasks' | 'calendar' | 'pomodoro' | 'habits' | 'notes'>('tasks');
  let newTitle = $state('');
  let newDue = $state('');
  let newList = $state('');
  let habitName = $state('');
  let habitEmoji = $state('✨');
  let dragId = $state<string | null>(null);

  // Calendar state
  let calOffset = $state(0);
  const calDays = $derived(Array.from({ length: 7 }, (_, i) => addDays(startOfWeek(addDays(new Date(), calOffset * 7)), i)));
  $effect(() => {
    void store.tasks;
    void store.completions;
    const from = startOfDay(calDays[0]);
    const to = new Date(startOfDay(calDays[6]).getTime() + 86399000);
    store.loadCalendar(from, to);
  });

  // Pomodoro state machine
  let pomoKind = $state<'focus' | 'short' | 'long'>('focus');
  const pomoDurations = { focus: 25 * 60, short: 5 * 60, long: 15 * 60 };
  let pomoLeft = $state(25 * 60);
  let pomoRunning = $state(false);
  let pomoTaskId = $state<string>('');

  $effect(() => {
    if (!pomoRunning) return;
    const id = setInterval(() => {
      if (pomoLeft > 1) {
        pomoLeft -= 1;
      } else {
        pomoRunning = false;
        const dur = pomoDurations[pomoKind];
        api.logPomodoro(pomoTaskId || null, dur, pomoKind).then(() => store.refresh());
        store.notify(`🍅 ${pomoKind === 'focus' ? 'Focus session' : 'Break'} complete!`);
        pomoLeft = dur;
      }
    }, 1000);
    return () => clearInterval(id);
  });

  function setPomoMode(k: 'focus' | 'short' | 'long') {
    pomoRunning = false;
    pomoKind = k;
    pomoLeft = pomoDurations[k];
  }

  const todayFocusMin = $derived(
    Math.round(
      store.pomodoros
        .filter((p) => p.kind === 'focus' && dateKey(new Date(p.startedAt)) === todayKey())
        .reduce((acc, p) => acc + p.durationSec, 0) / 60
    )
  );

  const palette = ['#7c83ff', '#38e1ff', '#7ef0b2', '#ffb454', '#ff6ac1', '#a78bfa'];
  const emojis = ['✨', '📚', '🏃', '🧘', '💧', '🌙', '🎸', '🧠'];
  const rrules = [
    ['', 'Does not repeat'],
    ['FREQ=DAILY', 'Daily'],
    ['FREQ=DAILY;INTERVAL=2', 'Every 2 days'],
    ['FREQ=WEEKLY', 'Weekly'],
    ['FREQ=MONTHLY', 'Monthly'],
    ['FREQ=YEARLY', 'Yearly']
  ];

  const visibleTasks = $derived.by(() => {
    const q = store.query.trim().toLowerCase();
    let out = store.tasks.filter((t) => {
      if (store.listFilter && t.listId !== store.listFilter) return false;
      if (q && !(`${t.title} ${t.notes}`.toLowerCase().includes(q))) return false;
      const done = !!t.completedAt;
      switch (store.filter) {
        case 'today': return !done && isToday(t.dueAt);
        case 'next7': return !done && isNext7(t.dueAt);
        case 'overdue': return isOverdue(t.dueAt, t.allDay, done);
        case 'countdown': return !done && t.countdown;
        case 'done': return done;
        default: return true;
      }
    });
    const byDue = (a: Task, b: Task) => (a.dueAt ?? '9999').localeCompare(b.dueAt ?? '9999');
    if (store.sort === 'due' || store.filter === 'countdown') out = [...out].sort(byDue);
    else if (store.sort === 'priority') out = [...out].sort((a, b) => b.priority - a.priority || byDue(a, b));
    else if (store.sort === 'created') out = [...out].sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    return out;
  });

  const editing = $derived(store.tasks.find((t) => t.id === store.editingTaskId) ?? null);
  const editingNote = $derived(store.notes.find((n) => n.id === store.editingNoteId) ?? null);
  const visibleNotes = $derived.by(() => {
    const q = store.query.trim().toLowerCase();
    return q ? store.notes.filter((n) => `${n.title} ${n.body}`.toLowerCase().includes(q)) : store.notes;
  });
  const week = $derived(Array.from({ length: 7 }, (_, i) => dateKey(addDays(new Date(), i - 6))));

  let draft = $state({ title: '', notes: '', due: '', remind: '', rrule: '', listId: '', priority: 0, countdown: false });
  let lastId: string | null = null;
  $effect(() => {
    if (editing && editing.id !== lastId) {
      lastId = editing.id;
      draft = {
        title: editing.title,
        notes: editing.notes,
        due: toLocalInput(editing.dueAt),
        remind: '',
        rrule: editing.rrule ?? '',
        listId: editing.listId ?? '',
        priority: editing.priority,
        countdown: editing.countdown
      };
    }
    if (!editing) lastId = null;
  });

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  function queueSave() {
    if (!editing) return;
    clearTimeout(saveTimer);
    const id = editing.id;
    const payload = {
      id, title: draft.title, notes: draft.notes,
      dueAt: draft.due ? new Date(draft.due).toISOString() : null,
      remindAt: draft.remind ? new Date(draft.remind).toISOString() : null,
      dueTz: Intl.DateTimeFormat().resolvedOptions().timeZone,
      allDay: draft.due ? false : editing.allDay,
      rrule: draft.rrule || null,
      countdown: draft.countdown,
      listId: draft.listId || null, priority: draft.priority
    };
    saveTimer = setTimeout(() => store.saveTask(payload).catch((e) => store.notify(String(e))), 500);
  }

  let noteDraft = $state({ title: '', body: '' });
  let lastNoteId: string | null = null;
  $effect(() => {
    if (editingNote && editingNote.id !== lastNoteId) {
      lastNoteId = editingNote.id;
      noteDraft = { title: editingNote.title, body: editingNote.body };
    }
    if (!editingNote) lastNoteId = null;
  });
  let noteTimer: ReturnType<typeof setTimeout> | undefined;
  function queueNoteSave() {
    if (!editingNote) return;
    clearTimeout(noteTimer);
    const payload = { id: editingNote.id, title: noteDraft.title, body: noteDraft.body, pinned: editingNote.pinned };
    noteTimer = setTimeout(() => api.saveNote(payload).catch((e) => store.notify(String(e))), 500);
  }

  async function addTask() {
    if (!newTitle.trim()) return;
    const isCd = store.filter === 'countdown';
    const due = newDue
      ? new Date(newDue + 'T09:00:00').toISOString()
      : isCd
        ? addDays(new Date(), 30).toISOString()
        : null;
    await store.quickAdd(newTitle, due, store.listFilter, isCd);
    newTitle = '';
    newDue = '';
  }
  async function addList() {
    if (!newList.trim()) return;
    await api.saveList({ name: newList.trim(), color: palette[store.lists.length % palette.length] });
    newList = '';
    await store.refresh();
  }
  async function addHabit() {
    if (!habitName.trim()) return;
    await api.saveHabit({ name: habitName.trim(), emoji: habitEmoji, color: palette[store.habits.length % palette.length] });
    habitName = ''; habitEmoji = '✨';
    await store.refresh();
  }
  async function newNote() {
    const n = await api.saveNote({ title: 'Untitled note', body: '', pinned: false });
    await store.refresh();
    store.editingNoteId = n.id;
  }
  function onDrop(targetId: string) {
    if (!dragId || dragId === targetId) return;
    const ids = visibleTasks.map((t) => t.id);
    const from = ids.indexOf(dragId);
    const to = ids.indexOf(targetId);
    ids.splice(to, 0, ids.splice(from, 1)[0]);
    dragId = null;
    store.reorder(ids);
  }
  const listColor = (id: string | null) => store.lists.find((l) => l.id === id)?.color ?? '#6b7494';
  const listName = (id: string | null) => store.lists.find((l) => l.id === id)?.name ?? 'Inbox';
</script>

<div class="shell">
  <aside class="rail">
    <div class="brand"><span class="mark"></span> Tempo</div>

    <nav>
      <button class:active={tab === 'tasks'} onclick={() => (tab = 'tasks')}>Tasks <em>{store.tasks.filter((t) => !t.completedAt).length}</em></button>
      <button class:active={tab === 'calendar'} onclick={() => (tab = 'calendar')}>Calendar</button>
      <button class:active={tab === 'pomodoro'} onclick={() => (tab = 'pomodoro')}>Pomodoro <em>{todayFocusMin}m</em></button>
      <button class:active={tab === 'habits'} onclick={() => (tab = 'habits')}>Habits <em>{store.habits.length}</em></button>
      <button class:active={tab === 'notes'} onclick={() => (tab = 'notes')}>Notes <em>{store.notes.length}</em></button>
    </nav>

    <div class="rail-title">Lists</div>
    <button class="list-row" class:active={store.listFilter === null} onclick={() => (store.listFilter = null)}>
      <i style="background:#6b7494"></i> All lists
    </button>
    {#each store.lists as l (l.id)}
      <button class="list-row" class:active={store.listFilter === l.id} onclick={() => (store.listFilter = l.id)}>
        <i style="background:{l.color}"></i> {l.name}
        <em>{store.tasks.filter((t) => t.listId === l.id && !t.completedAt).length}</em>
      </button>
    {/each}
    <input class="mini" placeholder="+ New list" bind:value={newList} onkeydown={(e) => e.key === 'Enter' && addList()} />

    <div class="rail-title">Desktop panels</div>
    {#each store.widgets as w (w.label)}
      <button class="list-row" onclick={() => api.toggleWidget(w.label)}>
        <i style="background:{w.visible ? '#7ef0b2' : '#6b7494'}"></i>
        {w.label === 'week' ? 'Week calendar' : 'Next 7 days'}
        <em>{w.visible ? 'on' : 'off'}</em>
      </button>
    {/each}
    <p class="foot">Saved locally · SQLite WAL</p>
  </aside>

  <main class="stage">
    <header class="topbar">
      <input class="search" placeholder="Search…" bind:value={store.query} />
      <span class="badge"><span class="dot"></span> Local</span>
    </header>

    {#if store.error}<div class="error">{store.error}</div>{/if}

    {#if tab === 'tasks'}
      <div class="controls">
        {#each [['all', 'All'], ['today', 'Today'], ['next7', 'Next 7'], ['overdue', 'Overdue'], ['countdown', 'Countdown'], ['done', 'Done']] as [key, label]}
          <button class="chip" class:on={store.filter === key} onclick={() => (store.filter = key as any)}>{label}</button>
        {/each}
        <select bind:value={store.sort}>
          <option value="manual">Manual</option><option value="due">Due date</option>
          <option value="priority">Priority</option><option value="created">Newest</option>
        </select>
      </div>

      <div class="add">
        <input placeholder={store.filter === 'countdown' ? 'Add a countdown event…' : 'Add a task and press Enter…'} bind:value={newTitle} onkeydown={(e) => e.key === 'Enter' && addTask()} />
        <input type="date" class="date-pick" bind:value={newDue} title="Target date" />
        <button class="primary" onclick={addTask}>Add</button>
      </div>

      {#if store.selection.length}
        <div class="bulk">
          {store.selection.length} selected
          <button onclick={() => store.bulkComplete()}>Complete</button>
          <button class="danger" onclick={() => store.bulkDelete()}>Delete</button>
          <button onclick={() => (store.selection = [])}>Clear</button>
        </div>
      {/if}

      <div class="rows">
        {#each visibleTasks as t, i (t.id)}
          {@const done = !!t.completedAt}
          {@const over = isOverdue(t.dueAt, t.allDay, done)}
          <div class="row" class:done class:over style="--i:{i}"
               ondragover={(e) => e.preventDefault()} ondrop={() => onDrop(t.id)}>
            <span class="grip" draggable="true" ondragstart={() => (dragId = t.id)}>⠿</span>
            <button class="check" class:on={done} onclick={() => store.toggleTask(t.id)} aria-label="Toggle task">
              <svg viewBox="0 0 24 24"><path d="M5 13l4 4L19 7" /></svg>
            </button>
            <button class="row-body" onclick={() => (store.editingTaskId = t.id)}>
              <span class="title">{t.title}</span>
              <span class="meta">
                <i style="background:{listColor(t.listId)}"></i>{listName(t.listId)}
                {#if t.dueAt}<span class="due" class:over>{fmtDue(t.dueAt, t.allDay)}</span>{/if}
                {#if t.countdown && t.dueAt}<span class="cd-tag">{fmtCountdown(t.dueAt)}</span>{/if}
                {#if t.rrule}<span class="tag">↻ {t.rrule.replace('FREQ=', '').toLowerCase()}</span>{/if}
              </span>
            </button>
            {#if t.priority > 0}<span class="prio p{t.priority}">⚑</span>{/if}
            <input type="checkbox" class="sel" checked={store.selection.includes(t.id)}
              onchange={(e) => (store.selection = e.currentTarget.checked ? [...store.selection, t.id] : store.selection.filter((x) => x !== t.id))} />
          </div>
        {:else}
          <div class="empty">Nothing here. Add a task above, or switch filters.</div>
        {/each}
      </div>

    {:else if tab === 'calendar'}
      <div class="controls">
        <button class="chip" onclick={() => (calOffset -= 1)}>‹ Prev</button>
        <button class="chip on" onclick={() => (calOffset = 0)}>This Week</button>
        <button class="chip" onclick={() => (calOffset += 1)}>Next ›</button>
      </div>
      <div class="cal-board">
        {#each calDays as d (d.toISOString())}
          {@const items = store.calendar.filter((c) => dateKey(new Date(c.startAt)) === dateKey(d))}
          <div class="cal-col" class:today={dateKey(d) === todayKey()}>
            <div class="cal-head">
              <span>{d.toLocaleDateString(undefined, { weekday: 'short' })}</span>
              <strong>{d.getDate()}</strong>
            </div>
            <div class="cal-list">
              {#each items as it (it.taskId + it.occurrenceKey)}
                <button class="cal-item" class:done={it.completed} style="--c:{listColor(it.listId)}"
                        onclick={() => store.toggleTask(it.taskId, it.occurrenceKey)}>
                  <span>{it.title}</span>
                  {#if it.recurring}<small>↻</small>{/if}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>

    {:else if tab === 'pomodoro'}
      <div class="pomo-wrap">
        <div class="pomo-card">
          <div class="pomo-modes">
            <button class="chip" class:on={pomoKind === 'focus'} onclick={() => setPomoMode('focus')}>Focus 25m</button>
            <button class="chip" class:on={pomoKind === 'short'} onclick={() => setPomoMode('short')}>Short Break 5m</button>
            <button class="chip" class:on={pomoKind === 'long'} onclick={() => setPomoMode('long')}>Long Break 15m</button>
          </div>
          <div class="pomo-clock">
            {String(Math.floor(pomoLeft / 60)).padStart(2, '0')}:{String(pomoLeft % 60).padStart(2, '0')}
          </div>
          <select bind:value={pomoTaskId}>
            <option value="">No linked task</option>
            {#each store.tasks.filter((t) => !t.completedAt) as t}<option value={t.id}>{t.title}</option>{/each}
          </select>
          <div class="pomo-actions">
            <button class="primary" onclick={() => (pomoRunning = !pomoRunning)}>{pomoRunning ? 'Pause' : 'Start'}</button>
            <button class="chip" onclick={() => setPomoMode(pomoKind)}>Reset</button>
          </div>
          <small>Today's focus: {todayFocusMin} min · {store.pomodoros.length} recent sessions</small>
        </div>
      </div>

    {:else if tab === 'notes'}
      <div class="controls"><button class="primary" onclick={newNote}>+ New note</button></div>
      <div class="notes">
        {#each visibleNotes as n (n.id)}
          <button class="note" onclick={() => (store.editingNoteId = n.id)}>
            <strong>{n.title || 'Untitled'}</strong>
            <p>{n.body.slice(0, 180)}</p>
            <span>{new Date(n.updatedAt).toLocaleDateString()}</span>
          </button>
        {:else}
          <div class="empty">No notes yet.</div>
        {/each}
      </div>

    {:else}
      <div class="add">
        <select bind:value={habitEmoji}>{#each emojis as e}<option value={e}>{e}</option>{/each}</select>
        <input placeholder="New habit…" bind:value={habitName} onkeydown={(e) => e.key === 'Enter' && addHabit()} />
        <button class="primary" onclick={addHabit}>Add</button>
      </div>
      <div class="habits">
        {#each store.habits as h (h.id)}
          {@const dates = store.habitEntries.filter((e) => e.habitId === h.id).map((e) => e.localDate)}
          {@const s = streak(dates)}
          <div class="habit">
            <div class="habit-head">
              <span class="emoji" style="background:{h.color}22">{h.emoji}</span>
              <div><strong>{h.name}</strong><small>🔥 {s.current} · best {s.best}</small></div>
              <button class="x" onclick={() => store.deleteHabit(h.id)}>×</button>
            </div>
            <div class="strip">
              {#each week as d}
                {@const on = dates.includes(d)}
                <button class="pip" class:on class:today={d === todayKey()} style="--c:{h.color}"
                        onclick={() => store.toggleHabit(h.id, d)} title={d}>
                  {new Date(d + 'T12:00:00').toLocaleDateString(undefined, { weekday: 'narrow' })}
                </button>
              {/each}
            </div>
          </div>
        {:else}
          <div class="empty">No habits yet.</div>
        {/each}
      </div>
    {/if}
  </main>

  {#if editing}
    <aside class="drawer">
      <header><strong>Task</strong><button class="x" onclick={() => (store.editingTaskId = null)}>×</button></header>
      <input class="big" bind:value={draft.title} oninput={queueSave} />
      <label>List
        <select bind:value={draft.listId} onchange={queueSave}>
          <option value="">Inbox</option>
          {#each store.lists as l}<option value={l.id}>{l.name}</option>{/each}
        </select>
      </label>
      <label>Due
        <input type="datetime-local" bind:value={draft.due} onchange={queueSave} />
      </label>
      <label>Repeat (RRULE)
        <select bind:value={draft.rrule} onchange={queueSave}>
          {#each rrules as [val, label]}<option value={val}>{label}</option>{/each}
        </select>
      </label>
      <label>Reminder alert
        <input type="datetime-local" bind:value={draft.remind} onchange={queueSave} />
      </label>
      <label class="inline-toggle">
        <input type="checkbox" bind:checked={draft.countdown} onchange={queueSave} />
        Pin to widget Countdown section
      </label>
      <label>Priority
        <div class="prios">
          {#each [0, 1, 2, 3] as p}
            <button class:active={draft.priority === p} onclick={() => { draft.priority = p; queueSave(); }}>{['None', 'Low', 'Med', 'High'][p]}</button>
          {/each}
        </div>
      </label>
      <label>Notes<textarea rows="6" bind:value={draft.notes} oninput={queueSave}></textarea></label>
      <footer>
        <button class="danger" onclick={() => editing && store.deleteTask(editing.id)}>Delete</button>
        <small>Autosaved locally</small>
      </footer>
    </aside>
  {/if}

  {#if editingNote}
    <aside class="drawer">
      <header><strong>Note</strong><button class="x" onclick={() => (store.editingNoteId = null)}>×</button></header>
      <input class="big" bind:value={noteDraft.title} oninput={queueNoteSave} />
      <label>Body<textarea rows="18" bind:value={noteDraft.body} oninput={queueNoteSave}></textarea></label>
      <footer>
        <button onclick={async () => { if (editingNote) { await api.saveNote({ id: editingNote.id, title: noteDraft.title, body: noteDraft.body, pinned: !editingNote.pinned }); await store.refresh(); } }}>
          {editingNote.pinned ? 'Unpin' : 'Pin'}
        </button>
        <button class="danger" onclick={() => editingNote && store.deleteNote(editingNote.id)}>Delete</button>
      </footer>
    </aside>
  {/if}

  {#if store.toast}
    <div class="toast">
      {store.toast.msg}
      {#if store.toast.undo}<button onclick={() => { store.toast?.undo?.(); store.toast = null; }}>Undo</button>{/if}
    </div>
  {/if}
</div>

<style>
  .shell { display: grid; grid-template-columns: 232px 1fr; height: 100vh; }
  .rail { background: rgba(12,15,26,.72); border-right: 1px solid var(--line); padding: 18px 14px; overflow-y: auto; }
  .brand { display: flex; align-items: center; gap: 10px; font-weight: 700; font-size: 17px; margin-bottom: 20px; }
  .mark { width: 22px; height: 22px; border-radius: 7px; background: var(--grad); }
  nav { display: grid; gap: 4px; margin-bottom: 18px; }
  nav button, .list-row { display: flex; align-items: center; gap: 8px; width: 100%; padding: 8px 10px; border-radius: 9px; background: none; border: 0; color: var(--text-2); font: inherit; font-size: 13.5px; cursor: pointer; text-align: left; }
  nav button:hover, .list-row:hover { background: rgba(124,131,255,.1); color: var(--text-1); }
  nav button.active, .list-row.active { background: rgba(124,131,255,.18); color: var(--text-1); }
  nav em, .list-row em { margin-left: auto; font-style: normal; font-size: 11px; color: var(--text-3); }
  .list-row i { width: 8px; height: 8px; border-radius: 50%; }
  .rail-title { font-size: 10.5px; letter-spacing: .12em; text-transform: uppercase; color: var(--text-3); margin: 16px 0 6px 10px; }
  .mini { width: 100%; margin-top: 6px; background: none; border: 1px dashed var(--line); border-radius: 8px; padding: 7px 9px; color: var(--text-2); font: inherit; font-size: 12.5px; }
  .foot { margin-top: 20px; font-size: 11px; color: var(--text-3); padding-left: 10px; }
  .stage { display: flex; flex-direction: column; min-width: 0; overflow: hidden; }
  .topbar { display: flex; gap: 12px; align-items: center; padding: 16px 22px; border-bottom: 1px solid var(--line); }
  .search { flex: 1; background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 10px; padding: 9px 13px; color: var(--text-1); font: inherit; }
  .badge { display: flex; align-items: center; gap: 7px; font-size: 12px; color: var(--text-2); }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--mint); box-shadow: 0 0 0 0 rgba(126,240,178,.6); animation: pulse 2.4s infinite; }
  @keyframes pulse { 70% { box-shadow: 0 0 0 7px rgba(126,240,178,0); } 100% { box-shadow: 0 0 0 0 rgba(126,240,178,0); } }
  .controls { display: flex; gap: 8px; align-items: center; padding: 14px 22px 0; flex-wrap: wrap; }
  .chip { padding: 6px 12px; border-radius: 999px; border: 1px solid var(--line); background: none; color: var(--text-2); font: inherit; font-size: 12.5px; cursor: pointer; }
  .chip.on { background: rgba(124,131,255,.2); border-color: transparent; color: var(--text-1); }
  select { background: var(--ink-2); border: 1px solid var(--line); border-radius: 8px; color: var(--text-1); padding: 6px 9px; font: inherit; font-size: 12.5px; }
  .add { display: flex; gap: 8px; padding: 14px 22px; }
  .add input { flex: 1; background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 10px; padding: 10px 13px; color: var(--text-1); font: inherit; }
  .add .date-pick { flex: 0 0 148px; }
  .primary { background: var(--grad); border: 0; border-radius: 10px; padding: 10px 18px; color: #fff; font: inherit; font-weight: 600; cursor: pointer; }
  .bulk { display: flex; gap: 10px; align-items: center; margin: 0 22px 10px; padding: 9px 14px; border-radius: 10px; background: rgba(124,131,255,.14); font-size: 13px; }
  .bulk button { background: none; border: 1px solid var(--line); border-radius: 7px; color: var(--text-1); padding: 4px 10px; cursor: pointer; font: inherit; font-size: 12px; }
  .rows { flex: 1; overflow-y: auto; padding: 0 22px 24px; }
  .row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 12px; border: 1px solid transparent; animation: rise .28s both; animation-delay: calc(var(--i) * 18ms); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .row:hover { background: rgba(255,255,255,.04); border-color: var(--line); }
  .row.done .title { text-decoration: line-through; color: var(--text-3); }
  .grip { cursor: grab; color: var(--text-3); font-size: 13px; opacity: 0; }
  .row:hover .grip { opacity: 1; }
  .check { width: 21px; height: 21px; border-radius: 50%; border: 1.6px solid var(--text-3); background: none; cursor: pointer; display: grid; place-items: center; flex: none; }
  .check svg { width: 13px; height: 13px; fill: none; stroke: #fff; stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; stroke-dasharray: 24; stroke-dashoffset: 24; }
  .check.on { background: var(--grad); border-color: transparent; }
  .check.on svg { stroke-dashoffset: 0; transition: stroke-dashoffset .22s ease .04s; }
  .row-body { flex: 1; min-width: 0; background: none; border: 0; text-align: left; cursor: pointer; color: inherit; font: inherit; padding: 0; }
  .title { display: block; font-size: 14px; color: var(--text-1); }
  .meta { display: flex; align-items: center; gap: 8px; font-size: 11.5px; color: var(--text-3); margin-top: 3px; }
  .meta i { width: 7px; height: 7px; border-radius: 50%; }
  .due { padding: 1px 7px; border-radius: 6px; background: rgba(255,255,255,.06); }
  .due.over { background: rgba(255,84,112,.16); color: var(--danger); }
  .cd-tag { padding: 1px 7px; border-radius: 6px; background: rgba(56,225,255,.14); color: var(--cyan); font-weight: 600; }
  .tag { padding: 1px 6px; border-radius: 6px; background: rgba(168,85,247,.18); color: #c4a5ff; }
  .prio { font-size: 13px; } .p1 { color: var(--cyan); } .p2 { color: var(--amber); } .p3 { color: var(--danger); }
  .sel { accent-color: #7c83ff; opacity: 0; } .row:hover .sel { opacity: 1; }
  .empty { padding: 40px; text-align: center; color: var(--text-3); font-size: 13.5px; }

  /* Calendar view */
  .cal-board { flex: 1; display: grid; grid-template-columns: repeat(7, 1fr); gap: 1px; background: var(--line); margin: 14px 22px 22px; border-radius: 14px; overflow: hidden; border: 1px solid var(--line); }
  .cal-col { background: rgba(11,14,26,.85); padding: 12px 10px; display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  .cal-col.today { background: rgba(124,131,255,.12); }
  .cal-head { display: flex; justify-content: space-between; align-items: baseline; border-bottom: 1px solid var(--line); padding-bottom: 6px; }
  .cal-head span { font-size: 11px; text-transform: uppercase; color: var(--text-3); }
  .cal-head strong { font-size: 16px; }
  .cal-col.today .cal-head strong { color: var(--cyan); }
  .cal-list { display: flex; flex-direction: column; gap: 6px; overflow-y: auto; flex: 1; }
  .cal-item { display: flex; justify-content: space-between; align-items: center; gap: 6px; padding: 6px 8px; border-radius: 8px; border: 0; border-left: 3px solid var(--c); background: rgba(255,255,255,.06); color: var(--text-1); font-size: 12px; text-align: left; cursor: pointer; }
  .cal-item.done { opacity: .45; text-decoration: line-through; }

  /* Pomodoro view */
  .pomo-wrap { flex: 1; display: grid; place-items: center; padding: 24px; }
  .pomo-card { background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 20px; padding: 32px; width: min(420px, 100%); display: grid; gap: 20px; justify-items: center; }
  .pomo-modes { display: flex; gap: 8px; }
  .pomo-clock { font-size: 64px; font-weight: 700; font-variant-numeric: tabular-nums; letter-spacing: .03em; }
  .pomo-actions { display: flex; gap: 10px; }
  .pomo-card small { color: var(--text-3); font-size: 12px; }

  .notes { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 12px; padding: 16px 22px; overflow-y: auto; }
  .note { text-align: left; background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 14px; padding: 14px; cursor: pointer; color: inherit; font: inherit; }
  .note:hover { border-color: rgba(124,131,255,.5); transform: translateY(-2px); transition: .16s; }
  .note strong { display: block; font-size: 14px; margin-bottom: 6px; }
  .note p { font-size: 12.5px; color: var(--text-2); white-space: pre-wrap; max-height: 88px; overflow: hidden; }
  .note span { font-size: 11px; color: var(--text-3); }
  .habits { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; padding: 4px 22px 22px; overflow-y: auto; }
  .habit { background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 14px; padding: 14px; }
  .habit-head { display: flex; align-items: center; gap: 10px; }
  .emoji { width: 34px; height: 34px; border-radius: 10px; display: grid; place-items: center; font-size: 17px; }
  .habit-head strong { display: block; font-size: 14px; }
  .habit-head small { color: var(--text-3); font-size: 11.5px; }
  .x { margin-left: auto; background: none; border: 0; color: var(--text-3); font-size: 18px; cursor: pointer; }
  .strip { display: flex; gap: 6px; margin-top: 12px; }
  .pip { flex: 1; aspect-ratio: 1; border-radius: 9px; border: 1px solid var(--line); background: none; color: var(--text-3); font: inherit; font-size: 11px; cursor: pointer; }
  .pip.on { background: var(--c); border-color: transparent; color: #08101c; font-weight: 700; }
  .pip.today { box-shadow: 0 0 0 2px rgba(124,131,255,.55); }
  .drawer { position: fixed; right: 0; top: 0; bottom: 0; width: 380px; background: rgba(12,15,26,.96); backdrop-filter: blur(24px); border-left: 1px solid var(--line); padding: 18px; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; animation: slide .22s ease; }
  @keyframes slide { from { transform: translateX(24px); opacity: 0; } }
  .drawer header { display: flex; align-items: center; }
  .drawer .big { background: none; border: 0; border-bottom: 1px solid var(--line); padding: 8px 0; color: var(--text-1); font: inherit; font-size: 17px; font-weight: 600; }
  .drawer label { display: grid; gap: 6px; font-size: 11px; letter-spacing: .08em; text-transform: uppercase; color: var(--text-3); }
  .drawer label.inline-toggle { display: flex; align-items: center; gap: 8px; font-size: 12.5px; text-transform: none; letter-spacing: 0; color: var(--text-2); cursor: pointer; }
  .drawer input[type="checkbox"] { accent-color: var(--cyan); width: 15px; height: 15px; }
  .drawer input:not([type="checkbox"]), .drawer textarea, .drawer select { background: rgba(255,255,255,.04); border: 1px solid var(--line); border-radius: 9px; padding: 9px 11px; color: var(--text-1); font: inherit; font-size: 13px; text-transform: none; letter-spacing: 0; }
  .drawer textarea { resize: vertical; line-height: 1.55; }
  .prios { display: flex; gap: 6px; }
  .prios button { flex: 1; padding: 7px; border-radius: 8px; border: 1px solid var(--line); background: none; color: var(--text-2); cursor: pointer; font: inherit; font-size: 12px; }
  .prios button.active { background: rgba(124,131,255,.22); color: var(--text-1); border-color: transparent; }
  .drawer footer { margin-top: auto; display: flex; align-items: center; gap: 10px; }
  .drawer footer small { margin-left: auto; color: var(--text-3); font-size: 11px; }
  .danger { background: rgba(255,84,112,.16); border: 0; border-radius: 8px; color: var(--danger); padding: 8px 14px; cursor: pointer; font: inherit; font-size: 12.5px; }
  .error { margin: 12px 22px; padding: 10px 14px; border-radius: 10px; background: rgba(255,84,112,.14); color: #ffb3c0; font-size: 12.5px; }
  .toast { position: fixed; bottom: 22px; left: 50%; transform: translateX(-50%); display: flex; gap: 14px; align-items: center; background: rgba(20,24,40,.97); border: 1px solid var(--line); border-radius: 12px; padding: 11px 16px; font-size: 13px; box-shadow: 0 18px 48px rgba(0,0,0,.5); animation: rise .2s; }
  .toast button { background: none; border: 0; color: var(--cyan); font: inherit; font-weight: 600; cursor: pointer; }
</style>