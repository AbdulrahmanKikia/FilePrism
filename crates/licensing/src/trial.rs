//! Trial clock and entitlement resolution.

use serde::{Deserialize, Serialize};

use crate::LicenseClaims;

/// Full-feature trial length in days (spec §7: 7–14). FilePrism ships **14**.
pub const TRIAL_DAYS: u32 = 14;

const SECONDS_PER_DAY: i64 = 86_400;

/// Trial configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialConfig {
    pub days: u32,
}

impl Default for TrialConfig {
    fn default() -> Self {
        Self { days: TRIAL_DAYS }
    }
}

/// High-level entitlement kind for UI / gating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EntitlementKind {
    /// Valid offline license (app keeps working after update window; updates may be gated later).
    Licensed {
        lid: String,
        email: Option<String>,
        #[serde(rename = "updatesActive")]
        updates_active: bool,
        #[serde(rename = "updatesUntil")]
        updates_until: i64,
    },
    /// Full-feature trial in progress.
    Trial {
        #[serde(rename = "daysRemaining")]
        days_remaining: u32,
        #[serde(rename = "endsAt")]
        ends_at: i64,
    },
    /// Trial finished and no valid license stored.
    TrialExpired,
    /// No trial start recorded yet (host should stamp first launch).
    NotStarted,
}

/// Resolved entitlement snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entitlement {
    pub kind: EntitlementKind,
    /// Spec: trial and paid license are full-feature for v1 conversion features.
    pub full_feature: bool,
    pub trial_days: u32,
}

/// Resolve entitlement from an optional verified license and trial start timestamp.
///
/// `trial_started_at` / `now` are Unix seconds (UTC).
/// `purchase_url` is informational only (passed through by the host if desired).
pub fn resolve_entitlement(
    license: Option<&LicenseClaims>,
    trial_started_at: Option<i64>,
    now: i64,
    config: &TrialConfig,
    _purchase_url: Option<&str>,
) -> Entitlement {
    if let Some(claims) = license {
        return Entitlement {
            kind: EntitlementKind::Licensed {
                lid: claims.lid.clone(),
                email: claims.email.clone(),
                updates_active: claims.updates_active_at(now),
                updates_until: claims.updates_until,
            },
            full_feature: true,
            trial_days: config.days,
        };
    }

    let Some(started) = trial_started_at else {
        return Entitlement {
            kind: EntitlementKind::NotStarted,
            full_feature: false,
            trial_days: config.days,
        };
    };

    let ends_at = started.saturating_add(i64::from(config.days) * SECONDS_PER_DAY);
    if now <= ends_at {
        let remaining_secs = (ends_at - now).max(0);
        let days_remaining = ((remaining_secs + SECONDS_PER_DAY - 1) / SECONDS_PER_DAY) as u32;
        Entitlement {
            kind: EntitlementKind::Trial {
                days_remaining: days_remaining.min(config.days),
                ends_at,
            },
            full_feature: true,
            trial_days: config.days,
        }
    } else {
        Entitlement {
            kind: EntitlementKind::TrialExpired,
            full_feature: false,
            trial_days: config.days,
        }
    }
}
