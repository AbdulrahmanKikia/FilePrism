# Licensing (M7)

Offline-verifiable **Ed25519** license keys plus a **14-day full-feature trial**.

| Piece | Location |
|-------|----------|
| Validation crate | `crates/licensing` (`fileprism-licensing`) |
| Desktop commands | `apps/desktop/src-tauri/src/license.rs` |
| UI | `apps/desktop/src/lib/components/LicenseView.svelte` |
| Merchant stubs | [merchant.md](./merchant.md) |
| Contract | `docs/CONTRACTS.md` → License / entitlement |

## Key format

```
FP1.<base64url(payload_json)>.<base64url(ed25519_signature)>
```

Payload (JSON, signed as raw UTF-8 bytes):

| Field | Meaning |
|-------|---------|
| `v` | Schema version (`1`) |
| `lid` | License id |
| `email` | Optional purchaser email |
| `issued` | Unix seconds |
| `updates_until` | Unix seconds (typically issue + 1 year of updates) |

## Public / signing keys

- **App embeds** verifying key `EMBEDDED_PUBLIC_KEY_HEX` (override: `FILEPRISM_LICENSE_PUBKEY`).
- **Signing seed** lives only in secrets: `FILEPRISM_LICENSE_SIGNING_SEED` (64 hex chars). Never commit it.
- Optional mint API: `fileprism-licensing` feature `mint`.

## Trial

- Duration: **14 days** (`TRIAL_DAYS`) from first launch.
- Persisted as setting `license.trial_started_at`.
- Activated key stored as `license.key`.
- After trial expiry without a key, `fullFeature` is false (UI still opens; gating can tighten later).

## Checkout

- Default placeholder Paddle URL in code; override with `FILEPRISM_CHECKOUT_URL`.
