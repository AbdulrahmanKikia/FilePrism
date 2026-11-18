<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import "$lib/styles/tokens.css";
  import ArchivesView from "$lib/components/ArchivesView.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import CompressView from "$lib/components/CompressView.svelte";
  import ConvertView from "$lib/components/ConvertView.svelte";
  import HistoryView from "$lib/components/HistoryView.svelte";
  import LicenseView from "$lib/components/LicenseView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Sidebar, { type NavId } from "$lib/components/Sidebar.svelte";
  import { isTauri } from "$lib/api";
  import {
    applyShellFiles,
    navForAction,
    type ShellFilesPayload,
  } from "$lib/shell.svelte";
  import {
    applySettingsToDom,
    DEFAULT_SETTINGS,
    loadHistoryEnabled,
    loadUiSettings,
    type UiSettings,
  } from "$lib/settings.svelte";

  let { children }: { children: Snippet } = $props();
  let active = $state<NavId>("home");
  let paletteOpen = $state(false);
  let settings = $state<UiSettings>({ ...DEFAULT_SETTINGS });
  let historyEnabled = $state(true);
  let shellEl: HTMLElement | undefined = $state();

  const labels: Record<NavId, string> = {
    home: "Home",
    convert: "Convert",
    compress: "Compress",
    archives: "Archives",
    history: "History",
    presets: "Presets",
    watched: "Watched folders",
    settings: "Settings",
    license: "License",
  };

  function onNavigate(id: NavId) {
    active = id;
  }

  setContext("fp-navigate", onNavigate);

  function apply(next: UiSettings = settings) {
    const root = shellEl ?? document.documentElement;
    applySettingsToDom(next, root);
  }

  $effect(() => {
    let cancelled = false;
    (async () => {
      const loaded = await loadUiSettings();
      const hist = await loadHistoryEnabled();
      if (cancelled) return;
      settings = loaded;
      historyEnabled = hist;
      apply(loaded);
    })();
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      if (cancelled) return;
      unlisten = await listen<ShellFilesPayload>("shell-files", (event) => {
        applyShellFiles(event.payload);
        active = navForAction(event.payload.action);
      });
    })();
    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  function onGlobalKey(e: KeyboardEvent) {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.key.toLowerCase() === "k") {
      e.preventDefault();
      paletteOpen = !paletteOpen;
    }
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

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
  <Sidebar {active} onnavigate={onNavigate} />
  <main class="main">
    {#if active === "home"}
      {@render children()}
    {:else if active === "convert"}
      <ConvertView onOpenHistory={() => onNavigate("history")} />
    {:else if active === "compress"}
      <CompressView />
    {:else if active === "archives"}
      <ArchivesView />
    {:else if active === "history"}
      <HistoryView />
    {:else if active === "settings"}
      <SettingsView
        bind:settings
        bind:historyEnabled
        onchange={(next) => apply(next)}
      />
    {:else if active === "license"}
      <LicenseView />
    {:else}
      <section class="placeholder view-enter">
        <h1>{labels[active]}</h1>
        <p>Coming in a later milestone.</p>
      </section>
    {/if}
  </main>
</div>

<CommandPalette bind:open={paletteOpen} onnavigate={onNavigate} />

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

  .placeholder h1 {
    margin: 0;
    font-size: 1.5rem;
    letter-spacing: -0.02em;
  }

  .placeholder p {
    margin: var(--space-2) 0 0;
    color: var(--color-muted);
  }

  .view-enter {
    animation: view-enter var(--motion-base) var(--motion-ease);
  }
</style>
