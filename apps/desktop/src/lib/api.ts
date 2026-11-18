/** Tauri IPC wrappers for FilePrism desktop commands (M5). */

import { invoke } from "@tauri-apps/api/core";

export type JobOptions = {
  quality?: number | null;
  width?: number | null;
  height?: number | null;
  strip_metadata?: boolean | null;
  password?: string | null;
};

export type JobOutput = {
  job_id: string;
  output_path: string;
  bytes_written: number;
};

export type JobRecord = {
  id: string;
  input_path: string;
  output_path: string;
  input_format: string;
  output_format: string;
  status: string;
  bytes_written: number | null;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
};

export type Setting = {
  key: string;
  value: string;
  updated_at: string;
};

export type RunJobArgs = {
  inputPath: string;
  outputPath: string;
  inputFormat: string;
  outputFormat: string;
  options?: JobOptions;
  id?: string;
};

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function runJob(args: RunJobArgs): Promise<JobOutput> {
  return invoke<JobOutput>("run_job", { args });
}

export async function searchHistory(
  query = "",
  limit = 100,
): Promise<JobRecord[]> {
  return invoke<JobRecord[]>("search_history", { query, limit });
}

export async function getHistoryJob(id: string): Promise<JobRecord | null> {
  return invoke<JobRecord | null>("get_history_job", { id });
}

export async function rerunJob(id: string): Promise<JobOutput> {
  return invoke<JobOutput>("rerun_job", { id });
}

export async function listSettings(): Promise<Setting[]> {
  return invoke<Setting[]>("list_settings");
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

/** M7 license / trial status from `fileprism-licensing`. */
export type EntitlementKind =
  | {
      type: "licensed";
      lid: string;
      email?: string | null;
      updatesActive: boolean;
      updatesUntil: number;
    }
  | { type: "trial"; daysRemaining: number; endsAt: number }
  | { type: "trialExpired" }
  | { type: "notStarted" };

export type Entitlement = {
  kind: EntitlementKind;
  fullFeature: boolean;
  trialDays: number;
};

export type LicenseStatus = {
  entitlement: Entitlement;
  hasStoredKey: boolean;
  checkoutUrl: string;
  trialDays: number;
  maskedKey: string | null;
};

export async function getLicenseStatus(): Promise<LicenseStatus> {
  return invoke<LicenseStatus>("license_status");
}

export async function activateLicense(key: string): Promise<LicenseStatus> {
  return invoke<LicenseStatus>("activate_license", { key });
}

export async function deactivateLicense(): Promise<LicenseStatus> {
  return invoke<LicenseStatus>("deactivate_license");
}

export async function purchaseUrl(): Promise<string> {
  return invoke<string>("purchase_url");
}

export function formatError(err: unknown): string {
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}
