<script lang="ts">
  import { formatError, isTauri, runJob, type JobOptions } from "$lib/api";
  import {
    getShellAction,
    getShellPaths,
    getShellRevision,
  } from "$lib/shell.svelte";
  import JobForm from "./JobForm.svelte";

  const INPUT_FORMATS = [
    "png",
    "jpg",
    "webp",
    "avif",
    "heic",
    "pdf",
    "md",
    "json",
    "csv",
    "yaml",
    "xlsx",
  ];
  const OUTPUT_FORMATS = [
    "png",
    "jpg",
    "webp",
    "avif",
    "txt",
    "html",
    "pdf",
    "json",
    "csv",
    "yaml",
    "xlsx",
  ];

  let inputPath = $state("");
  let outputPath = $state("");
  let inputFormat = $state("png");
  let outputFormat = $state("webp");
  let quality = $state<number | "">("");
  let width = $state<number | "">("");
  let height = $state<number | "">("");
  let busy = $state(false);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const rev = getShellRevision();
    const action = getShellAction();
    const paths = getShellPaths();
    if (rev === 0 || paths.length === 0) return;
    if (action !== "open" && action !== "convert") return;
    inputPath = paths[0] ?? "";
    const ext = inputPath.split(".").pop()?.toLowerCase() ?? "";
    if (INPUT_FORMATS.includes(ext) || (ext === "jpeg" && INPUT_FORMATS.includes("jpg"))) {
      inputFormat = ext === "jpeg" ? "jpg" : ext;
    }
  });

  async function onSubmit(e: Event) {
    e.preventDefault();
    message = null;
    error = null;

    if (!inputPath.trim() || !outputPath.trim()) {
      error = "Input and output paths are required.";
      return;
    }
    if (!isTauri()) {
      error = "Conversion requires the Tauri desktop shell (pnpm tauri dev).";
      return;
    }

    busy = true;
    try {
      const options: JobOptions = {};
      if (quality !== "") options.quality = Number(quality);
      if (width !== "") options.width = Number(width);
      if (height !== "") options.height = Number(height);

      const out = await runJob({
        inputPath: inputPath.trim(),
        outputPath: outputPath.trim(),
        inputFormat,
        outputFormat,
        options,
      });
      message = `Done — wrote ${out.bytes_written} bytes to ${out.output_path} (${out.job_id}).`;
    } catch (err) {
      error = formatError(err);
    } finally {
      busy = false;
    }
  }
</script>

<section class="view">
  <header class="hero">
    <h1>Convert</h1>
    <p class="lede">Images, documents, and data — locally through the Rust pipeline.</p>
  </header>

  <form class="panel" onsubmit={onSubmit}>
    <JobForm
      bind:inputPath
      bind:outputPath
      bind:inputFormat
      bind:outputFormat
      bind:quality
      bind:width
      bind:height
      showImageOptions={true}
      inputFormats={INPUT_FORMATS}
      outputFormats={OUTPUT_FORMATS}
    />

    <div class="actions">
      <button type="submit" class="primary" disabled={busy}>
        {busy ? "Converting…" : "Convert"}
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
    transition: opacity var(--motion-fast) var(--motion-ease);
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
