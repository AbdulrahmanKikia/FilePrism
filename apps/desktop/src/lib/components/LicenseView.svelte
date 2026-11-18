<script lang="ts">
  import {
    activateLicense,
    deactivateLicense,
    formatError,
    getLicenseStatus,
    isTauri,
    type LicenseStatus,
  } from "$lib/api";

  let status = $state<LicenseStatus | null>(null);
  let keyInput = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);

  async function refresh() {
    if (!isTauri()) {
      status = {
        entitlement: {
          kind: { type: "trial", daysRemaining: 14, endsAt: 0 },
          fullFeature: true,
          trialDays: 14,
        },
        hasStoredKey: false,
        checkoutUrl: "https://checkout.paddle.com/checkout/custom/fileprism-placeholder",
        trialDays: 14,
        maskedKey: null,
      };
      return;
    }
    status = await getLicenseStatus();
  }

  $effect(() => {
    let cancelled = false;
    (async () => {
      try {
        await refresh();
      } catch (e) {
        if (!cancelled) error = formatError(e);
      }
    })();
    return () => {
      cancelled = true;
    };
  });

  async function onActivate() {
    error = null;
    message = null;
    busy = true;
    try {
      status = await activateLicense(keyInput);
      keyInput = "";
      message = "License activated.";
    } catch (e) {
      error = formatError(e);
    } finally {
      busy = false;
    }
  }

  async function onDeactivate() {
    error = null;
    message = null;
    busy = true;
    try {
      status = await deactivateLicense();
      message = "License removed from this device.";
    } catch (e) {
      error = formatError(e);
    } finally {
      busy = false;
    }
  }

  const headline = $derived.by(() => {
    const kind = status?.entitlement.kind;
    if (!kind) return "Checking license…";
    switch (kind.type) {
      case "licensed":
        return kind.updatesActive
          ? "Licensed — updates active"
          : "Licensed — update window ended";
      case "trial":
        return `Trial — ${kind.daysRemaining} day${kind.daysRemaining === 1 ? "" : "s"} left`;
      case "trialExpired":
        return "Trial ended";
      case "notStarted":
        return "Starting trial…";
      default:
        return "License";
    }
  });

  const detail = $derived.by(() => {
    const kind = status?.entitlement.kind;
    if (!kind) return "";
    if (kind.type === "licensed") {
      const email = kind.email ? ` · ${kind.email}` : "";
      return `License ${kind.lid}${email}`;
    }
    if (kind.type === "trial") {
      return `Full-feature trial (${status?.trialDays ?? 14} days). Buy a key anytime — $19 one-time.`;
    }
    if (kind.type === "trialExpired") {
      return "Enter a license key to keep using FilePrism, or purchase below.";
    }
    return "";
  });
</script>

<section class="view">
  <header class="hero">
    <h1>License</h1>
    <p class="lede">Offline Ed25519 keys · 14-day full-feature trial · one-time purchase.</p>
  </header>

  <div class="panel status">
    <p class="status-title">{headline}</p>
    <p class="status-detail">{detail}</p>
    {#if status?.maskedKey}
      <p class="masked">Stored key: <code>{status.maskedKey}</code></p>
    {/if}
  </div>

  <div class="panel activate">
    <label class="field">
      <span>License key</span>
      <input
        type="text"
        placeholder="FP1.….…"
        autocomplete="off"
        spellcheck="false"
        bind:value={keyInput}
        disabled={busy}
      />
    </label>
    <div class="actions">
      <button type="button" class="primary" disabled={busy || !keyInput.trim()} onclick={onActivate}>
        Activate
      </button>
      {#if status?.hasStoredKey}
        <button type="button" class="ghost" disabled={busy} onclick={onDeactivate}>
          Remove key
        </button>
      {/if}
      {#if status?.checkoutUrl}
        <a class="buy" href={status.checkoutUrl} target="_blank" rel="noreferrer">
          Buy license (Paddle)
        </a>
      {/if}
    </div>
    {#if error}
      <p class="err" role="alert">{error}</p>
    {/if}
    {#if message}
      <p class="ok" role="status">{message}</p>
    {/if}
  </div>
</section>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    animation: view-enter var(--motion-base) var(--motion-ease);
    max-width: 640px;
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

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    background: color-mix(in srgb, var(--color-surface) calc(var(--opacity-glass) * 100%), transparent);
    backdrop-filter: blur(var(--blur-glass));
  }

  .status-title {
    margin: 0;
    font-weight: 650;
    font-size: 1.125rem;
  }

  .status-detail,
  .masked {
    margin: var(--space-2) 0 0;
    color: var(--color-muted);
    font-size: var(--font-size-sm);
  }

  .masked code {
    font-family: ui-monospace, "Cascadia Code", monospace;
    color: var(--color-text);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field span {
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  input {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text);
    padding: var(--space-3);
    font-family: ui-monospace, "Cascadia Code", monospace;
    font-size: var(--font-size-sm);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-4);
  }

  button {
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-4);
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.primary {
    background: var(--color-accent);
    border-color: transparent;
    color: #041210;
  }

  button.ghost {
    background: transparent;
  }

  .buy {
    color: var(--color-accent);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  .err {
    margin: var(--space-3) 0 0;
    color: var(--color-bad);
    font-size: var(--font-size-sm);
  }

  .ok {
    margin: var(--space-3) 0 0;
    color: var(--color-good);
    font-size: var(--font-size-sm);
  }
</style>
