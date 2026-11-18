<script lang="ts">
  import { addHistory } from "$lib/history";
  import { loadHistoryEnabled } from "$lib/settings.svelte";
  import {
    EXT_BY_FORMAT,
    WEB_IMAGE_FORMATS,
    convertImage,
    getConvertEngine,
    normalizeImageFormat,
    type ImageFormat,
  } from "$lib/wasm";

  let file = $state<File | null>(null);
  let inputFormat = $state<ImageFormat>("png");
  let outputFormat = $state<ImageFormat>("webp");
  let quality = $state<number | "">(90);
  let width = $state<number | "">("");
  let height = $state<number | "">("");
  let busy = $state(false);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);
  let engineLabel = $state("detecting…");
  let dragOver = $state(false);
  let previewUrl = $state<string | null>(null);

  $effect(() => {
    let cancelled = false;
    (async () => {
      const engine = await getConvertEngine();
      if (!cancelled) {
        engineLabel =
          engine.id === "wasm" ? "WASM core (@fileprism/wasm)" : "canvas stub (WASM unavailable)";
      }
    })();
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    return () => {
      if (previewUrl) URL.revokeObjectURL(previewUrl);
    };
  });

  function setFile(next: File | null) {
    if (previewUrl) {
      URL.revokeObjectURL(previewUrl);
      previewUrl = null;
    }
    file = next;
    message = null;
    error = null;
    if (!next) return;
    previewUrl = URL.createObjectURL(next);
    const ext = next.name.split(".").pop() ?? "";
    const detected = normalizeImageFormat(ext) ?? normalizeImageFormat(next.type.replace("image/", ""));
    if (detected) inputFormat = detected;
  }

  function onFileInput(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    setFile(input.files?.[0] ?? null);
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const dropped = e.dataTransfer?.files?.[0] ?? null;
    if (dropped) setFile(dropped);
  }

  function downloadBytes(bytes: Uint8Array, filename: string, mimeType: string) {
    const blob = new Blob([bytes.slice()], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function outputName(source: string, format: ImageFormat): string {
    const base = source.replace(/\.[^.]+$/, "") || "converted";
    return `${base}.${EXT_BY_FORMAT[format]}`;
  }

  async function onSubmit(e: Event) {
    e.preventDefault();
    message = null;
    error = null;

    if (!file) {
      error = "Choose an image first.";
      return;
    }

    busy = true;
    try {
      const buffer = new Uint8Array(await file.arrayBuffer());
      const options: { quality?: number; width?: number; height?: number } = {};
      if (quality !== "") options.quality = Number(quality);
      if (width !== "") options.width = Number(width);
      if (height !== "") options.height = Number(height);

      const result = await convertImage(buffer, inputFormat, outputFormat, options);
      const name = outputName(file.name, outputFormat);
      downloadBytes(result.bytes, name, result.mimeType);

      if (loadHistoryEnabled()) {
        addHistory({
          inputName: file.name,
          inputFormat,
          outputFormat,
          status: "completed",
          bytesWritten: result.bytesWritten,
          engine: result.engine,
        });
      }

      message = `Converted with ${result.engine} — downloaded ${name} (${result.bytesWritten.toLocaleString()} bytes).`;
    } catch (err) {
      const text = err instanceof Error ? err.message : String(err);
      error = text;
      if (loadHistoryEnabled()) {
        addHistory({
          inputName: file.name,
          inputFormat,
          outputFormat,
          status: "failed",
          errorMessage: text,
        });
      }
    } finally {
      busy = false;
    }
  }
</script>

<section class="view">
  <header class="hero">
    <h1>Convert</h1>
    <p class="lede">Pick an image, convert in the browser, download the result.</p>
    <p class="engine" role="status">Engine: {engineLabel}</p>
  </header>

  <form
    class="panel"
    onsubmit={onSubmit}
    ondragover={(e) => {
      e.preventDefault();
      dragOver = true;
    }}
    ondragleave={() => (dragOver = false)}
    ondrop={onDrop}
  >
    <label class="drop" class:active={dragOver}>
      <input type="file" accept="image/png,image/jpeg,image/webp,.png,.jpg,.jpeg,.webp" onchange={onFileInput} />
      {#if file}
        <span class="drop-title">{file.name}</span>
        <span class="drop-hint">{(file.size / 1024).toFixed(1)} KB · click to replace</span>
      {:else}
        <span class="drop-title">Drop an image or browse</span>
        <span class="drop-hint">PNG, JPG, or WebP</span>
      {/if}
    </label>

    {#if previewUrl}
      <img class="preview" src={previewUrl} alt="Selected input preview" />
    {/if}

    <div class="grid">
      <label class="field">
        <span>Input format</span>
        <select bind:value={inputFormat}>
          {#each WEB_IMAGE_FORMATS as fmt}
            <option value={fmt}>{fmt}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>Output format</span>
        <select bind:value={outputFormat}>
          {#each WEB_IMAGE_FORMATS as fmt}
            <option value={fmt}>{fmt}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>Quality (JPG/WebP)</span>
        <input type="number" min="1" max="100" placeholder="90" bind:value={quality} />
      </label>
      <label class="field">
        <span>Width (optional)</span>
        <input type="number" min="1" placeholder="auto" bind:value={width} />
      </label>
      <label class="field">
        <span>Height (optional)</span>
        <input type="number" min="1" placeholder="auto" bind:value={height} />
      </label>
    </div>

    <div class="actions">
      <button type="submit" class="primary" disabled={busy || !file}>
        {busy ? "Converting…" : "Convert & download"}
      </button>
    </div>

    {#if message}
      <p class="ok" role="status">{message}</p>
    {/if}
    {#if error}
      <p class="err" role="alert">{error}</p>
    {/if}
  </form>
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

  .engine {
    margin: var(--space-2) 0 0;
    font-size: var(--font-size-sm);
    color: var(--color-muted);
  }

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .drop {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-md);
    text-align: center;
    cursor: pointer;
    transition:
      border-color var(--motion-fast) var(--motion-ease),
      background var(--motion-fast) var(--motion-ease);
  }

  .drop.active,
  .drop:hover {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }

  .drop input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  .drop-title {
    font-weight: 600;
  }

  .drop-hint {
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .preview {
    max-width: 100%;
    max-height: 220px;
    object-fit: contain;
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
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
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  select,
  input[type="number"] {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    color: var(--color-text);
    padding: var(--space-3);
    font: inherit;
    font-weight: 400;
  }

  .actions {
    display: flex;
    gap: var(--space-3);
  }

  .primary {
    appearance: none;
    border: 0;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-5);
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    color: #fff;
    font-weight: 600;
    cursor: pointer;
  }

  .primary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .ok {
    margin: 0;
    color: var(--color-good);
    font-size: var(--font-size-sm);
  }

  .err {
    margin: 0;
    color: var(--color-bad);
    font-size: var(--font-size-sm);
  }
</style>
