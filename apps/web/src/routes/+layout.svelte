<script lang="ts">
  import { page } from "$app/state";
  import "$lib/styles/tokens.css";
  import Sidebar, { type NavId } from "$lib/components/Sidebar.svelte";
  import {
    applySettingsToDom,
    DEFAULT_SETTINGS,
    loadUiSettings,
    type UiSettings,
  } from "$lib/settings.svelte";

  let { children } = $props();
  let settings = $state<UiSettings>({ ...DEFAULT_SETTINGS });
  let shellEl: HTMLElement | undefined = $state();

  const active = $derived.by((): NavId => {
    const path = page.url.pathname.replace(/\/$/, "") || "/";
    if (path.startsWith("/convert")) return "convert";
    if (path.startsWith("/history")) return "history";
    if (path.startsWith("/settings")) return "settings";
    return "home";
  });

  function apply(next: UiSettings = settings) {
    const root = shellEl ?? document.documentElement;
    applySettingsToDom(next, root);
  }

  $effect(() => {
    settings = loadUiSettings();
    apply(settings);
  });
</script>

<div
  class="shell"
  bind:this={shellEl}
  data-theme={settings.theme === "system" ? undefined : settings.theme}
  data-accent={settings.accent}
  data-glass={settings.glass}
  data-density={settings.density}
  data-motion={settings.motion}
  data-font={settings.font}
>
  <Sidebar {active} />
  <main class="main">
    {@render children()}
  </main>
</div>

<style>
  .shell {
    display: flex;
    min-height: 100vh;
    min-height: 100dvh;
  }

  .main {
    flex: 1;
    min-width: 0;
    padding: var(--space-5) var(--space-6);
    overflow: auto;
  }
</style>
