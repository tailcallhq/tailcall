#!/usr/bin/env node

import { familySync, GLIBC, MUSL } from "detect-libc";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const filename = fileURLToPath(import.meta.url);
const dirname = path.dirname(filename);
const rootDir = path.resolve(dirname, "..");

const platform = process.platform;
const arch = process.arch;

const libcFamily = familySync();
let libc;
if (platform === "win32") {
  libc = "-msvc";
} else {
  libc = libcFamily === GLIBC ? "-gnu" : libcFamily === MUSL ? "-musl" : "";
}

const pkg = `@tailcallhq/core-${platform}-${arch}${libc}`;
const binaryPath = path.join(rootDir, "node_modules", pkg, "bin", platform === "win32" ? "tailcall.exe" : "tailcall");

try {
  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    shell: true
  });

  child.on("error", (err) => {
    console.error(`Failed to execute tailcall: ${err.message}`);
    process.exit(1);
  });

  child.on("exit", (code) => {
    process.exit(code ?? 1);
  });
} catch (error) {
  console.error(`Failed to execute tailcall: ${error.message}`);
  process.exit(1);
}
