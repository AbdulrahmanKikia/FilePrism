//! License / trial Tauri commands (M7).

use std::time::{SystemTime, UNIX_EPOCH};

use fileprism_licensing::{
    embedded_verifying_key, parse_license_key, resolve_entitlement, Entitlement, TrialConfig,
    TRIAL_DAYS,
};
use serde::Serialize;

use crate::AppState;

pub const SETTING_LICENSE_KEY: &str = "license.key";
pub const SETTING_TRIAL_STARTED: &str = "license.trial_started_at";

/// Placeholder checkout URL — override with `FILEPRISM_CHECKOUT_URL` (Paddle overlay / LS).
pub const DEFAULT_CHECKOUT_URL: &str =
    "https://checkout.paddle.com/checkout/custom/fileprism-placeholder";

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn checkout_url() -> String {
    std::env::var("FILEPRISM_CHECKOUT_URL").unwrap_or_else(|_| DEFAULT_CHECKOUT_URL.to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatusDto {
    pub entitlement: Entitlement,
    pub has_stored_key: bool,
    pub checkout_url: String,
    pub trial_days: u32,
    pub masked_key: Option<String>,
}

fn mask_key(key: &str) -> String {
    let key = key.trim();
    if key.len() <= 12 {
        return "***".into();
    }
    format!("{}…{}", &key[..8], &key[key.len().saturating_sub(4)..])
}

pub(crate) fn ensure_trial_started(state: &AppState) -> Result<i64, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    if let Some(raw) = history
        .get_setting(SETTING_TRIAL_STARTED)
        .map_err(|e| e.to_string())?
    {
        if let Ok(ts) = raw.parse::<i64>() {
            return Ok(ts);
        }
    }
    let started = now_unix();
    history
        .set_setting(SETTING_TRIAL_STARTED, &started.to_string())
        .map_err(|e| e.to_string())?;
    Ok(started)
}

fn build_status(state: &AppState) -> Result<LicenseStatusDto, String> {
    let trial_started = ensure_trial_started(state)?;
    let history = state.history.lock().map_err(|e| e.to_string())?;
    let stored = history
        .get_setting(SETTING_LICENSE_KEY)
        .map_err(|e| e.to_string())?;

    let vk = embedded_verifying_key();
    let claims = match stored.as_deref() {
        Some(key) if !key.trim().is_empty() => match parse_license_key(key, &vk) {
            Ok(c) => Some(c),
            Err(_) => None,
        },
        _ => None,
    };

    let entitlement = resolve_entitlement(
        claims.as_ref(),
        Some(trial_started),
        now_unix(),
        &TrialConfig::default(),
        Some(&checkout_url()),
    );

    let has_stored_key = stored
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    Ok(LicenseStatusDto {
        entitlement,
        has_stored_key,
        checkout_url: checkout_url(),
        trial_days: TRIAL_DAYS,
        masked_key: stored.as_deref().filter(|s| !s.trim().is_empty()).map(mask_key),
    })
}

#[tauri::command]
pub fn license_status(state: tauri::State<'_, AppState>) -> Result<LicenseStatusDto, String> {
    build_status(&state)
}

#[tauri::command]
pub fn activate_license(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<LicenseStatusDto, String> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("license key is empty".into());
    }
    let vk = embedded_verifying_key();
    parse_license_key(&key, &vk).map_err(|e| e.to_string())?;

    {
        let history = state.history.lock().map_err(|e| e.to_string())?;
        history
            .set_setting(SETTING_LICENSE_KEY, &key)
            .map_err(|e| e.to_string())?;
    }
    let _ = ensure_trial_started(&state)?;
    build_status(&state)
}

#[tauri::command]
pub fn deactivate_license(
    state: tauri::State<'_, AppState>,
) -> Result<LicenseStatusDto, String> {
    {
        let history = state.history.lock().map_err(|e| e.to_string())?;
        history
            .set_setting(SETTING_LICENSE_KEY, "")
            .map_err(|e| e.to_string())?;
    }
    build_status(&state)
}

#[tauri::command]
pub fn purchase_url() -> String {
    checkout_url()
}
