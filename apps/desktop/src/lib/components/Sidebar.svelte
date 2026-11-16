<script lang="ts">
  export type NavId =
    | "home"
    | "convert"
    | "compress"
    | "archives"
    | "history"
    | "presets"
    | "watched"
    | "settings"
    | "license";

  type NavItem = { id: NavId; label: string };

  const items: NavItem[] = [
    { id: "home", label: "Home" },
    { id: "convert", label: "Convert" },
    { id: "compress", label: "Compress" },
    { id: "archives", label: "Archives" },
    { id: "history", label: "History" },
    { id: "presets", label: "Presets" },
    { id: "watched", label: "Watched folders" },
    { id: "settings", label: "Settings" },
    { id: "license", label: "License" },
  ];

  let {
    active = "home",
    onnavigate,
  }: {
    active?: NavId;
    onnavigate?: (id: NavId) => void;
  } = $props();
</script>

<aside class="sidebar" aria-label="Primary">
  <div class="brand">
    <span class="mark" aria-hidden="true"></span>
    <span class="name">FilePrism</span>
  </div>

  <nav class="nav">
    {#each items as item}
      <button
        type="button"
        class="nav-item"
        class:active={active === item.id}
        aria-current={active === item.id ? "page" : undefined}
        onclick={() => onnavigate?.(item.id)}
      >
        {item.label}
      </button>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
    border-right: 1px solid var(--color-border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-2);
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 40%, transparent);
  }

  .name {
    font-size: var(--font-size-lg);
    font-weight: 650;
    letter-spacing: -0.02em;
  }

  .nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--color-muted);
    text-align: left;
    padding: var(--space-3) var(--space-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background var(--motion-fast) var(--motion-ease),
      color var(--motion-fast) var(--motion-ease);
  }

  .nav-item:hover {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-text) 6%, transparent);
  }

  .nav-item.active {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
  }
</style>
