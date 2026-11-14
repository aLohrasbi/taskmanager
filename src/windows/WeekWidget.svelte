<!-- src/windows/WeekWidget.svelte -->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { api, isTauri } from '../lib/api';
  import { addDays, dateKey, fmtCountdown, startOfDay, startOfWeek, todayKey } from '../lib/dates';
  import '../styles/widget.css';

  let offset = $state(0);
  let adding = $state<string | null>(null);
  let title = $state('');

  const days = $derived(Array.from({ length: 7 }, (_, i) => addDays(startOfWeek(addDays(new Date(), offset * 7)), i)));
  const range = $derived({ from: startOfDay(days[0]), to: new Date(startOfDay(days[6]).getTime() + 86399000) });

  $effect(() => {
    void store.tasks;
    void store.completions;
    store.loadCalendar(range.from, range.to);
  });

  const itemsFor = (d: Date) => store.calendar.filter((c) => dateKey(new Date(c.startAt)) === dateKey(d));
  const color = (id: string | null) => store.lists.find((l) => l.id === id)?.color ?? '#7c83ff';
  const me = $derived(store.widgets.find((w) => w.label === 'week'));

  async function add(d: Date) {
    if (!title.trim()) { adding = null; return; }
    const due = new Date(d.getFullYear(), d.getMonth(), d.getDate(), 12, 0, 0);
    await store.quickAdd(title, due.toISOString());
    title = ''; adding = null;
  }
</script>

<section class="panel">
  <header data-tauri-drag-region>
    <span class="grip" data-tauri-drag-region>⠿</span>
    <strong data-tauri-drag-region>{offset === 0 ? 'This week' : `${days[0].toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} – ${days[6].toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}`}</strong>
    <div class="spacer" data-tauri-drag-region></div>
    <button onclick={() => (offset -= 1)} title="Previous week">‹</button>
    <button onclick={() => (offset = 0)} title="This week">•</button>
    <button onclick={() => (offset += 1)} title="Next week">›</button>
    {#if isTauri}
      <button onclick={() => api.setPin('week', me?.pin === 'top' ? 'bottom' : 'top')} title="Toggle float">{me?.pin === 'top' ? '📌' : '📍'}</button>
      <button onclick={() => api.hideWidget('week')} title="Hide">×</button>
    {/if}
  </header>

  {#if store.countdowns.length}
    <div class="cd-strip">
      {#each store.countdowns as cd (cd.id)}
        <button class="cd-pill" onclick={() => store.toggleTask(cd.id)} title="Complete countdown">
          <span class="cd-dot" class:high={cd.priority >= 3}></span>
          <span class="cd-name">{cd.title}</span>
          <em>{fmtCountdown(cd.dueAt)}</em>
        </button>
      {/each}
    </div>
  {/if}

  <div class="grid">
    {#each days as d, i (d.toISOString())}
      {@const items = itemsFor(d)}
      <div class="col" class:today={dateKey(d) === todayKey()} style="--i:{i}">
        <div class="col-head">
          <span>{d.toLocaleDateString(undefined, { weekday: 'short' })}</span>
          <b>{d.getDate()}</b>
        </div>
        <div class="chips">
          {#each items as it (it.taskId + it.occurrenceKey)}
            <button class="chip" class:done={it.completed} style="--c:{color(it.listId)}"
                    onclick={() => store.toggleTask(it.taskId, it.occurrenceKey)}>
              <i></i><span>{it.title}</span>
            </button>
          {/each}
        </div>
        {#if adding === dateKey(d)}
          <input class="quick" autofocus bind:value={title} placeholder="Task…"
                 onkeydown={(e) => { if (e.key === 'Enter') add(d); if (e.key === 'Escape') adding = null; }}
                 onblur={() => add(d)} />
        {:else}
          <button class="plus" onclick={() => (adding = dateKey(d))}>+</button>
        {/if}
      </div>
    {/each}
  </div>
</section>