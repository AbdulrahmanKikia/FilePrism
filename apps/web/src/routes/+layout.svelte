<script lang="ts">
  import "$lib/styles/web.css";
  import AdSlot from "$lib/seo/AdSlot.svelte";

  let { children } = $props();
</script>

<div class="shell">
  <header class="top">
    <a class="logo" href="/" aria-label="FilePrism home">FilePrism</a>
    <nav class="nav" aria-label="Primary">
      <a href="/convert">Converter</a>
      <a href="/png-to-webp">PNG→WebP</a>
      <a href="/jpg-to-webp">JPG→WebP</a>
    </nav>
    <p class="badge" title="Web positioning per product spec">Free · ad-supported</p>
  </header>

  <div class="top-ad">
    <AdSlot slot="site-top-leaderboard" size="leaderboard" />
  </div>

  <main class="main">
    {@render children()}
  </main>

  <footer class="foot">
    <p>
      <strong>FilePrism Web</strong> is free and ad-supported. Conversions run in
      your browser when WASM supports the format.
    </p>
    <p class="diff">
      Desktop FilePrism stays the full product: offline, batch, Explorer
      integration, large files, and no ads — web does not replace it.
    </p>
  </footer>
</div>

<style>
  .shell {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    max-width: var(--page-max);
    margin: 0 auto;
    padding: 1rem clamp(1rem, 3vw, 1.75rem) 2.5rem;
  }

  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem 1.25rem;
    margin-bottom: 1rem;
  }

  .logo {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 1.35rem;
    letter-spacing: -0.04em;
    text-decoration: none;
  }

  .nav {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    flex: 1;
  }

  .nav a {
    font-size: var(--font-size-sm);
    color: var(--color-muted);
    text-decoration: none;
  }

  .nav a:hover {
    color: var(--color-accent);
  }

  .badge {
    margin: 0;
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-accent-2);
    border: 1px solid color-mix(in srgb, var(--color-accent) 35%, transparent);
    padding: 0.3rem 0.55rem;
    border-radius: 999px;
  }

  .top-ad {
    margin-bottom: 1.25rem;
  }

  .main {
    flex: 1;
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

  .foot {
    margin-top: 2.5rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--color-border);
    font-size: var(--font-size-sm);
    color: var(--color-muted);
    line-height: 1.55;
  }

  .foot p {
    margin: 0 0 0.6rem;
  }

  .diff {
    max-width: 42rem;
  }
</style>
