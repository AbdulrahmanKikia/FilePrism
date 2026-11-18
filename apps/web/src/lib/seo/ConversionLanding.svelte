<script lang="ts">
  import AdSlot from "./AdSlot.svelte";
  import {
    converterHref,
    pairDescription,
    pairTitle,
    relatedPairs,
    type ConversionPair,
  } from "./pairs";

  let { pair }: { pair: ConversionPair } = $props();

  const related = $derived(relatedPairs(pair));
  const href = $derived(converterHref(pair));
  const title = $derived(pairTitle(pair));
</script>

<article class="landing">
  <header class="hero">
    <p class="brand">FilePrism</p>
    <h1>{pair.fromLabel} to {pair.toLabel}</h1>
    <p class="lede">{pair.blurb}</p>

    <div class="cta-row">
      {#if pair.webReady}
        <a class="cta primary" href={href}>Convert in browser — free</a>
      {:else}
        <a class="cta primary" href={href}>Open converter</a>
        <p class="desktop-note">
          This pair is desktop-first. Web may show a clear “desktop only” state —
          no fake success.
        </p>
      {/if}
    </div>

    <ul class="trust">
      <li>Free &amp; ad-supported on the web</li>
      <li>Client-side — files stay on your device</li>
      <li>No account required</li>
    </ul>
  </header>

  <div class="ad-rail" aria-hidden="false">
    <AdSlot slot={`landing-hero-${pair.slug}`} size="leaderboard" />
  </div>

  <section class="body">
    <div class="copy">
      <h2>How it works</h2>
      <ol>
        <li>Open the free browser converter for {pair.fromLabel}→{pair.toLabel}.</li>
        <li>Pick a file — conversion runs locally via WebAssembly when available.</li>
        <li>Download the result. Nothing is uploaded to FilePrism servers.</li>
      </ol>

      <h2>Web vs desktop</h2>
      <p>
        FilePrism Web is the free, ad-supported companion for quick one-off
        conversions. It does <strong>not</strong> replace desktop FilePrism —
        which stays offline-first with batch queues, Explorer integration, large
        files, and no ads.
      </p>

      {#if !pair.webReady}
        <p class="callout">
          Prefer desktop for {pair.fromLabel}→{pair.toLabel}: richer format
          support and offline batch without advertising.
        </p>
      {/if}
    </div>

    <aside class="side">
      <AdSlot slot={`landing-side-${pair.slug}`} size="rectangle" />
    </aside>
  </section>

  {#if related.length}
    <section class="related">
      <h2>Related conversions</h2>
      <ul>
        {#each related as rel}
          <li>
            <a href={`/${rel.slug}`}>{rel.fromLabel} → {rel.toLabel}</a>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <footer class="seo-foot">
    <p>{pairDescription(pair)}</p>
    <p class="muted">
      Looking for batch, offline, or Explorer right-click? Use
      <span class="desktop-name">FilePrism desktop</span> — one-time purchase, no
      ads.
    </p>
  </footer>
</article>

<!-- SEO title used by +page via svelte:head in the route -->
{#if false}{title}{/if}

<style>
  .landing {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .hero {
    position: relative;
    padding: clamp(2rem, 5vw, 3.5rem) 0 1rem;
    background:
      radial-gradient(120% 80% at 10% 0%, color-mix(in srgb, var(--color-accent) 22%, transparent), transparent 55%),
      radial-gradient(90% 60% at 90% 10%, color-mix(in srgb, var(--color-accent-2) 18%, transparent), transparent 50%);
  }

  .brand {
    margin: 0 0 0.5rem;
    font-family: var(--font-display);
    font-size: clamp(1.75rem, 4vw, 2.35rem);
    font-weight: 700;
    letter-spacing: -0.04em;
    color: var(--color-text);
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: clamp(1.35rem, 3vw, 1.85rem);
    font-weight: 650;
    letter-spacing: -0.03em;
  }

  .lede {
    margin: 0.75rem 0 0;
    max-width: 38rem;
    font-size: 1.05rem;
    color: var(--color-muted);
    line-height: 1.5;
  }

  .cta-row {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .cta {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1.25rem;
    border-radius: var(--radius-md);
    font-family: var(--font-display);
    font-weight: 600;
    text-decoration: none;
    transition: transform var(--motion-fast) var(--motion-ease),
      background var(--motion-fast) var(--motion-ease);
  }

  .cta.primary {
    background: var(--color-accent);
    color: #041016;
  }

  .cta.primary:hover {
    background: var(--color-accent-2);
    color: #fff;
    transform: translateY(-1px);
  }

  .desktop-note {
    margin: 0;
    max-width: 32rem;
    font-size: var(--font-size-sm);
    color: var(--color-muted);
  }

  .trust {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1.25rem;
    margin: 1.5rem 0 0;
    padding: 0;
    list-style: none;
    font-size: var(--font-size-sm);
    color: var(--color-muted);
  }

  .trust li::before {
    content: "·";
    margin-right: 0.5rem;
    color: var(--color-accent);
  }

  .ad-rail {
    width: 100%;
  }

  .body {
    display: grid;
    gap: var(--space-5);
    grid-template-columns: 1fr;
  }

  @media (min-width: 900px) {
    .body {
      grid-template-columns: minmax(0, 1fr) 300px;
      align-items: start;
    }
  }

  .copy h2 {
    margin: 0 0 0.75rem;
    font-family: var(--font-display);
    font-size: 1.15rem;
  }

  .copy h2 + h2,
  .copy p + h2 {
    margin-top: 1.75rem;
  }

  .copy ol {
    margin: 0;
    padding-left: 1.25rem;
    color: var(--color-muted);
    line-height: 1.6;
  }

  .copy p {
    margin: 0;
    color: var(--color-muted);
    line-height: 1.6;
  }

  .callout {
    margin-top: 1rem !important;
    padding: 0.9rem 1rem;
    border-left: 3px solid var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
    color: var(--color-text) !important;
  }

  .related h2 {
    margin: 0 0 0.75rem;
    font-family: var(--font-display);
    font-size: 1.15rem;
  }

  .related ul {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .related a {
    color: var(--color-accent);
    text-decoration: none;
    font-weight: 500;
  }

  .related a:hover {
    text-decoration: underline;
  }

  .seo-foot {
    padding-top: var(--space-5);
    border-top: 1px solid var(--color-border);
    font-size: var(--font-size-sm);
    color: var(--color-muted);
    line-height: 1.55;
  }

  .seo-foot p {
    margin: 0 0 0.75rem;
  }

  .muted {
    color: var(--color-muted);
  }

  .desktop-name {
    color: var(--color-text);
    font-weight: 600;
  }
</style>
