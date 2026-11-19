/** Native file / folder dialogs via Tauri plugin-dialog. */

import { open, save, type DialogFilter } from "@tauri-apps/plugin-dialog";
import { isTauri } from "$lib/api";

export type PathKind = "file" | "directory" | "save";

/** Lowercase extension without leading dot; `jpeg` → `jpg`. */
export function normalizeFormatExt(ext: string): string {
  const e = ext.replace(/^\./, "").toLowerCase();
  if (e === "jpeg") return "jpg";
  return e;
}

/** File extension of a path (lowercase, no leading dot), or empty. */
export function extensionFromPath(path: string): string {
  const trimmed = path.trim();
  if (!trimmed) return "";
  const normalized = trimmed.replace(/\\/g, "/");
  const slash = normalized.lastIndexOf("/");
  const base = slash >= 0 ? normalized.slice(slash + 1) : normalized;
  const dot = base.lastIndexOf(".");
  if (dot <= 0 || dot === base.length - 1) return "";
  return base.slice(dot + 1).toLowerCase();
}

/** Replace the final extension of `path` (keeps stem and directory). */
export function replaceExtension(path: string, newExt: string): string {
  return suggestSiblingPath(path, newExt);
}

export function suggestSiblingPath(inputPath: string, newExt: string): string {
  const trimmed = inputPath.trim();
  if (!trimmed) return "";
  const normalized = trimmed.replace(/\\/g, "/");
  const slash = Math.max(normalized.lastIndexOf("/"), 0);
  const base = normalized.slice(slash === 0 && !normalized.includes("/") ? 0 : slash + 1);
  const dir = trimmed.slice(0, trimmed.length - base.length);
  const stem = base.includes(".") ? base.slice(0, base.lastIndexOf(".")) : base;
  const ext = newExt.replace(/^\./, "");
  return `${dir}${stem}.${ext}`;
}

export function suggestDirBeside(inputPath: string): string {
  const trimmed = inputPath.trim();
  if (!trimmed) return "";
  const normalized = trimmed.replace(/\\/g, "/");
  const slash = Math.max(normalized.lastIndexOf("/"), 0);
  const base = normalized.slice(slash === 0 && !normalized.includes("/") ? 0 : slash + 1);
  const dir = trimmed.slice(0, trimmed.length - base.length);
  const stem = base.includes(".") ? base.slice(0, base.lastIndexOf(".")) : base;
  return `${dir}${stem}`;
}

export async function pickPath(opts: {
  kind: PathKind;
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
}): Promise<string | null> {
  if (!isTauri()) return null;

  if (opts.kind === "save") {
    return save({
      title: opts.title,
      defaultPath: opts.defaultPath || undefined,
      filters: opts.filters,
    });
  }

  const selected = await open({
    title: opts.title,
    defaultPath: opts.defaultPath || undefined,
    directory: opts.kind === "directory",
    multiple: false,
    filters: opts.kind === "file" ? opts.filters : undefined,
  });

  if (selected === null) return null;
  return Array.isArray(selected) ? (selected[0] ?? null) : selected;
}
