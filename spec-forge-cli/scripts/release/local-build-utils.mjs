#!/usr/bin/env node
// Shared local-only build helpers for rehearsal and prepublish publication.

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { tmpdir } from "node:os";

import { readReleaseConfig } from "./release-config.mjs";
import {
  regenerateLockedCargoGraph,
  stageCargoPackageVersion,
} from "./cargo-version-utils.mjs";

function snapshotDir(dir) {
  const existed = existsSync(dir);
  const entries = new Map();
  if (!existed) return { existed, entries };

  const queue = [""];
  while (queue.length) {
    const rel = queue.shift();
    const full = rel ? path.join(dir, rel) : dir;
    const st = statSync(full);
    if (st.isDirectory()) {
      for (const child of readdirSync(full)) {
        queue.push(rel ? `${rel}/${child}` : child);
      }
    } else {
      entries.set(rel, readFileSync(full));
    }
  }

  return { existed, entries };
}

function restoreDir(dir, snapshot) {
  rmSync(dir, { recursive: true, force: true });
  if (!snapshot.existed) return;

  mkdirSync(dir, { recursive: true });
  for (const [rel, content] of snapshot.entries) {
    const full = path.join(dir, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
}

export function createLocalReleaseWorkspace(rootDir) {
  const config = readReleaseConfig(rootDir);
  const distDir = path.join(rootDir, "dist");
  const platformsDir = path.join(rootDir, "npm/platforms");
  const cargoTomlPath = path.join(rootDir, "Cargo.toml");
  const cargoLockPath = path.join(rootDir, "Cargo.lock");
  const mainPkgPath = path.join(rootDir, "npm/main/package.json");
  const mainReadmePath = path.join(rootDir, "npm/main/README.md");
  const isolatedTargetDir = mkdtempSync(
    path.join(tmpdir(), "cli-forge-local-release-"),
  );
  const isolatedZigGlobalCacheDir = path.join(
    isolatedTargetDir,
    "zig-global-cache",
  );
  const isolatedZigLocalCacheDir = path.join(
    isolatedTargetDir,
    "zig-local-cache",
  );
  const isolatedNpmCacheDir = path.join(isolatedTargetDir, "npm-cache");
  mkdirSync(isolatedZigGlobalCacheDir, { recursive: true });
  mkdirSync(isolatedZigLocalCacheDir, { recursive: true });
  mkdirSync(isolatedNpmCacheDir, { recursive: true });

  const fileSnapshots = new Map();
  const trackedFilePaths = [
    cargoTomlPath,
    cargoLockPath,
    mainPkgPath,
    mainReadmePath,
  ];
  for (const filePath of trackedFilePaths) {
    if (existsSync(filePath)) {
      fileSnapshots.set(filePath, readFileSync(filePath));
    }
  }

  const distSnapshot = snapshotDir(distDir);
  const platformsSnapshot = snapshotDir(platformsDir);

  let restored = false;
  const sigintHandler = () => {
    restore();
    process.exit(130);
  };

  function restore() {
    if (restored) return;
    restored = true;

    for (const filePath of trackedFilePaths) {
      const content = fileSnapshots.get(filePath);
      if (content) {
        writeFileSync(filePath, content);
      } else {
        rmSync(filePath, { force: true });
      }
    }
    restoreDir(distDir, distSnapshot);
    restoreDir(platformsDir, platformsSnapshot);
    rmSync(isolatedTargetDir, { recursive: true, force: true });
  }

  function cleanup() {
    process.removeListener("SIGINT", sigintHandler);
    restore();
  }

  function validateConfig() {
    const validateResult = spawnSync(
      process.execPath,
      [path.join(rootDir, "scripts/release/validate-config.mjs")],
      { cwd: rootDir, stdio: "inherit" },
    );
    if (validateResult.status !== 0) {
      throw new Error("Config validation failed.");
    }
  }

  function runBuildPreflight() {
    console.log("\n=== Preflight ===\n");

    const hasLinux = config.targets.some((target) =>
      target.rustTarget.includes("linux"),
    );
    const hasWindows = config.targets.some((target) =>
      target.rustTarget.includes("windows"),
    );

    if (spawnSync("cargo", ["--version"], { encoding: "utf8" }).status !== 0) {
      throw new Error("cargo not found. Install Rust: https://rustup.rs");
    }
    console.log("  cargo: OK");

    if (hasLinux) {
      const zigcheck = spawnSync("cargo", ["zigbuild", "--help"], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      });
      if (zigcheck.status !== 0) {
        throw new Error(
          "cargo-zigbuild not found. Install: cargo install cargo-zigbuild && brew install zig (macOS)",
        );
      }
      console.log("  cargo-zigbuild: OK");
    }

    const installedTargets =
      spawnSync("rustup", ["target", "list", "--installed"], {
        encoding: "utf8",
      }).stdout ?? "";

    if (hasWindows) {
      const winTargets = config.targets
        .filter((target) => target.rustTarget.includes("windows"))
        .map((target) => target.rustTarget);
      for (const rustTarget of winTargets) {
        if (!installedTargets.includes(rustTarget)) {
          throw new Error(
            `Rust target ${rustTarget} not installed. Run: rustup target add ${rustTarget}`,
          );
        }
      }
      const requiredLinkers = new Set(
        winTargets.map((rustTarget) => {
          if (rustTarget.startsWith("aarch64-")) {
            return "aarch64-w64-mingw32-clang";
          }
          if (rustTarget.startsWith("x86_64-")) {
            return "x86_64-w64-mingw32-clang";
          }
          return null;
        }),
      );
      for (const linker of requiredLinkers) {
        if (!linker) continue;
        const linkerCheck = spawnSync(linker, ["--version"], {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"],
        });
        if (linkerCheck.status !== 0) {
          throw new Error(
            `${linker} not found. Install llvm-mingw or run the GitHub release workflow, whose setup-build-env action installs it automatically.`,
          );
        }
      }
      console.log("  windows targets/linkers: OK");
    }

    for (const target of config.targets) {
      if (!installedTargets.includes(target.rustTarget)) {
        throw new Error(
          `Rust target ${target.rustTarget} not installed. Run: rustup target add ${target.rustTarget}`,
        );
      }
    }
    console.log("  all rust targets: OK\n");
  }

  function buildDist() {
    console.log("=== Step 1: Build ===\n");
    mkdirSync(distDir, { recursive: true });

    for (const target of config.targets) {
      const rustTarget = target.rustTarget;
      console.log(`Building ${rustTarget}...`);

      const isWindows = rustTarget.includes("windows");
      const isLinux = rustTarget.includes("linux");
      const binaryName = `${config.cliName}${isWindows ? ".exe" : ""}`;
      const outDir = path.join(distDir, rustTarget);
      mkdirSync(outDir, { recursive: true });

      const buildArgs = [
        "build",
        "--locked",
        "--release",
        "--target",
        rustTarget,
      ];
      if (isLinux) {
        buildArgs[0] = "zigbuild";
      }

      const buildResult = spawnSync("cargo", buildArgs, {
        stdio: "inherit",
        env: {
          ...process.env,
          CARGO_TARGET_DIR: isolatedTargetDir,
          ZIG_GLOBAL_CACHE_DIR: isolatedZigGlobalCacheDir,
          ZIG_LOCAL_CACHE_DIR: isolatedZigLocalCacheDir,
        },
      });
      if (buildResult.status !== 0) {
        throw new Error(
          `cargo build failed for ${rustTarget} (exit ${buildResult.status}).`,
        );
      }

      const src = path.join(
        isolatedTargetDir,
        rustTarget,
        "release",
        binaryName,
      );
      if (!existsSync(src)) {
        throw new Error(`Built binary not found at ${src}.`);
      }
      const dst = path.join(outDir, binaryName);
      copyFileSync(src, dst);
      console.log(`  -> ${dst}`);
    }
  }

  function stageCargoVersion(version) {
    if (!existsSync(cargoTomlPath)) {
      throw new Error(`Missing ${cargoTomlPath}.`);
    }

    stageCargoPackageVersion(cargoTomlPath, version);
    console.log(
      `Staged Cargo.toml version ${version} for local release build.`,
    );
    regenerateLockedCargoGraph(rootDir, config.cliName, version);
  }

  process.on("SIGINT", sigintHandler);

  return {
    config,
    distDir,
    npmCacheDir: isolatedNpmCacheDir,
    platformsDir,
    rootDir,
    validateConfig,
    runBuildPreflight,
    stageCargoVersion,
    buildDist,
    cleanup,
  };
}
