/** UI customization — same Glass Pro tokens as desktop; persisted in localStorage. */

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

const STORAGE_KEY = "fileprism.web.ui";
const HISTORY_KEY = "fileprism.web.historyEnabled";

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

export function applySettingsToDom(
  settings: UiSettings,
  root: HTMLElement = document.documentElement,
) {
  const resolved = resolveTheme(settings.theme);
  root.dataset.theme = resolved;
  root.dataset.accent = settings.accent;
  root.dataset.glass = settings.glass;
  root.dataset.density = settings.density;
  root.dataset.motion = settings.motion;
  root.dataset.font = settings.font;

  document.body?.setAttribute("data-theme", resolved);
  document.body?.setAttribute("data-accent", settings.accent);
  document.body?.setAttribute("data-glass", settings.glass);
  document.body?.setAttribute("data-density", settings.density);
  document.body?.setAttribute("data-motion", settings.motion);
  document.body?.setAttribute("data-font", settings.font);
}

function isAccent(v: string): v is Accent {
  return (ACCENTS as string[]).includes(v);
}

export function loadUiSettings(): UiSettings {
  if (typeof localStorage === "undefined") return { ...DEFAULT_SETTINGS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_SETTINGS };
    const parsed = JSON.parse(raw) as Partial<UiSettings>;
    return {
      theme:
        parsed.theme === "light" || parsed.theme === "dark" || parsed.theme === "system"
          ? parsed.theme
          : DEFAULT_SETTINGS.theme,
      accent: parsed.accent && isAccent(parsed.accent) ? parsed.accent : DEFAULT_SETTINGS.accent,
      glass:
        parsed.glass === "off" || parsed.glass === "subtle" || parsed.glass === "full"
          ? parsed.glass
          : DEFAULT_SETTINGS.glass,
      density:
        parsed.density === "comfortable" || parsed.density === "compact"
          ? parsed.density
          : DEFAULT_SETTINGS.density,
      motion:
        parsed.motion === "full" || parsed.motion === "reduced"
          ? parsed.motion
          : DEFAULT_SETTINGS.motion,
      font:
        parsed.font === "system" || parsed.font === "small" || parsed.font === "large"
          ? parsed.font
          : DEFAULT_SETTINGS.font,
    };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export function persistUiSettings(settings: UiSettings): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export function loadHistoryEnabled(): boolean {
  if (typeof localStorage === "undefined") return true;
  const v = localStorage.getItem(HISTORY_KEY);
  if (v == null) return true;
  return !(v === "0" || v.toLowerCase() === "false");
}

export function persistHistoryEnabled(enabled: boolean): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(HISTORY_KEY, enabled ? "true" : "false");
}
