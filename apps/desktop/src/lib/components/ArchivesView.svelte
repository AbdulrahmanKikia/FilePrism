<script lang="ts">
  import { formatError, isTauri, runJob } from "$lib/api";
  import { suggestDirBeside } from "$lib/dialogs";
  import {
    getShellAction,
    getShellPaths,
    getShellRevision,
  } from "$lib/shell.svelte";
  import JobForm from "./JobForm.svelte";

  let inputPath = $state("");
  let outputPath = $state("");
  let inputFormat = $state("zip");
  let outputFormat = $state("dir");
  let password = $state("");
  let busy = $state(false);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  const ARCHIVE_FILTERS = [
    { name: "Archives", extensions: ["zip", "tar", "7z", "rar", "tgz", "gz"] },
  ];

  function applyInputPath(path: string) {
    inputPath = path;
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    if (["zip", "tar", "7z", "rar"].includes(ext)) {
      inputFormat = ext;
    }
    if (!outputPath.trim()) {
      outputPath = suggestDirBeside(path);
    }
  }

  $effect(() => {
    const rev = getShellRevision();
    const action = getShellAction();
    const paths = getShellPaths();
    if (rev === 0 || paths.length === 0) return;
    if (action !== "extract") return;
    applyInputPath(paths[0] ?? "");
  });

  async function onSubmit(e: Event) {
    e.preventDefault();
    message = null;
    error = null;

    if (!inputPath.trim() || !outputPath.trim()) {
      error = "Archive and output folder paths are required.";
      return;
    }
    if (!isTauri()) {
      error = "Extraction requires the Tauri desktop shell (pnpm tauri dev).";
      return;
    }

    busy = true;
    try {
      const out = await runJob({
        inputPath: inputPath.trim(),
        outputPath: outputPath.trim(),
        inputFormat,
        outputFormat,
        options: password ? { password } : {},
      });
      message = `Extracted to ${out.output_path} (${out.bytes_written} bytes written).`;
    } catch (err) {
      error = formatError(err);
    } finally {
      busy = false;
    }
  }
</script>

<section class="view">
  <header class="hero">
    <h1>Archives</h1>
    <p class="lede">Extract ZIP, TAR, 7z, and RAR (RAR via UnRAR sidecar).</p>
  </header>

  <form class="panel" onsubmit={onSubmit}>
    <JobForm
      bind:inputPath
      bind:outputPath
      bind:inputFormat
      bind:outputFormat
      bind:password
      showPassword={true}
      inputFormats={["zip", "tar", "7z", "rar"]}
      outputFormats={["dir"]}
      inputLabel="Archive path"
      outputLabel="Extract folder"
      inputHint="Absolute path to the archive"
      outputHint="Destination directory for extracted files"
      inputPicker="file"
      outputPicker="directory"
      inputFilters={ARCHIVE_FILTERS}
      onInputPicked={applyInputPath}
    />

    <div class="actions">
      <button type="submit" class="primary" disabled={busy}>
        {busy ? "Extracting…" : "Extract"}
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
