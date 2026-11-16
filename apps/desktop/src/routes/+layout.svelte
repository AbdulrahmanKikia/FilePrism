<script lang="ts">
  import "$lib/styles/tokens.css";
  import Sidebar, { type NavId } from "$lib/components/Sidebar.svelte";

  let { children } = $props();
  let active = $state<NavId>("home");

  const labels: Record<NavId, string> = {
    home: "Home",
    convert: "Convert",
    compress: "Compress",
    archives: "Archives",
    history: "History",
    presets: "Presets",
    watched: "Watched folders",
    settings: "Settings",
    license: "License",
  };

  function onNavigate(id: NavId) {
    active = id;
  }
</script>

<div class="shell" data-theme="dark" data-accent="teal" data-glass="full">
  <Sidebar {active} onnavigate={onNavigate} />
  <main class="main">
    {#if active === "home"}
      {@render children()}
    {:else}
      <section class="placeholder">
        <h1>{labels[active]}</h1>
        <p>Coming in a later milestone.</p>
      </section>
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    min-height: 100vh;
    min-height: 100dvh;
  }

  .main {
    flex: 1;
    min-width: 0;
    padding: var(--space-5) var(--space-6);
    overflow: auto;
  }

  .placeholder h1 {
    margin: 0;
    font-size: 1.5rem;
    letter-spacing: -0.02em;
  }

  .placeholder p {
    margin: var(--space-2) 0 0;
    color: var(--color-muted);
  }
</style>
