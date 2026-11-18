<script lang="ts">
  export type NavId = "home" | "convert" | "history" | "settings";

  type NavItem = { id: NavId; label: string; href: string };

  const items: NavItem[] = [
    { id: "home", label: "Home", href: "/" },
    { id: "convert", label: "Convert", href: "/convert" },
    { id: "history", label: "History", href: "/history" },
    { id: "settings", label: "Settings", href: "/settings" },
  ];

  let {
    active = "home",
  }: {
    active?: NavId;
  } = $props();
</script>

<aside class="sidebar" aria-label="Primary">
  <div class="brand">
    <span class="mark" aria-hidden="true"></span>
    <span class="name">FilePrism</span>
  </div>

  <nav class="nav">
    {#each items as item}
      <a
        class="nav-item"
        class:active={active === item.id}
        href={item.href}
        aria-current={active === item.id ? "page" : undefined}
      >
        {item.label}
      </a>
    {/each}
  </nav>

  <p class="note">Client-side convert · files stay in your browser</p>
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
    padding: var(--space-2);
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
    color: var(--color-muted);
    text-decoration: none;
    padding: var(--space-3);
    border-radius: var(--radius-md);
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

  .note {
    margin: auto 0 0;
    font-size: var(--font-size-sm);
    color: var(--color-muted);
    line-height: 1.4;
  }

  @media (max-width: 720px) {
    .sidebar {
      width: 100%;
      border-right: 0;
      border-bottom: 1px solid var(--color-border);
    }

    .nav {
      flex-direction: row;
      flex-wrap: wrap;
    }

    .note {
      display: none;
    }
  }
</style>
