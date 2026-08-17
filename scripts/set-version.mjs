#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";
import process from "node:process";

const version = process.argv[2];
const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/;

if (!version || !semver.test(version)) {
  console.error("Usage: pnpm release:version <semver> (for example: 0.2.0)");
  process.exit(1);
}

async function updateJson(path) {
  const contents = JSON.parse(await readFile(path, "utf8"));
  contents.version = version;
  await writeFile(path, `${JSON.stringify(contents, null, 2)}\n`);
}

await Promise.all([
  updateJson("package.json"),
  updateJson("apps/desktop/package.json"),
  updateJson("apps/desktop/src-tauri/tauri.conf.json"),
]);

const cargoTomlPath = "Cargo.toml";
const cargoToml = await readFile(cargoTomlPath, "utf8");
const workspaceVersionPattern = /(\[workspace\.package\][\s\S]*?\nversion = ")[^"]+("\n)/;
if (!workspaceVersionPattern.test(cargoToml)) {
  throw new Error(`Could not update workspace version in ${cargoTomlPath}`);
}
const updatedCargoToml = cargoToml.replace(workspaceVersionPattern, `$1${version}$2`);
await writeFile(cargoTomlPath, updatedCargoToml);

const lockPath = "Cargo.lock";
const lock = await readFile(lockPath, "utf8");
const workspacePackages = [
  "agent-pipeline-core",
  "agent-pipeline-desktop",
  "agent-pipeline-runner",
];
let updatedLock = lock;
for (const packageName of workspacePackages) {
  const pattern = new RegExp(`(name = "${packageName}"\\nversion = ")[^"]+(")`);
  if (!pattern.test(updatedLock)) {
    throw new Error(`Could not find ${packageName} in ${lockPath}`);
  }
  updatedLock = updatedLock.replace(pattern, `$1${version}$2`);
}
await writeFile(lockPath, updatedLock);
await writeFile("VERSION", `${version}\n`);

console.log(`Prepared Agent Pipeline v${version}. Commit and push these changes to publish the release.`);
