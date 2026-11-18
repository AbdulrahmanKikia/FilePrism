<script lang="ts">
  import {
    formatError,
    isTauri,
    rerunJob,
    searchHistory,
    type JobRecord,
  } from "$lib/api";

  let query = $state("");
  let statusFilter = $state("all");
  let jobs = $state<JobRecord[]>([]);
  let busy = $state(false);
  let rerunning = $state<string | null>(null);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  const filtered = $derived(
    statusFilter === "all"
      ? jobs
      : jobs.filter((j) => j.status === statusFilter),
  );

  async function refresh() {
    message = null;
    error = null;
    if (!isTauri()) {
      error = "History requires the Tauri desktop shell.";
      jobs = [];
      return;
    }
    busy = true;
    try {
      jobs = await searchHistory(query.trim(), 200);
    } catch (err) {
      error = formatError(err);
      jobs = [];
    } finally {
      busy = false;
    }
  }

  async function onRerun(id: string) {
    message = null;
    error = null;
    rerunning = id;
    try {
      const out = await rerunJob(id);
      message = `Re-ran job → ${out.job_id} (${out.bytes_written} bytes at ${out.output_path}).`;
      await refresh();
    } catch (err) {
      error = formatError(err);
    } finally {
      rerunning = null;
    }
  }

  function canRerun(job: JobRecord): boolean {
    return job.status !== "running" && job.status !== "pending";
  }

  function basename(path: string): string {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  }

  $effect(() => {
    void refresh();
  });
</script>

<section class="view">
  <header class="hero">
    <h1>History</h1>
    <p class="lede">Search past jobs and re-run them with the same paths.</p>
  </header>

  <div class="toolbar">
    <input
      type="search"
      placeholder="Search filename, format, status…"
      bind:value={query}
      onkeydown={(e) => e.key === "Enter" && refresh()}
    />
    <select bind:value={statusFilter} aria-label="Filter by status">
      <option value="all">All statuses</option>
      <option value="completed">Completed</option>
      <option value="failed">Failed</option>
      <option value="running">Running</option>
      <option value="cancelled">Cancelled</option>
      <option value="pending">Pending</option>
    </select>
    <button type="button" class="ghost" onclick={refresh} disabled={busy}>
      {busy ? "Loading…" : "Refresh"}
    </button>
  </div>

  {#if message}
    <p class="ok" role="status">{message}</p>
  {/if}
  {#if error}
    <p class="err" role="alert">{error}</p>
  {/if}

  <div class="list" role="list">
    {#if filtered.length === 0}
      <p class="empty">No jobs yet. Run a conversion to populate history.</p>
    {:else}
      {#each filtered as job (job.id)}
        <article class="row" role="listitem">
          <div class="meta">
            <div class="title">
              <span class="name">{basename(job.input_path)}</span>
              <span class="arrow" aria-hidden="true">→</span>
              <span class="fmt">{job.input_format} → {job.output_format}</span>
            </div>
            <div class="sub">
              <span class="status" data-status={job.status}>{job.status}</span>
              <span class="time">{job.created_at}</span>
              <span class="id">{job.id}</span>
            </div>
            {#if job.error_message}
              <p class="job-err">{job.error_message}</p>
            {/if}
          </div>
          <button
            type="button"
            class="primary"
            disabled={rerunning === job.id || !canRerun(job)}
            title={canRerun(job) ? "Run again with the same paths and formats" : "Job is still in progress"}
            onclick={() => onRerun(job.id)}
          >
            {rerunning === job.id ? "Re-running…" : "Re-run"}
          </button>
        </article>
      {/each}
    {/if}
  </div>
</section>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
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

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  input,
  select {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    color: var(--color-text);
    padding: var(--space-3);
    min-width: 0;
  }

  input[type="search"] {
    flex: 1;
    min-width: 180px;
  }

  .ghost,
  .primary {
    appearance: none;
    border: 0;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    font-weight: 600;
    cursor: pointer;
  }

  .ghost {
    background: color-mix(in srgb, var(--color-text) 8%, transparent);
    color: var(--color-text);
  }

  .primary {
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    color: #fff;
    flex-shrink: 0;
  }

  .primary:disabled,
  .ghost:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    animation: view-enter var(--motion-fast) var(--motion-ease);
  }

  .meta {
    flex: 1;
    min-width: 0;
  }

  .title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2);
  }

  .name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .arrow,
  .fmt {
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .sub {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--color-muted);
  }

  .status {
    text-transform: capitalize;
    color: var(--color-text);
  }

  .status[data-status="completed"] {
    color: var(--color-good);
  }
  .status[data-status="failed"] {
    color: var(--color-bad);
  }
  .status[data-status="running"] {
    color: var(--color-warn);
  }

  .job-err {
    margin: var(--space-2) 0 0;
    color: var(--color-bad);
    font-size: var(--font-size-sm);
  }

  .empty {
    margin: 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
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
