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

  let collapsed = $state(false);

  function toggleCollapsed() {
    collapsed = !collapsed;
  }
</script>

<aside class="sidebar" class:collapsed aria-label="Primary">
  <div class="brand-row">
    {#if !collapsed}
      <div class="brand">
        <span class="mark" aria-hidden="true"></span>
        <span class="name">FilePrism</span>
      </div>
    {/if}
    <button
      type="button"
      class="collapse-btn"
      aria-expanded={!collapsed}
      aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      title={collapsed ? "Expand" : "Collapse"}
      onclick={toggleCollapsed}
    >
      <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
        {#if collapsed}
          <path
            fill="currentColor"
            d="M9.29 6.71a1 1 0 0 0 0 1.41L13.17 12l-3.88 3.88a1 1 0 1 0 1.41 1.41l4.59-4.59a1 1 0 0 0 0-1.41L10.7 6.7a1 1 0 0 0-1.41.01z"
          />
        {:else}
          <path
            fill="currentColor"
            d="M14.71 6.71a1 1 0 0 0-1.41 0L8.71 11.3a1 1 0 0 0 0 1.41l4.59 4.59a1 1 0 0 0 1.41-1.41L11.83 12l3.88-3.88a1 1 0 0 0 0-1.41z"
          />
        {/if}
      </svg>
    </button>
  </div>

  <nav class="nav">
    {#each items as item}
      <button
        type="button"
        class="nav-item"
        class:active={active === item.id}
        aria-current={active === item.id ? "page" : undefined}
        aria-label={item.label}
        title={item.label}
        onclick={() => onnavigate?.(item.id)}
      >
        <span class="icon" aria-hidden="true">
          {#if item.id === "home"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 3.2 3 11h2v9h6v-6h2v6h6v-9h2L12 3.2z"
              /></svg
            >
          {:else if item.id === "convert"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M7 7h7.2l-1.6-1.6L14 4l4 4-4 4-1.4-1.4L14.2 9H7v5H5V9a2 2 0 0 1 2-2zm10 10H9.8l1.6 1.6L10 20l-4-4 4-4 1.4 1.4L9.8 15H17v-5h2v5a2 2 0 0 1-2 2z"
              /></svg
            >
          {:else if item.id === "compress"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M9 3h6v2h-2v2h2v2h-2v2h2v2h-2v2h2v2h-6V3zm2 2v2h2V5h-2zm0 4v2h2V9h-2zm0 4v2h2v-2h-2z"
              /></svg
            >
          {:else if item.id === "archives"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M4 6.5 12 3l8 3.5v4.2c0 4.9-3.3 9.4-8 10.8-4.7-1.4-8-5.9-8-10.8V6.5zm2 .9v3.3c0 3.7 2.4 7.1 6 8.3 3.6-1.2 6-4.6 6-8.3V7.4L12 5.1 6 7.4z"
              /></svg
            >
          {:else if item.id === "history"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M13 3a9 9 0 1 0 8.9 7.5h-2.1A7 7 0 1 1 13 5v4.6l3.7 2.2-.9 1.5L11 10.2V3h2z"
              /></svg
            >
          {:else if item.id === "presets"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M4 6h16v2H4V6zm0 5h16v2H4v-2zm0 5h10v2H4v-2z"
              /></svg
            >
          {:else if item.id === "watched"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 5c5 0 9.3 3.1 11 7.5C21.3 16.9 17 20 12 20S2.7 16.9 1 12.5C2.7 8.1 7 5 12 5zm0 2C8.2 7 4.9 9.2 3.4 12.5 4.9 15.8 8.2 18 12 18s7.1-2.2 8.6-5.5C19.1 9.2 15.8 7 12 7zm0 2.5a3 3 0 1 1 0 6 3 3 0 0 1 0-6z"
              /></svg
            >
          {:else if item.id === "settings"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M19.1 12.9c0-.3 0-.6-.1-.9l1.8-1.4-1.7-3-2.1.7a6.5 6.5 0 0 0-1.6-.9l-.3-2.2h-3.4l-.3 2.2c-.6.2-1.1.5-1.6.9l-2.1-.7-1.7 3 1.8 1.4c0 .3-.1.6-.1.9s0 .6.1.9L4.5 15.3l1.7 3 2.1-.7c.5.4 1 .7 1.6.9l.3 2.2h3.4l.3-2.2c.6-.2 1.1-.5 1.6-.9l2.1.7 1.7-3-1.8-1.4c.1-.3.1-.6.1-.9zM12 15.5A3.5 3.5 0 1 1 12 8.5a3.5 3.5 0 0 1 0 7z"
              /></svg
            >
          {:else}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 2a5 5 0 0 1 5 5v1h1a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2h1V7a5 5 0 0 1 5-5zm0 2a3 3 0 0 0-3 3v1h6V7a3 3 0 0 0-3-3zm0 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"
              /></svg
            >
          {/if}
        </span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    --rail: var(--sidebar-width);
    width: var(--rail);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
    border-right: 1px solid var(--color-border);
    overflow: hidden;
    transition: width var(--motion-base) var(--motion-ease);
  }

  .sidebar.collapsed {
    --rail: var(--sidebar-width-collapsed);
    padding-left: var(--space-2);
    padding-right: var(--space-2);
  }

  .brand-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 36px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2);
    min-width: 0;
    flex: 1;
  }

  .mark {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    border-radius: 7px;
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 40%, transparent);
  }

  .name {
    font-size: var(--font-size-lg);
    font-weight: 650;
    letter-spacing: -0.02em;
    white-space: nowrap;
  }

  .collapse-btn {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--color-muted);
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    display: grid;
    place-items: center;
    cursor: pointer;
    flex-shrink: 0;
    margin-left: auto;
    transition:
      background var(--motion-fast) var(--motion-ease),
      color var(--motion-fast) var(--motion-ease);
  }

  .collapsed .brand-row {
    justify-content: center;
  }

  .collapsed .collapse-btn {
    margin-left: 0;
  }

  .collapse-btn:hover {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-text) 6%, transparent);
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
    padding: var(--space-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 40px;
    transition:
      background var(--motion-fast) var(--motion-ease),
      color var(--motion-fast) var(--motion-ease);
  }

  .collapsed .nav-item {
    justify-content: center;
    padding-inline: var(--space-2);
  }

  .icon {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 18px;
    height: 18px;
  }

  .label {
    white-space: nowrap;
    overflow: hidden;
    opacity: 1;
    max-width: 160px;
    transition:
      opacity var(--motion-fast) var(--motion-ease),
      max-width var(--motion-base) var(--motion-ease);
  }

  .collapsed .label {
    opacity: 0;
    max-width: 0;
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
