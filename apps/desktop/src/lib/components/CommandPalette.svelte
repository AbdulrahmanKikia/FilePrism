<script lang="ts">
  import type { NavId } from "./Sidebar.svelte";

  type Command = {
    id: string;
    label: string;
    hint?: string;
    nav?: NavId;
    action?: () => void;
  };

  let {
    open = $bindable(false),
    onnavigate,
  }: {
    open?: boolean;
    onnavigate?: (id: NavId) => void;
  } = $props();

  let query = $state("");
  let selected = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  const commands: Command[] = [
    { id: "home", label: "Go to Home", hint: "Navigation", nav: "home" },
    { id: "convert", label: "Go to Convert", hint: "Navigation", nav: "convert" },
    { id: "compress", label: "Go to Compress", hint: "Navigation", nav: "compress" },
    { id: "archives", label: "Go to Archives", hint: "Navigation", nav: "archives" },
    { id: "history", label: "Go to History", hint: "Navigation", nav: "history" },
    { id: "settings", label: "Go to Settings", hint: "Navigation", nav: "settings" },
    { id: "presets", label: "Go to Presets", hint: "Coming later", nav: "presets" },
    { id: "watched", label: "Go to Watched folders", hint: "Coming later", nav: "watched" },
    { id: "license", label: "Go to License", hint: "Coming later", nav: "license" },
  ];

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return commands;
    return commands.filter(
      (c) =>
        c.label.toLowerCase().includes(q) ||
        c.hint?.toLowerCase().includes(q) ||
        c.id.includes(q),
    );
  });

  $effect(() => {
    if (open) {
      query = "";
      selected = 0;
      queueMicrotask(() => inputEl?.focus());
    }
  });

  $effect(() => {
    // Keep selection in range when filter changes.
    if (selected >= filtered.length) selected = Math.max(0, filtered.length - 1);
  });

  function run(cmd: Command) {
    open = false;
    if (cmd.nav) onnavigate?.(cmd.nav);
    cmd.action?.();
  }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") {
      e.preventDefault();
      open = false;
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, Math.max(0, filtered.length - 1));
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const cmd = filtered[selected];
      if (cmd) run(cmd);
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => (open = false)} role="presentation">
    <div
      class="palette"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-label="Command palette"
      onclick={(e) => e.stopPropagation()}
    >
      <input
        bind:this={inputEl}
        class="query"
        type="search"
        placeholder="Jump to a view…"
        bind:value={query}
        aria-autocomplete="list"
        aria-controls="command-list"
      />
      <ul id="command-list" class="results" role="listbox">
        {#each filtered as cmd, i (cmd.id)}
          <li role="option" aria-selected={i === selected}>
            <button
              type="button"
              class="item"
              class:active={i === selected}
              onmouseenter={() => (selected = i)}
              onclick={() => run(cmd)}
            >
              <span class="label">{cmd.label}</span>
              {#if cmd.hint}
                <span class="hint">{cmd.hint}</span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="empty">No matching commands</li>
        {/each}
      </ul>
      <p class="footer"><kbd>↑↓</kbd> navigate · <kbd>Enter</kbd> run · <kbd>Esc</kbd> close</p>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    background: color-mix(in srgb, #000 45%, transparent);
    animation: fade-in var(--motion-fast) var(--motion-ease);
  }

  .palette {
    width: min(520px, calc(100vw - 32px));
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--color-surface) 92%, var(--color-bg));
    backdrop-filter: blur(var(--blur-glass));
    box-shadow: 0 24px 60px color-mix(in srgb, #000 35%, transparent);
    overflow: hidden;
    animation: view-enter var(--motion-base) var(--motion-ease);
  }

  .query {
    width: 100%;
    border: 0;
    border-bottom: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text);
    padding: var(--space-4);
    outline: none;
    font-size: var(--font-size-lg);
  }

  .results {
    list-style: none;
    margin: 0;
    padding: var(--space-2);
    max-height: 320px;
    overflow: auto;
  }

  .item {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--color-text);
    text-align: left;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .item.active,
  .item:hover {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
  }

  .hint {
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .empty {
    padding: var(--space-4);
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .footer {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--color-border);
    color: var(--color-muted);
    font-size: 0.75rem;
  }

  kbd {
    font: inherit;
    padding: 0 4px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
  }
</style>
