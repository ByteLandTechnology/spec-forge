#!/usr/bin/env node
// Check whether a release version is already complete across GitHub assets
// (when RELEASE_ASSETS_JSON is provided) and npm packages. Prints a
// semicolon-delimited reason string when anything is missing.

import { spawnSync } from "node:child_process";

import { buildAllPackageNames, readReleaseConfig } from "./release-config.mjs";

const version = process.argv[2];
if (!version) {
  console.error(
    "Usage: node scripts/release/check-release-complete.mjs <version>",
  );
  process.exit(1);
}

const config = readReleaseConfig(process.cwd());
const missingReasons = [];

if (process.env.RELEASE_ASSETS_JSON) {
  const actual = new Set(
    (JSON.parse(process.env.RELEASE_ASSETS_JSON).assets ?? []).map(
      (asset) => asset.name,
    ),
  );
  const expected = [
    ...config.targets.flatMap((target) => {
      const archive = `${config.cliName}-${target.rustTarget}.tar.gz`;
      return [archive, `${archive}.sha256`];
    }),
    "provenance.json",
  ];
  const missingAssets = expected.filter((name) => !actual.has(name));
  if (missingAssets.length > 0) {
    missingReasons.push(
      `github_release_missing_assets:${missingAssets.join(",")}`,
    );
  }
}

const packageNames = buildAllPackageNames(config);
const missingPackages = [];
for (const name of packageNames) {
  const result = spawnSync("npm", ["view", `${name}@${version}`, "version"], {
    encoding: "utf8",
    stdio: "pipe",
  });
  if (result.status !== 0) {
    const stderr = (result.stderr ?? "").toLowerCase();
    const notFound =
      stderr.includes("e404") ||
      stderr.includes("etarget") ||
      stderr.includes("not found");
    if (notFound) {
      missingPackages.push(`${name}@${version}`);
      continue;
    }
    throw new Error(
      `npm view ${name}@${version} failed: ${(result.stderr ?? "").trim()}`,
    );
  }
  if (result.stdout.trim() !== version) {
    missingPackages.push(`${name}@${version}`);
  }
}

if (missingPackages.length > 0) {
  missingReasons.push(`npm_packages_missing:${missingPackages.join(",")}`);
}

if (missingReasons.length > 0) {
  process.stdout.write(missingReasons.join(";"));
}
