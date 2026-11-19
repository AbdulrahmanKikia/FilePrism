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

  const STORAGE_KEY = "fileprism.sidebarCollapsed";

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

  $effect(() => {
    try {
      collapsed = localStorage.getItem(STORAGE_KEY) === "1";
    } catch {
      /* ignore */
    }
  });

  function toggleCollapsed() {
    collapsed = !collapsed;
    try {
      localStorage.setItem(STORAGE_KEY, collapsed ? "1" : "0");
    } catch {
      /* ignore */
    }
  }
</script>

<aside
  class="sidebar"
  class:collapsed
  aria-label="Primary"
  style:width={collapsed ? "var(--sidebar-width-collapsed)" : "var(--sidebar-width)"}
>
  <div class="brand">
    <span class="mark" aria-hidden="true"></span>
    <span class="name">FilePrism</span>
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
            d="M14.71 6.71a1 1 0 0 0-1.41 0L8.71 11.3a1 1 0 0 0 0 1.41l4.59 4.59a1 1 0 0 0 1.41-1.41L10.83 12l3.88-3.88a1 1 0 0 0 0-1.41z"
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
        title={collapsed ? item.label : undefined}
        onclick={() => onnavigate?.(item.id)}
      >
        <span class="icon" aria-hidden="true">
          {#if item.id === "home"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 3.2 4 10v10h5v-6h6v6h5V10l-8-6.8z"
              /></svg
            >
          {:else if item.id === "convert"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M7 7h11l-1.5-1.5L18 4l4 4-4 4-1.5-1.5L18 9H7V7zm10 10H6l1.5 1.5L6 20l-4-4 4-4 1.5 1.5L6 15h11v2z"
              /></svg
            >
          {:else if item.id === "compress"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M6 2h8l4 4v16H6V2zm8 1.5V7h3.5L14 3.5zM9 9h2v2H9V9zm0 3h2v2H9v-2zm0 3h2v2H9v-2zm3-6h2v2h-2V9zm0 3h2v2h-2v-2z"
              /></svg
            >
          {:else if item.id === "archives"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M4 4h16v4H4V4zm1 5h14v11H5V9zm5 2v2h4v-2h-4z"
              /></svg
            >
          {:else if item.id === "history"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 4a8 8 0 1 1-7.07 4.14L3 6.2V10h3.8L5.4 8.6A6 6 0 1 0 12 6v2l4-3-4-3v2zm-1 4h2v5l3.5 2.1-.9 1.5L11 14V8z"
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
                d="M12 5c5 0 9 5.5 9 7s-4 7-9 7-9-5.5-9-7 4-7 9-7zm0 2c-3.4 0-6.2 3.3-6.9 5 .7 1.7 3.5 5 6.9 5s6.2-3.3 6.9-5c-.7-1.7-3.5-5-6.9-5zm0 2.5A2.5 2.5 0 1 1 9.5 12 2.5 2.5 0 0 1 12 9.5z"
              /></svg
            >
          {:else if item.id === "settings"}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M19.14 12.94c.04-.31.06-.63.06-.94s-.02-.63-.06-.94l2.03-1.58a.5.5 0 0 0 .12-.64l-1.92-3.32a.5.5 0 0 0-.6-.22l-2.39.96a7.1 7.1 0 0 0-1.63-.94l-.36-2.54a.5.5 0 0 0-.5-.42h-3.84a.5.5 0 0 0-.5.42l-.36 2.54c-.59.24-1.13.55-1.63.94l-2.39-.96a.5.5 0 0 0-.6.22L2.77 8.84a.5.5 0 0 0 .12.64l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94L2.89 14.52a.5.5 0 0 0-.12.64l1.92 3.32c.13.23.4.32.64.22l2.39-.96c.5.39 1.04.7 1.63.94l.36 2.54c.05.24.25.42.5.42h3.84c.25 0 .45-.18.5-.42l.36-2.54c.59-.24 1.13-.55 1.63-.94l2.39.96c.24.1.51 0 .64-.22l1.92-3.32a.5.5 0 0 0-.12-.64l-2.03-1.58zM12 15.5A3.5 3.5 0 1 1 15.5 12 3.5 3.5 0 0 1 12 15.5z"
              /></svg
            >
          {:else}
            <svg viewBox="0 0 24 24" width="18" height="18"
              ><path
                fill="currentColor"
                d="M12 2 4 5v6c0 5 3.4 9.7 8 11 4.6-1.3 8-6 8-11V5l-8-3zm0 2.2 6 2.25V11c0 3.9-2.5 7.5-6 8.7-3.5-1.2-6-4.8-6-8.7V6.45l6-2.25zM11 8h2v5h-2V8zm0 6h2v2h-2v-2z"
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

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2);
    min-height: 36px;
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    flex-shrink: 0;
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-2));
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 40%, transparent);
  }

  .name {
    flex: 1;
    min-width: 0;
    font-size: var(--font-size-lg);
    font-weight: 650;
    letter-spacing: -0.02em;
    white-space: nowrap;
    overflow: hidden;
    opacity: 1;
    transition:
      opacity var(--motion-base) var(--motion-ease),
      max-width var(--motion-base) var(--motion-ease);
    max-width: 140px;
  }

  .collapse-btn {
    appearance: none;
    border: 0;
    background: color-mix(in srgb, var(--color-text) 8%, transparent);
    color: var(--color-muted);
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    display: grid;
    place-items: center;
    cursor: pointer;
    flex-shrink: 0;
    margin-left: auto;
    transition:
      background var(--motion-fast) var(--motion-ease),
      color var(--motion-fast) var(--motion-ease);
  }

  .collapse-btn:hover {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-text) 14%, transparent);
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

  .nav-item:hover {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-text) 6%, transparent);
  }

  .nav-item.active {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .label {
    white-space: nowrap;
    overflow: hidden;
    opacity: 1;
    max-width: 160px;
    transition:
      opacity var(--motion-base) var(--motion-ease),
      max-width var(--motion-base) var(--motion-ease);
  }

  .sidebar.collapsed {
    padding-left: var(--space-3);
    padding-right: var(--space-3);
  }

  .sidebar.collapsed .name {
    opacity: 0;
    max-width: 0;
  }

  .sidebar.collapsed .brand {
    justify-content: center;
    gap: 0;
  }

  .sidebar.collapsed .collapse-btn {
    margin-left: 0;
  }

  .sidebar.collapsed .mark {
    display: none;
  }

  .sidebar.collapsed .nav-item {
    justify-content: center;
    padding-left: var(--space-2);
    padding-right: var(--space-2);
  }

  .sidebar.collapsed .label {
    opacity: 0;
    max-width: 0;
  }
</style>
