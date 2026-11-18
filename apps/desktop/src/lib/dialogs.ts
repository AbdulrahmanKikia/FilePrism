/** Native file / folder dialogs via Tauri plugin-dialog. */

import { open, save, type DialogFilter } from "@tauri-apps/plugin-dialog";
import { isTauri } from "$lib/api";

export type PathKind = "file" | "directory" | "save";

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
