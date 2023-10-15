#!/usr/bin/env node
// @ts-check

import { spawnSync } from "child_process"
import { fileURLToPath } from "url"
import { dirname } from "path"
import EasyDl from "easydl"
import fs from "fs"
import os from "os"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const getBinName = () => {
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

  const binName = platformBinMap[platform]?.[architecture]

  return binName
}

const binaryName = getBinName()
const binaryPath = `${os.homedir()}/.tailcall/bin`
const fullPath = `${binaryPath}/tailcall`

const preload = async () => {
  if (fs.existsSync(fullPath)) return

  fs.mkdirSync(binaryPath, {recursive: true})

  const packageJson = JSON.parse(fs.readFileSync(`${__dirname}/../package.json`, "utf8"))
  const version = packageJson.version

  try {
    await new EasyDl(`https://github.com/tailcallhq/tailcall/releases/download/v${version}/${binaryName}`, fullPath, {
      connections: 10,
      maxRetry: 5,
      overwrite: true,
    }).wait()

    fs.chmodSync(fullPath, "755")
  } catch (err) {
    console.log("[error]", err)
  }
}

async function run() {
  await preload()
  const args = process.argv.slice(2)
  const processResult = spawnSync(fullPath, args, {stdio: "inherit"})
  process.exit(processResult.status ?? 0)
}

process.argv.includes("--preload") ? preload() : run()
