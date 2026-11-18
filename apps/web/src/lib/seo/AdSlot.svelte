<script lang="ts">
  /**
   * Placeholder ad slot — no live ad network wired in M8.
   * Lane B/C can later swap `data-ad-slot` for a real network snippet.
   */
  type SlotSize = "leaderboard" | "rectangle" | "sidebar";

  let {
    slot = "web-default",
    size = "rectangle" as SlotSize,
    label = "Advertisement",
  }: {
    slot?: string;
    size?: SlotSize;
    label?: string;
  } = $props();
</script>

<aside
  class="ad-slot"
  class:leaderboard={size === "leaderboard"}
  class:rectangle={size === "rectangle"}
  class:sidebar={size === "sidebar"}
  data-ad-slot={slot}
  data-ad-network="placeholder"
  aria-label={label}
>
  <span class="ad-label">{label}</span>
  <div class="ad-frame" role="presentation">
    <span class="ad-copy">Ad slot · {slot}</span>
    <span class="ad-hint">Placeholder — free web is ad-supported</span>
  </div>
</aside>

<style>
  .ad-slot {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    width: 100%;
  }

  .ad-label {
    font-family: var(--font-sans);
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-muted);
  }

  .ad-frame {
    display: grid;
    place-content: center;
    gap: 0.25rem;
    text-align: center;
    border: 1px dashed color-mix(in srgb, var(--color-border) 80%, transparent);
    background:
      linear-gradient(
        135deg,
        color-mix(in srgb, var(--color-surface) 88%, transparent),
        color-mix(in srgb, var(--color-accent) 6%, var(--color-surface))
      );
    color: var(--color-muted);
  }

  .leaderboard .ad-frame {
    min-height: 90px;
  }

  .rectangle .ad-frame {
    min-height: 250px;
    max-width: 300px;
  }

  .sidebar .ad-frame {
    min-height: 600px;
    max-width: 160px;
  }

  .ad-copy {
    font-family: var(--font-display);
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .ad-hint {
    font-size: 0.75rem;
  }
</style>
