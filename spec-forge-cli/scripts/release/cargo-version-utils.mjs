#!/usr/bin/env node
// Shared helpers for release-time Cargo version staging. They allow the root
// package version to move while rejecting any other Cargo.lock graph changes.

import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

function replacePackageVersionLine(tomlText, packageName, nextVersion) {
  const escapedName = packageName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const blockPattern = new RegExp(
    String.raw`(\[\[package\]\]\nname = "${escapedName}"\nversion = ")([^"]+)(")`,
  );
  return tomlText.replace(blockPattern, `$1${nextVersion}$3`);
}

function readLockedPackageVersion(lockText, packageName) {
  const escapedName = packageName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const blockPattern = new RegExp(
    String.raw`\[\[package\]\]\nname = "${escapedName}"\nversion = "([^"]+)"`,
  );
  return lockText.match(blockPattern)?.[1] ?? null;
}

export function stageCargoPackageVersion(cargoTomlPath, version) {
  const cargoLines = readFileSync(cargoTomlPath, "utf8").split("\n");
  let inPackage = false;
  let bumped = false;
  let alreadyCorrect = false;
  for (let i = 0; i < cargoLines.length; i += 1) {
    const trimmed = cargoLines[i].trim();
    if (trimmed === "[package]") {
      inPackage = true;
      continue;
    }
    if (inPackage && trimmed.startsWith("[")) {
      break;
    }
    if (inPackage && /^version\s*=/.test(trimmed)) {
      const currentVersion = trimmed.match(/version\s*=\s*"([^"]+)"/)?.[1];
      if (currentVersion === version) {
        alreadyCorrect = true;
      } else {
        cargoLines[i] = `version = "${version}"`;
      }
      bumped = true;
      break;
    }
  }

  if (!bumped) {
    throw new Error("Could not find [package].version in Cargo.toml.");
  }

  if (!alreadyCorrect) {
    writeFileSync(cargoTomlPath, cargoLines.join("\n"), "utf8");
  }

  return { alreadyCorrect };
}

export function regenerateLockedCargoGraph(rootDir, packageName, version) {
  const cargoLockPath = path.join(rootDir, "Cargo.lock");
  const beforeLock = readFileSync(cargoLockPath, "utf8");
  const beforeVersion = readLockedPackageVersion(beforeLock, packageName);
  const expectedLock = replacePackageVersionLine(
    beforeLock,
    packageName,
    version,
  );
  if (expectedLock === beforeLock && beforeVersion !== version) {
    throw new Error(
      `Could not locate ${packageName} in Cargo.lock to stage version ${version}.`,
    );
  }

  writeFileSync(cargoLockPath, expectedLock, "utf8");
}
