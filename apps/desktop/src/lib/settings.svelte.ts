/** UI customization state — CSS data attributes + SQLite persistence (M5). */

import { getSetting, isTauri, listSettings, setSetting } from "./api";

export type Theme = "system" | "light" | "dark";
export type Accent = "blue" | "violet" | "teal" | "orange" | "pink" | "green";
export type Glass = "off" | "subtle" | "full";
export type Density = "comfortable" | "compact";
export type Motion = "full" | "reduced";
export type FontSize = "system" | "small" | "large";

export type UiSettings = {
  theme: Theme;
  accent: Accent;
  glass: Glass;
  density: Density;
  motion: Motion;
  font: FontSize;
};

export const UI_SETTING_KEYS = {
  theme: "ui.theme",
  accent: "ui.accent",
  glass: "ui.glass",
  density: "ui.density",
  motion: "ui.motion",
  font: "ui.font",
} as const;

export const DEFAULT_SETTINGS: UiSettings = {
  theme: "system",
  accent: "teal",
  glass: "full",
  density: "comfortable",
  motion: "full",
  font: "system",
};

export const ACCENTS: Accent[] = [
  "blue",
  "violet",
  "teal",
  "orange",
  "pink",
  "green",
];

function resolveTheme(theme: Theme): "light" | "dark" {
  if (theme === "light" || theme === "dark") return theme;
  if (typeof window !== "undefined" && window.matchMedia) {
    return window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  }
  return "dark";
}

export function applySettingsToDom(settings: UiSettings, root: HTMLElement = document.documentElement) {
  const resolved = resolveTheme(settings.theme);
  root.dataset.theme = resolved;
  root.dataset.accent = settings.accent;
  root.dataset.glass = settings.glass;
  root.dataset.density = settings.density;
  root.dataset.motion = settings.motion;
  root.dataset.font = settings.font;

  // Also mirror on body / shell for scoped selectors used in layout.
  document.body?.setAttribute("data-theme", resolved);
  document.body?.setAttribute("data-accent", settings.accent);
  document.body?.setAttribute("data-glass", settings.glass);
  document.body?.setAttribute("data-density", settings.density);
  document.body?.setAttribute("data-motion", settings.motion);
  document.body?.setAttribute("data-font", settings.font);
}

function parseStored(map: Record<string, string>): UiSettings {
  const pick = <T extends string>(key: string, allowed: readonly T[], fallback: T): T => {
    const v = map[key];
    return v && (allowed as readonly string[]).includes(v) ? (v as T) : fallback;
  };

  return {
    theme: pick(UI_SETTING_KEYS.theme, ["system", "light", "dark"] as const, DEFAULT_SETTINGS.theme),
    accent: pick(UI_SETTING_KEYS.accent, ACCENTS, DEFAULT_SETTINGS.accent),
    glass: pick(UI_SETTING_KEYS.glass, ["off", "subtle", "full"] as const, DEFAULT_SETTINGS.glass),
    density: pick(
      UI_SETTING_KEYS.density,
      ["comfortable", "compact"] as const,
      DEFAULT_SETTINGS.density,
    ),
    motion: pick(UI_SETTING_KEYS.motion, ["full", "reduced"] as const, DEFAULT_SETTINGS.motion),
    font: pick(UI_SETTING_KEYS.font, ["system", "small", "large"] as const, DEFAULT_SETTINGS.font),
  };
}

export async function loadUiSettings(): Promise<UiSettings> {
  if (!isTauri()) return { ...DEFAULT_SETTINGS };
  try {
    const rows = await listSettings();
    const map: Record<string, string> = {};
    for (const row of rows) map[row.key] = row.value;
    return parseStored(map);
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export async function persistUiSetting(
  key: keyof typeof UI_SETTING_KEYS,
  value: string,
): Promise<void> {
  if (!isTauri()) return;
  try {
    await setSetting(UI_SETTING_KEYS[key], value);
  } catch {
    // Persistence is best-effort in UI; surface elsewhere if needed.
  }
}

export async function loadHistoryEnabled(): Promise<boolean> {
  if (!isTauri()) return true;
  try {
    const v = await getSetting("history.enabled");
    if (v == null) return true;
    return !(v === "0" || v.toLowerCase() === "false");
  } catch {
    return true;
  }
}

export async function persistHistoryEnabled(enabled: boolean): Promise<void> {
  if (!isTauri()) return;
  await setSetting("history.enabled", enabled ? "true" : "false");
}
