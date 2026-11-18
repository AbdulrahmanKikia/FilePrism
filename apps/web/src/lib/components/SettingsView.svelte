<script lang="ts">
  import {
    ACCENTS,
    persistHistoryEnabled,
    persistUiSettings,
    type Accent,
    type Density,
    type FontSize,
    type Glass,
    type Motion,
    type Theme,
    type UiSettings,
  } from "$lib/settings.svelte";

  let {
    settings = $bindable(),
    historyEnabled = $bindable(true),
    onchange,
  }: {
    settings: UiSettings;
    historyEnabled?: boolean;
    onchange?: (next: UiSettings) => void;
  } = $props();

  function patch<K extends keyof UiSettings>(key: K, value: UiSettings[K]) {
    settings = { ...settings, [key]: value };
    onchange?.(settings);
    persistUiSettings(settings);
  }

  function onHistoryToggle(e: Event) {
    const checked = (e.currentTarget as HTMLInputElement).checked;
    historyEnabled = checked;
    persistHistoryEnabled(checked);
  }
</script>

<section class="view">
  <header class="hero">
    <h1>Settings</h1>
    <p class="lede">Theme tokens aligned with desktop Glass Pro — stored in this browser.</p>
  </header>

  <div class="grid">
    <label class="field">
      <span>Theme</span>
      <select
        value={settings.theme}
        onchange={(e) => patch("theme", (e.currentTarget as HTMLSelectElement).value as Theme)}
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>

    <fieldset class="field accents">
      <legend>Accent</legend>
      <div class="swatches">
        {#each ACCENTS as accent}
          <button
            type="button"
            class="swatch"
            class:active={settings.accent === accent}
            data-accent={accent}
            aria-label={accent}
            aria-pressed={settings.accent === accent}
            onclick={() => patch("accent", accent as Accent)}
          ></button>
        {/each}
      </div>
    </fieldset>

    <label class="field">
      <span>Glass</span>
      <select
        value={settings.glass}
        onchange={(e) => patch("glass", (e.currentTarget as HTMLSelectElement).value as Glass)}
      >
        <option value="off">Off</option>
        <option value="subtle">Subtle</option>
        <option value="full">Full</option>
      </select>
    </label>

    <label class="field">
      <span>Density</span>
      <select
        value={settings.density}
        onchange={(e) => patch("density", (e.currentTarget as HTMLSelectElement).value as Density)}
      >
        <option value="comfortable">Comfortable</option>
        <option value="compact">Compact</option>
      </select>
    </label>

    <label class="field">
      <span>Motion</span>
      <select
        value={settings.motion}
        onchange={(e) => patch("motion", (e.currentTarget as HTMLSelectElement).value as Motion)}
      >
        <option value="full">Full</option>
        <option value="reduced">Reduced</option>
      </select>
    </label>

    <label class="field">
      <span>Font size</span>
      <select
        value={settings.font}
        onchange={(e) => patch("font", (e.currentTarget as HTMLSelectElement).value as FontSize)}
      >
        <option value="system">System</option>
        <option value="small">Small</option>
        <option value="large">Large</option>
      </select>
    </label>

    <label class="toggle">
      <input type="checkbox" checked={historyEnabled} onchange={onHistoryToggle} />
      <span>Record conversion history in this browser</span>
    </label>
  </div>
</section>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    animation: view-enter var(--motion-base) var(--motion-ease);
  }

  .hero h1 {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 650;
    letter-spacing: -0.03em;
  }

  .lede {
    margin: var(--space-2) 0 0;
    color: var(--color-muted);
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    border: 0;
    margin: 0;
    padding: 0;
    min-width: 0;
  }

  .field span,
  .field legend {
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  select {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    color: var(--color-text);
    padding: var(--space-3);
  }

  .accents {
    grid-column: 1 / -1;
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    background: var(--color-accent);
  }

  .swatch.active {
    border-color: var(--color-text);
  }

  .swatch[data-accent="blue"] {
    background: #3b82f6;
  }
  .swatch[data-accent="violet"] {
    background: #8b5cf6;
  }
  .swatch[data-accent="teal"] {
    background: #2f9e8f;
  }
  .swatch[data-accent="orange"] {
    background: #f59e0b;
  }
  .swatch[data-accent="pink"] {
    background: #ec4899;
  }
  .swatch[data-accent="green"] {
    background: #22c55e;
  }

  .toggle {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--font-size-sm);
  }
</style>
