<script lang="ts">
  import { formatError, isTauri, searchHistory, type JobRecord } from "$lib/api";
  import {
    applyShellFiles,
    getShellPaths,
    getShellRevision,
  } from "$lib/shell.svelte";

  let dragOver = $state(false);
  let recent = $state<JobRecord[]>([]);
  let recentError = $state<string | null>(null);

  let {
    onNavigate,
  }: {
    onNavigate?: (id: "convert" | "history") => void;
  } = $props();

  function basename(path: string): string {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  }

  async function loadRecent() {
    recentError = null;
    if (!isTauri()) {
      recent = [];
      return;
    }
    try {
      recent = (await searchHistory("", 5)).slice(0, 5);
    } catch (err) {
      recentError = formatError(err);
      recent = [];
    }
  }

  $effect(() => {
    void loadRecent();
  });

  $effect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    (async () => {
      const { getCurrentWebview } = await import("@tauri-apps/api/webview");
      if (cancelled) return;
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          dragOver = true;
        } else if (event.payload.type === "leave") {
          dragOver = false;
        } else if (event.payload.type === "drop") {
          dragOver = false;
          const paths = event.payload.paths ?? [];
          if (paths.length === 0) return;
          applyShellFiles({ v: 1, action: "convert", paths });
          onNavigate?.("convert");
        }
      });
    })();
    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function onDragLeave() {
    dragOver = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    // Browser drops lack absolute paths; Tauri drag-drop handler above owns the real path.
    if (!isTauri()) {
      onNavigate?.("convert");
    }
  }

  const shellPaths = $derived.by(() => {
    const _ = getShellRevision();
    return getShellPaths();
  });
</script>

<section class="home">
  <header class="hero">
    <h1>Home</h1>
    <p class="lede">Convert, compress, and organize files — locally, privately.</p>
  </header>

  {#if shellPaths.length > 0}
    <p class="shell-banner" role="status">
      Shell sent {shellPaths.length} path{shellPaths.length === 1 ? "" : "s"} —
      opened in the matching view.
    </p>
  {/if}

  <div
    class="dropzone"
    class:active={dragOver}
    role="region"
    aria-label="Drop files to convert"
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
  >
    <p class="drop-title">Drop files here</p>
    <p class="drop-hint">
      Opens Convert with the dropped path · or use Browse on Convert / Compress / Archives
    </p>
  </div>

  <div class="panels">
    <section class="panel">
      <h2>Quick start</h2>
      <div class="quick">
        <button type="button" class="ghost" onclick={() => onNavigate?.("convert")}>
          Open Convert
        </button>
        <button type="button" class="ghost" onclick={() => onNavigate?.("history")}>
          Open History
        </button>
      </div>
    </section>
    <section class="panel">
      <h2>Recent history</h2>
      {#if recentError}
        <p class="empty">{recentError}</p>
      {:else if recent.length === 0}
        <p class="empty">History appears after your first conversion.</p>
      {:else}
        <ul class="recent">
          {#each recent as job (job.id)}
            <li>
              <button type="button" class="recent-item" onclick={() => onNavigate?.("history")}>
                <span class="name">{basename(job.input_path)}</span>
                <span class="meta">{job.input_format} → {job.output_format} · {job.status}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  </div>
</section>

<style>
  .home {
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

  .shell-banner {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-text);
    font-size: var(--font-size-sm);
  }

  .dropzone {
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    text-align: center;
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
    transition:
      border-color var(--motion-fast) var(--motion-ease),
      background var(--motion-fast) var(--motion-ease);
  }

  .dropzone.active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
  }

  .drop-title {
    margin: 0;
    font-weight: 600;
  }

  .drop-hint {
    margin: var(--space-2) 0 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  @media (max-width: 800px) {
    .panels {
      grid-template-columns: 1fr;
    }
  }

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
  }

  .panel h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--font-size-base);
    font-weight: 600;
  }

  .quick {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .ghost {
    appearance: none;
    border: 0;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    background: color-mix(in srgb, var(--color-text) 8%, transparent);
    color: var(--color-text);
    font-weight: 600;
    cursor: pointer;
  }

  .empty {
    margin: 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .recent {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .recent-item {
    appearance: none;
    border: 0;
    width: 100%;
    text-align: left;
    background: transparent;
    color: var(--color-text);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .recent-item:hover {
    background: color-mix(in srgb, var(--color-text) 6%, transparent);
  }

  .name {
    font-weight: 600;
    font-size: var(--font-size-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    font-size: 0.75rem;
    color: var(--color-muted);
  }
</style>
