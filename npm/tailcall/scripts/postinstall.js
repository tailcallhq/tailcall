import {execSync} from "child_process"
import {readFileSync} from "fs"

const {platform, arch} = process

const PLATFORMS = {
  darwin: {
    arm64: "@tailcallhq/tailcall-aarch64-apple-darwin/tailcall",
    x64: "tailcall/tailcall-x86_64-apple-darwin/tailcall",
  },
  linux: {
    x64: "@tailcallhq/tailcall-x86_64-linux-gnu/tailcall",
    x32: "@tailcallhq/tailcall-i686-linux-gnu/tailcall",
    arm64: "@tailcallhq/tailcall-aarch64-linux-gnu/tailcall",
  },
}

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== "function") {
    try {
      const lddPath = execSync("which ldd").toString().trim()
      return readFileSync(lddPath, "utf8").includes("musl")
    } catch (e) {
      return true
    }
  } else {
    const {glibcVersionRuntime} = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

if (isMusl()) {
  PLATFORMS["linux"] = {
    ...PLATFORMS["linux"],
    x64: "@tailcallhq/tailcall-x86_64-linux-musl/tailcall",
    x32: "@tailcallhq/tailcall-i686-linux-musl/tailcall",
  }
}

const binName = PLATFORMS?.[platform]?.[arch]
console.log(binName, "name?")
if (binName) {
  let binPath
  try {
    binPath = require.resolve(binName)
  } catch {
    console.warn(
      `The tailcall CLI postinstall script failed to resolve the binary file "${binName}". Running tailcall from the npm package will probably not work correctly.`
    )
  }
} else {
  console.warn(
    "The tailcall CLI package doesn't ship with prebuilt binaries for your platform yet. " +
      "You can still use the CLI by cloning the tailcallhq/tailcall repo from GitHub, " +
      "and follow the instructions there to build the CLI for your platform."
  )
}
