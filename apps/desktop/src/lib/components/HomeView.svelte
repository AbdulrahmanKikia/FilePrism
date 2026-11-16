<script lang="ts">
  let dragOver = $state(false);

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
  }
</script>

<section class="home">
  <header class="hero">
    <h1>Home</h1>
    <p class="lede">Convert, compress, and organize files — locally, privately.</p>
  </header>

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
    <p class="drop-hint">or choose a conversion from the sidebar</p>
  </div>

  <div class="panels">
    <section class="panel">
      <h2>Active jobs</h2>
      <p class="empty">No jobs running.</p>
    </section>
    <section class="panel">
      <h2>Recent history</h2>
      <p class="empty">History appears after your first conversion.</p>
    </section>
  </div>
</section>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    animation: enter var(--motion-base) var(--motion-ease);
  }

  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: none;
    }
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

  .empty {
    margin: 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }
</style>
