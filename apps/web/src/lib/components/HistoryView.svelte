<script lang="ts">
  import { clearHistory, listHistory, type WebJobRecord } from "$lib/history";

  let query = $state("");
  let statusFilter = $state("all");
  let jobs = $state<WebJobRecord[]>([]);

  const filtered = $derived(
    statusFilter === "all" ? jobs : jobs.filter((j) => j.status === statusFilter),
  );

  function refresh() {
    jobs = listHistory(query);
  }

  function onClear() {
    clearHistory();
    refresh();
  }

  $effect(() => {
    void query;
    refresh();
  });
</script>

<section class="view">
  <header class="hero">
    <h1>History</h1>
    <p class="lede">Recent browser conversions stored in localStorage.</p>
  </header>

  <div class="toolbar">
    <input type="search" placeholder="Search filename, format, status…" bind:value={query} />
    <select bind:value={statusFilter} aria-label="Filter by status">
      <option value="all">All statuses</option>
      <option value="completed">Completed</option>
      <option value="failed">Failed</option>
    </select>
    <button type="button" class="ghost" onclick={refresh}>Refresh</button>
    <button type="button" class="ghost" onclick={onClear}>Clear</button>
  </div>

  <div class="list" role="list">
    {#if filtered.length === 0}
      <p class="empty">No jobs yet. Convert an image to populate history.</p>
    {:else}
      {#each filtered as job (job.id)}
        <article class="row" role="listitem">
          <div class="meta">
            <div class="title">
              <span class="name">{job.inputName}</span>
              <span class="arrow" aria-hidden="true">→</span>
              <span class="fmt">{job.inputFormat} → {job.outputFormat}</span>
            </div>
            <div class="sub">
              <span class="status" data-status={job.status}>{job.status}</span>
              <span class="time">{job.createdAt}</span>
              {#if job.engine}
                <span>{job.engine}</span>
              {/if}
              {#if job.bytesWritten != null}
                <span>{job.bytesWritten.toLocaleString()} B</span>
              {/if}
            </div>
            {#if job.errorMessage}
              <p class="job-err">{job.errorMessage}</p>
            {/if}
          </div>
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

  .ghost {
    appearance: none;
    border: 0;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    font-weight: 600;
    cursor: pointer;
    background: color-mix(in srgb, var(--color-text) 8%, transparent);
    color: var(--color-text);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .row {
    padding: var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
  }

  .title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2);
  }

  .name {
    font-weight: 600;
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

  .status[data-status="completed"] {
    color: var(--color-good);
  }
  .status[data-status="failed"] {
    color: var(--color-bad);
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
</style>
