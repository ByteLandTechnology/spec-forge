#!/usr/bin/env node
// Bumps Cargo.toml [package].version, builds all configured targets, and
// creates dist/ archives with sha256 checksums. Called by @semantic-release/exec
// prepareCmd with the next release version, before sync-platform-packages.mjs.

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { readReleaseConfig } from "./release-config.mjs";
import {
  regenerateLockedCargoGraph,
  stageCargoPackageVersion,
} from "./cargo-version-utils.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const version = process.argv[2];
if (!version) {
  throw new Error("Usage: build-binaries.mjs <version>");
}

const config = readReleaseConfig(rootDir);
const cliName = config.cliName;
const cargoTomlPath = path.join(rootDir, "Cargo.toml");

// --- Bump Cargo.toml [package].version so the binary embeds the correct version ---
const { alreadyCorrect } = stageCargoPackageVersion(cargoTomlPath, version);
if (!alreadyCorrect) {
  console.log(`Bumped Cargo.toml version to ${version}`);
} else {
  console.log(`Cargo.toml already at version ${version}, skipping bump`);
}

// Allow Cargo.lock to reflect the staged root-package version, but reject any
// dependency-graph drift after quality gates have already passed.
regenerateLockedCargoGraph(rootDir, cliName, version);

// --- Build all targets and create archives ---
const distDir = path.join(rootDir, "dist");
const zigGlobalCacheDir = path.join(rootDir, "target", "zig-global-cache");
const zigLocalCacheDir = path.join(rootDir, "target", "zig-local-cache");
mkdirSync(zigGlobalCacheDir, { recursive: true });
mkdirSync(zigLocalCacheDir, { recursive: true });
// Clean dist/ completely to ensure no stale archives from prior runs survive.
if (existsSync(distDir)) rmSync(distDir, { recursive: true, force: true });
mkdirSync(distDir, { recursive: true });

for (const target of config.targets) {
  const rt = target.rustTarget;
  const isWindows = rt.includes("windows");
  const isLinux = rt.includes("linux");
  const binaryName = `${cliName}${isWindows ? ".exe" : ""}`;

  console.log(`Building ${rt}...`);
  const buildArgs = isLinux
    ? ["zigbuild", "--locked", "--release", "--target", rt]
    : ["build", "--locked", "--release", "--target", rt];

  const result = spawnSync("cargo", buildArgs, {
    stdio: "inherit",
    env: {
      ...process.env,
      ZIG_GLOBAL_CACHE_DIR: zigGlobalCacheDir,
      ZIG_LOCAL_CACHE_DIR: zigLocalCacheDir,
    },
  });
  if (result.error) {
    throw new Error(`cargo build failed for ${rt}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`cargo build failed for ${rt} (exit ${result.status}).`);
  }

  const src = path.join(rootDir, "target", rt, "release", binaryName);
  if (!existsSync(src)) {
    throw new Error(`Built binary not found at ${src}.`);
  }

  const outDir = path.join(distDir, rt);
  mkdirSync(outDir, { recursive: true });
  copyFileSync(src, path.join(outDir, binaryName));

  const archiveName = `${cliName}-${rt}.tar.gz`;
  const archivePath = path.join(distDir, archiveName);
  const tar = spawnSync(
    "tar",
    ["-czf", archivePath, "-C", outDir, binaryName],
    { stdio: "pipe" },
  );
  if (tar.error || tar.status !== 0) {
    throw new Error(
      `tar failed for ${archiveName}: ${tar.error?.message ?? `exit ${tar.status}`}`,
    );
  }
  const archiveBuf = readFileSync(archivePath);
  const hash = createHash("sha256").update(archiveBuf).digest("hex");
  writeFileSync(`${archivePath}.sha256`, `${hash}  ${archiveName}\n`, "utf8");

  console.log(`  -> ${archivePath}`);
}

console.log(`Built ${config.targets.length} targets for v${version}.`);

// Write build provenance so recovery can verify artifact origin.
const commitSha =
  spawnSync("git", ["rev-parse", "HEAD"], {
    encoding: "utf8",
  }).stdout?.trim() ?? "unknown";
const gitTag =
  spawnSync("git", ["describe", "--tags", "--exact-match", "HEAD"], {
    encoding: "utf8",
  }).stdout?.trim() ?? null;
const repository = config.sourceRepository;
writeFileSync(
  path.join(distDir, "provenance.json"),
  JSON.stringify(
    {
      version,
      commitSha,
      gitTag,
      repository,
      timestamp: new Date().toISOString(),
    },
    null,
    2,
  ) + "\n",
  "utf8",
);
