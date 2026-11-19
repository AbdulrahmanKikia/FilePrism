<script lang="ts">
  /** Shared path / format / options form used by Convert, Compress, Archives. */

  import { isTauri } from "$lib/api";
  import { pickPath, type PathKind } from "$lib/dialogs";
  import type { DialogFilter } from "@tauri-apps/plugin-dialog";

  let {
    inputPath = $bindable(""),
    outputPath = $bindable(""),
    inputFormat = $bindable(""),
    outputFormat = $bindable(""),
    password = $bindable(""),
    quality = $bindable(100),
    width = $bindable<number | "">(""),
    height = $bindable<number | "">(""),
    showPassword = false,
    showQuality = false,
    showResize = false,
    showImageOptions = false,
    inputFormats = [] as string[],
    outputFormats = [] as string[],
    inputLabel = "Input path",
    outputLabel = "Output path",
    inputHint = "Absolute path to the source file or folder",
    outputHint = "Absolute path for the result",
    inputPicker = "file" as PathKind,
    outputPicker = "save" as PathKind,
    inputFilters = [] as DialogFilter[],
    outputFilters = [] as DialogFilter[],
    onInputPicked = undefined as ((path: string) => void) | undefined,
    onOutputPicked = undefined as ((path: string) => void) | undefined,
    onOutputPathChange = undefined as ((path: string) => void) | undefined,
  }: {
    inputPath?: string;
    outputPath?: string;
    inputFormat?: string;
    outputFormat?: string;
    password?: string;
    quality?: number;
    width?: number | "";
    height?: number | "";
    showPassword?: boolean;
    showQuality?: boolean;
    showResize?: boolean;
    /** @deprecated Prefer showQuality / showResize. Enables both when true. */
    showImageOptions?: boolean;
    inputFormats?: string[];
    outputFormats?: string[];
    inputLabel?: string;
    outputLabel?: string;
    inputHint?: string;
    outputHint?: string;
    inputPicker?: PathKind;
    outputPicker?: PathKind;
    inputFilters?: DialogFilter[];
    outputFilters?: DialogFilter[];
    onInputPicked?: (path: string) => void;
    onOutputPicked?: (path: string) => void;
    onOutputPathChange?: (path: string) => void;
  } = $props();

  const qualityVisible = $derived(showQuality || showImageOptions);
  const resizeVisible = $derived(showResize || showImageOptions);

  let browsing = $state<"input" | "output" | null>(null);

  async function browseInput() {
    if (!isTauri() || browsing) return;
    browsing = "input";
    try {
      const path = await pickPath({
        kind: inputPicker,
        title: inputLabel,
        defaultPath: inputPath || undefined,
        filters: inputFilters.length ? inputFilters : undefined,
      });
      if (path) {
        inputPath = path;
        onInputPicked?.(path);
      }
    } finally {
      browsing = null;
    }
  }

  async function browseOutput() {
    if (!isTauri() || browsing) return;
    browsing = "output";
    try {
      const path = await pickPath({
        kind: outputPicker,
        title: outputLabel,
        defaultPath: outputPath || inputPath || undefined,
        filters: outputFilters.length ? outputFilters : undefined,
      });
      if (path) {
        outputPath = path;
        onOutputPicked?.(path);
      }
    } finally {
      browsing = null;
    }
  }

  function onOutputInput(e: Event) {
    const value = (e.currentTarget as HTMLInputElement).value;
    outputPath = value;
    onOutputPathChange?.(value);
  }
</script>

<div class="form">
  <label class="field">
    <span>{inputLabel}</span>
    <div class="path-row">
      <input type="text" bind:value={inputPath} placeholder="C:\path\to\input" autocomplete="off" />
      <button
        type="button"
        class="browse"
        disabled={!isTauri() || browsing !== null}
        onclick={browseInput}
      >
        {browsing === "input" ? "…" : "Browse"}
      </button>
    </div>
    <small>{inputHint}{#if !isTauri()} · Browse needs the Tauri shell{/if}</small>
  </label>

  <label class="field">
    <span>{outputLabel}</span>
    <div class="path-row">
      <input
        type="text"
        value={outputPath}
        oninput={onOutputInput}
        placeholder="C:\path\to\output"
        autocomplete="off"
      />
      <button
        type="button"
        class="browse"
        disabled={!isTauri() || browsing !== null}
        onclick={browseOutput}
      >
        {browsing === "output" ? "…" : "Browse"}
      </button>
    </div>
    <small>{outputHint}</small>
  </label>

  <div class="row">
    <label class="field">
      <span>Input format</span>
      {#if inputFormats.length}
        <select bind:value={inputFormat}>
          {#each inputFormats as fmt}
            <option value={fmt}>{fmt}</option>
          {/each}
        </select>
      {:else}
        <input type="text" bind:value={inputFormat} placeholder="png" />
      {/if}
    </label>

    <label class="field">
      <span>Output format</span>
      {#if outputFormats.length}
        <select bind:value={outputFormat}>
          {#each outputFormats as fmt}
            <option value={fmt}>{fmt}</option>
          {/each}
        </select>
      {:else}
        <input type="text" bind:value={outputFormat} placeholder="webp" />
      {/if}
    </label>
  </div>

  {#if showPassword}
    <label class="field">
      <span>Password (optional)</span>
      <input type="password" bind:value={password} autocomplete="off" />
    </label>
  {/if}

  {#if qualityVisible}
    <label class="field quality">
      <span class="quality-label">
        <span>Quality</span>
        <strong>{quality}</strong>
      </span>
      <input type="range" min="1" max="100" bind:value={quality} />
    </label>
  {/if}

  {#if resizeVisible}
    <div class="row">
      <label class="field">
        <span>Width</span>
        <input type="number" min="1" bind:value={width} placeholder="auto" />
      </label>
      <label class="field">
        <span>Height</span>
        <input type="number" min="1" bind:value={height} placeholder="auto" />
      </label>
    </div>
  {/if}
</div>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  @media (max-width: 720px) {
    .row {
      grid-template-columns: 1fr;
    }
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--color-muted);
  }

  .field span {
    font-weight: 600;
    color: var(--color-text);
  }

  .field small {
    color: var(--color-muted);
    font-size: 0.75rem;
  }

  .quality-label {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .quality-label strong {
    font-variant-numeric: tabular-nums;
    color: var(--color-accent);
  }

  .quality input[type="range"] {
    width: 100%;
    accent-color: var(--color-accent);
    padding: 0;
    border: 0;
    background: transparent;
    box-shadow: none;
  }

  .path-row {
    display: flex;
    gap: var(--space-2);
    align-items: stretch;
  }

  .path-row input {
    flex: 1;
    min-width: 0;
  }

  .browse {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-text) 8%, transparent);
    color: var(--color-text);
    padding: var(--space-3) var(--space-4);
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
  }

  .browse:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  input,
  select {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    color: var(--color-text);
    padding: var(--space-3);
    outline: none;
    transition:
      border-color var(--motion-fast) var(--motion-ease),
      box-shadow var(--motion-fast) var(--motion-ease);
  }

  input:focus,
  select:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 25%, transparent);
  }
</style>
