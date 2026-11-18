# Merchant of record (M7 stubs)

**Primary:** [Paddle](https://www.paddle.com/)  
**Alternate:** [Lemon Squeezy](https://www.lemonsqueezy.com/)

No real API keys, webhook secrets, or vendor IDs are committed. Use environment / CI secrets and local `.env` (gitignored).

## Product model (spec §7)

| Item | Value |
|------|-------|
| Price | $19 USD one-time |
| Updates | 1 year included; optional paid renewal later |
| Trial | 14-day full-feature (app-side; MoR may also offer a checkout trial) |
| Fulfillment | MoR webhook → mint Ed25519 key → email to buyer |

## Primary — Paddle

### Env / config placeholders

| Variable | Purpose |
|----------|---------|
| `PADDLE_API_KEY` | Server-side API (fulfillment worker) |
| `PADDLE_WEBHOOK_SECRET` | Verify webhook signatures |
| `PADDLE_PRICE_ID` | $19 product/price id |
| `PADDLE_CLIENT_TOKEN` | Client-side overlay token (if using Paddle.js) |
| `FILEPRISM_CHECKOUT_URL` | Desktop “Buy license” link (overlay or hosted checkout) |

### Stub flow

1. Desktop opens `FILEPRISM_CHECKOUT_URL` (see License UI).
2. After payment, Paddle sends `transaction.completed` (or legacy `payment_succeeded`) to your webhook.
3. Worker verifies signature with `PADDLE_WEBHOOK_SECRET`.
4. Worker mints `FP1.…` via `FILEPRISM_LICENSE_SIGNING_SEED` and emails the key.
5. Buyer pastes key into FilePrism → Settings / License → Activate.

### Local stub

```bash
# PowerShell
$env:FILEPRISM_CHECKOUT_URL = "https://checkout.paddle.com/checkout/custom/YOUR_PRICE"
# Do not set real PADDLE_* keys in the repo; use a secrets manager / CI.
```

## Alternate — Lemon Squeezy

Use when Paddle is unavailable in a region or you prefer LS checkout.

| Variable | Purpose |
|----------|---------|
| `LEMONSQUEEZY_API_KEY` | REST API |
| `LEMONSQUEEZY_WEBHOOK_SECRET` | Webhook HMAC |
| `LEMONSQUEEZY_STORE_ID` | Store id |
| `LEMONSQUEEZY_VARIANT_ID` | $19 variant |
| `FILEPRISM_CHECKOUT_URL` | Point at the LS product checkout URL instead |

Fulfillment is the same mint-and-email step after a verified `order_created` webhook.

## What is intentionally not in-repo

- Live vendor API keys / webhook secrets
- Production signing seed
- Hard-coded buyer PII

Desktop only needs a public checkout URL + offline key verification.
