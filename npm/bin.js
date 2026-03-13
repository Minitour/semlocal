#!/usr/bin/env node

const { execFileSync } = require("child_process");

const PLATFORMS = {
  "win32 x64": "@semlocal/cli-win32-x64/bin/semlocal.exe",
  "darwin x64": "@semlocal/cli-darwin-x64/bin/semlocal",
  "darwin arm64": "@semlocal/cli-darwin-arm64/bin/semlocal",
  "linux x64": "@semlocal/cli-linux-x64/bin/semlocal",
  "linux arm64": "@semlocal/cli-linux-arm64/bin/semlocal",
};

const key = `${process.platform} ${process.arch}`;
const binPath = PLATFORMS[key];

if (!binPath) {
  console.error(`Unsupported platform: ${key}`);
  console.error(`semlocal supports: ${Object.keys(PLATFORMS).join(", ")}`);
  process.exit(1);
}

let binary;
try {
  binary = require.resolve(binPath);
} catch {
  console.error(
    `Could not find the semlocal binary for your platform (${key}).`
  );
  console.error(
    `The package ${binPath.split("/bin/")[0]} may not have been installed correctly.`
  );
  console.error("Try reinstalling: npm install -g semlocal");
  process.exit(1);
}

try {
  execFileSync(binary, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  if (e.status !== undefined) {
    process.exit(e.status);
  }
  throw e;
}
