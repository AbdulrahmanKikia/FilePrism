import type { ImageFormat } from "./wasm/types";

const STORAGE_KEY = "fileprism.web.history";
const MAX_ENTRIES = 100;

export type WebJobRecord = {
  id: string;
  createdAt: string;
  inputName: string;
  inputFormat: ImageFormat;
  outputFormat: ImageFormat;
  status: "completed" | "failed";
  bytesWritten?: number;
  engine?: "wasm" | "canvas-stub";
  errorMessage?: string;
};

function readAll(): WebJobRecord[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as WebJobRecord[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function writeAll(jobs: WebJobRecord[]): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(jobs.slice(0, MAX_ENTRIES)));
}

export function listHistory(query = ""): WebJobRecord[] {
  const q = query.trim().toLowerCase();
  const jobs = readAll();
  if (!q) return jobs;
  return jobs.filter((job) => {
    const hay = [
      job.inputName,
      job.inputFormat,
      job.outputFormat,
      job.status,
      job.id,
      job.errorMessage ?? "",
    ]
      .join(" ")
      .toLowerCase();
    return hay.includes(q);
  });
}

export function addHistory(entry: Omit<WebJobRecord, "id" | "createdAt"> & { id?: string }): WebJobRecord {
  const record: WebJobRecord = {
    id: entry.id ?? crypto.randomUUID(),
    createdAt: new Date().toISOString(),
    inputName: entry.inputName,
    inputFormat: entry.inputFormat,
    outputFormat: entry.outputFormat,
    status: entry.status,
    bytesWritten: entry.bytesWritten,
    engine: entry.engine,
    errorMessage: entry.errorMessage,
  };
  const next = [record, ...readAll()].slice(0, MAX_ENTRIES);
  writeAll(next);
  return record;
}

export function clearHistory(): void {
  writeAll([]);
}
