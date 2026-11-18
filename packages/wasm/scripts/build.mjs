#!/usr/bin/env node
/**
 * Build crates/wasm → packages/wasm/pkg for @fileprism/wasm.
 * Cooler default: CARGO_BUILD_JOBS=1 (override with --jobs N).
 */
import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(__dirname, "..");
const repoRoot = resolve(pkgRoot, "../..");
const outDir = join(pkgRoot, "pkg");
const targetDir = join(repoRoot, "target/wasm32-unknown-unknown");

const release = process.argv.includes("--release");
const jobsIdx = process.argv.indexOf("--jobs");
const jobs =
  jobsIdx >= 0 && process.argv[jobsIdx + 1]
    ? process.argv[jobsIdx + 1]
    : process.env.CARGO_BUILD_JOBS || "1";

function run(cmd, args, opts = {}) {
  console.log(`> ${cmd} ${args.join(" ")}`);
  const r = spawnSync(cmd, args, {
    stdio: "inherit",
    cwd: repoRoot,
    env: { ...process.env, CARGO_BUILD_JOBS: String(jobs) },
    shell: process.platform === "win32",
    ...opts,
  });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

// Ensure wasm32 target
run("rustup", ["target", "add", "wasm32-unknown-unknown"]);

const profile = release ? "release" : "dev";
const cargoArgs = [
  "build",
  "-j",
  String(jobs),
  "-p",
  "fileprism-wasm",
  "--target",
  "wasm32-unknown-unknown",
];
if (release) cargoArgs.push("--release");
run("cargo", cargoArgs);

const wasmArtifact = join(
  targetDir,
  release ? "release" : "debug",
  "fileprism_wasm.wasm",
);
if (!existsSync(wasmArtifact)) {
  console.error(`missing artifact: ${wasmArtifact}`);
  process.exit(1);
}

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

// Prefer wasm-bindgen CLI; fall back to wasm-pack if present.
const bindgen = spawnSync("wasm-bindgen", ["--version"], {
  encoding: "utf8",
  shell: process.platform === "win32",
});
if (bindgen.status === 0) {
  run("wasm-bindgen", [
    wasmArtifact,
    "--out-dir",
    outDir,
    "--target",
    "web",
    "--typescript",
  ]);
} else {
  console.log("wasm-bindgen CLI missing; trying wasm-pack…");
  run("wasm-pack", [
    "build",
    "crates/wasm",
    "--target",
    "web",
    "--out-dir",
    outDir,
    release ? "--release" : "--dev",
  ]);
}

// Soften wasm-bindgen package.json if it overwrote ours — we keep parent package.json.
const generatedPkg = join(outDir, "package.json");
if (existsSync(generatedPkg)) {
  // Keep generated for tooling but tag it private helper.
  const raw = JSON.parse(readFileSync(generatedPkg, "utf8"));
  raw.name = "@fileprism/wasm-pkg";
  raw.private = true;
  writeFileSync(generatedPkg, JSON.stringify(raw, null, 2) + "\n");
}

console.log(`Built @fileprism/wasm (${profile}, -j ${jobs}) → ${outDir}`);
