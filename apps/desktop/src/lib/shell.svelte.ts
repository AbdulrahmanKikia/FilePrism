/** Shell integration handoff state (Explorer / .desktop → UI). */

export type ShellAction = "open" | "convert" | "compress" | "extract";

export type ShellFilesPayload = {
  v: number;
  action: ShellAction;
  paths: string[];
};

let paths = $state<string[]>([]);
let action = $state<ShellAction>("open");
let revision = $state(0);

export function getShellPaths(): string[] {
  return paths;
}

export function getShellAction(): ShellAction {
  return action;
}

export function getShellRevision(): number {
  return revision;
}

export function applyShellFiles(payload: ShellFilesPayload) {
  paths = [...payload.paths];
  action = payload.action ?? "open";
  revision += 1;
}

export function clearShellFiles() {
  paths = [];
  action = "open";
  revision += 1;
}

export function takePrimaryPath(): string | null {
  return paths[0] ?? null;
}

/** Map shell action → sidebar nav id. */
export function navForAction(a: ShellAction): "home" | "convert" | "compress" | "archives" {
  switch (a) {
    case "convert":
      return "convert";
    case "compress":
      return "compress";
    case "extract":
      return "archives";
    default:
      return "convert";
  }
}
