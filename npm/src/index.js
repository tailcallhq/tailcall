#!/usr/bin/env node
// @ts-check

import { spawnSync } from "child_process"
import { fileURLToPath } from 'url';
import { dirname } from 'path';
/**
 * Runs the application with args using nodejs spawn
 */

const getBinPath = () => {
  const platform = process.platform
  const architecture = process.arch

  const platformBinMap = {
    darwin: {
      arm64: "tailcall-aarch64-apple-darwin",
      x64: "tailcall-x86_64-apple-darwin",
    },
    linux: {
      arm64: "tailcall-aarch64-unknown-linux-gnu",
      x64: "tailcall-x86_64-unknown-linux-gnu",
      ia32: "tailcall-i686-unknown-linux-gnu",
    },
  }

  const binPath = platformBinMap[platform]?.[architecture]

  if (!binPath) {
    throw new Error(`unsupported ${platform} ${architecture}`)
  }

  return binPath
}

function run() {
  const args = process.argv.slice(2)
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  const processResult = spawnSync(`${__dirname}/../target/${getBinPath()}`, args, {stdio: "inherit"})
  process.exit(processResult.status ?? 0)
}

run()
