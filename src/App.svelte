<!-- src/App.svelte -->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { isTauri } from './lib/api';
  import { store } from './lib/store.svelte';

  type Loader = () => Promise<{ default: any }>;
  const routes: Record<string, Loader> = {
    main: () => import('./windows/MainWindow.svelte'),
    week: () => import('./windows/WeekWidget.svelte'),
    next7: () => import('./windows/Next7Widget.svelte')
  };

  let Component = $state<any>(null);
  let label = $state('main');

  $effect(() => {
    (async () => {
      label = isTauri
        ? getCurrentWindow().label
        : new URLSearchParams(location.search).get('window') ?? 'main';
      document.body.dataset.window = label;
      Component = (await (routes[label] ?? routes.main)()).default;
      await store.init();
    })();
  });
</script>

{#if Component}
  <Component />
{:else}
  <div class="boot">Loading Tempo…</div>
{/if}

<style>
  .boot { display: grid; place-items: center; height: 100vh; color: var(--text-2); font-size: 14px; letter-spacing: .04em; }
</style>