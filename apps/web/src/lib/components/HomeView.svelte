<script lang="ts">
  import { listHistory } from "$lib/history";

  const recent = $derived(listHistory().slice(0, 5));
</script>

<section class="home">
  <header class="hero">
    <h1>FilePrism Web</h1>
    <p class="lede">Convert images in your browser — nothing uploads to a server.</p>
  </header>

  <a class="cta" href="/convert">Open Convert</a>

  <div class="panels">
    <section class="panel">
      <h2>Supported now</h2>
      <p class="empty">PNG ↔ JPG ↔ WebP via @fileprism/wasm (browser).</p>
    </section>
    <section class="panel">
      <h2>Recent history</h2>
      {#if recent.length === 0}
        <p class="empty">History appears after your first conversion.</p>
      {:else}
        <ul class="recent">
          {#each recent as job (job.id)}
            <li>{job.inputName}: {job.inputFormat} → {job.outputFormat}</li>
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

  .cta {
    align-self: flex-start;
    text-decoration: none;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-5);
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    color: #fff;
    font-weight: 600;
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

  .empty,
  .recent {
    margin: 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .recent {
    padding-left: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
</style>
