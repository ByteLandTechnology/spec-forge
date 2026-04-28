#!/usr/bin/env node
// Runs during semantic-release `publish`. Publishes every per-platform
// package, then the main wrapper package. Both are idempotent on re-run for
// the same version: each package is probed via `npm view <pkg>@<version>`
// before publishing so a partial failure can be retried without re-failing on
// packages that already made it to the registry.
//
// `.releaserc.json` sets @semantic-release/npm `npmPublish: false`; that
// plugin only handles version bumping during `prepare`, and this script owns
// every `npm publish` invocation for the release.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildMainPackageName, readReleaseConfig } from "./release-config.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const version = process.argv[2];
if (!version) {
  throw new Error("Usage: publish-npm-packages.mjs <version>");
}

const config = readReleaseConfig(rootDir);
const platformsDir = path.join(rootDir, "npm/platforms");
const mainPkgDir = path.join(rootDir, "npm/main");
const RECOVERY_DIST_TAG = "recovery";

// Track skipped steps for receipt generation.
const skippedSteps = [];

function alreadyPublished(pkgName, pkgVersion) {
  const r = spawnSync("npm", ["view", `${pkgName}@${pkgVersion}`, "version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (r.status === 0) return r.stdout.trim() === pkgVersion;

  const stderr = (r.stderr ?? "").toLowerCase();
  // Only treat explicit "not found" indicators as "not published".
  const notFound =
    stderr.includes("e404") ||
    stderr.includes("etarget") ||
    stderr.includes("not found");
  if (notFound) return false;

  // Anything else (network, auth, rate-limit) is ambiguous — fail fast.
  throw new Error(
    `npm view ${pkgName}@${pkgVersion} failed with unexpected error:\n${r.stderr?.trim() ?? "(no stderr)"}`,
  );
}

function parseSemver(value) {
  const match = /^(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)(?:-(?<prerelease>[0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(
    value,
  );
  if (!match?.groups) {
    throw new Error(`Unsupported semver version: ${JSON.stringify(value)}`);
  }
  return {
    major: Number(match.groups.major),
    minor: Number(match.groups.minor),
    patch: Number(match.groups.patch),
    prerelease: match.groups.prerelease
      ? match.groups.prerelease.split(".")
      : [],
  };
}

function compareSemver(a, b) {
  const left = parseSemver(a);
  const right = parseSemver(b);
  for (const key of ["major", "minor", "patch"]) {
    if (left[key] !== right[key]) {
      return left[key] < right[key] ? -1 : 1;
    }
  }

  if (left.prerelease.length === 0 && right.prerelease.length === 0) return 0;
  if (left.prerelease.length === 0) return 1;
  if (right.prerelease.length === 0) return -1;

  const count = Math.max(left.prerelease.length, right.prerelease.length);
  for (let index = 0; index < count; index += 1) {
    const l = left.prerelease[index];
    const r = right.prerelease[index];
    if (l === undefined) return -1;
    if (r === undefined) return 1;
    if (l === r) continue;

    const lNumeric = /^\d+$/.test(l);
    const rNumeric = /^\d+$/.test(r);
    if (lNumeric && rNumeric) return Number(l) < Number(r) ? -1 : 1;
    if (lNumeric) return -1;
    if (rNumeric) return 1;
    return l < r ? -1 : 1;
  }
  return 0;
}

function readLatestDistTag(pkgName) {
  const r = spawnSync("npm", ["view", pkgName, "dist-tags", "--json"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (r.status === 0) {
    const distTags = JSON.parse(r.stdout || "{}");
    return typeof distTags.latest === "string" ? distTags.latest.trim() : null;
  }

  const stderr = (r.stderr ?? "").toLowerCase();
  const notFound =
    stderr.includes("e404") ||
    stderr.includes("etarget") ||
    stderr.includes("not found");
  if (notFound) return null;

  throw new Error(
    `npm view ${pkgName} dist-tags failed with unexpected error:\n${r.stderr?.trim() ?? "(no stderr)"}`,
  );
}

function resolvePublishTag(pkgName, pkgVersion) {
  const latestVersion = readLatestDistTag(pkgName);
  if (!latestVersion) return null;
  return compareSemver(pkgVersion, latestVersion) < 0
    ? RECOVERY_DIST_TAG
    : null;
}

function publishPackage(pkgDir, label) {
  const pkgName = JSON.parse(
    readFileSync(path.join(pkgDir, "package.json"), "utf8"),
  ).name;

  if (alreadyPublished(pkgName, version)) {
    console.log(`skip ${label} ${pkgName}@${version} (already on registry)`);
    skippedSteps.push({
      step: `publish-npm:${label}`,
      reason: "already_on_registry",
      existing_ref: `${pkgName}@${version}`,
    });
    return "skipped";
  }

  const publishTag = resolvePublishTag(pkgName, version);
  const publishArgs = ["publish", "--access=public", "--provenance"];
  if (publishTag) {
    publishArgs.push("--tag", publishTag);
    console.log(
      `publish ${label} ${pkgName}@${version} with dist-tag ${publishTag} (latest is newer)`,
    );
  }

  const result = spawnSync("npm", publishArgs, {
    cwd: pkgDir,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    throw new Error(
      `npm publish failed for ${pkgName}@${version} (exit ${result.status}).`,
    );
  }
  return "published";
}

let platformPublished = 0;
let platformSkipped = 0;

for (const target of config.targets) {
  const pkgDir = path.join(platformsDir, target.packageSuffix);
  if (!existsSync(pkgDir)) {
    throw new Error(
      `Platform package missing: ${pkgDir}. Did sync-platform-packages.mjs run?`,
    );
  }
  const outcome = publishPackage(pkgDir, "platform");
  if (outcome === "published") platformPublished += 1;
  else platformSkipped += 1;
}

const mainOutcome = publishPackage(mainPkgDir, "main");

console.log(
  `npm publish: main=${buildMainPackageName(config)} => ${mainOutcome}; platform=${platformPublished} published, ${platformSkipped} skipped.`,
);

// Output skipped_steps as JSON on stdout for receipt generation.
if (skippedSteps.length > 0) {
  console.log(`\nskipped_steps:`);
  for (const s of skippedSteps) {
    console.log(`  - step: "${s.step}"`);
    console.log(`    reason: "${s.reason}"`);
    console.log(`    existing_ref: "${s.existing_ref}"`);
  }
}
