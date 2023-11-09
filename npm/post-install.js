const exec = require("child_process").exec
const {family, GLIBC, MUSL} = require("detect-libc")
const platform = process.platform
const architecture = process.arch

const nodeToTailcallArchMap = {
  x64: "x86_64",
  arm64: "arm64",
  ia32: "i686",
}

const nodeToTailcallPlatformMap = {
  darwin: "apple",
  linux: "linux",
  win32: "windows",
}

let libcFamily
family().then((fam) => {
  libcFamily = fam
})

const tailcallArch = nodeToTailcallArchMap[architecture]
const tailcallPlatform = nodeToTailcallPlatformMap[platform]

let tailcallLibc
if (platform === "win32") {
  tailcallLibc = "-msvc"
} else {
  tailcallLibc = libcFamily === GLIBC ? "-gnu" : libcFamily === MUSL ? "-musl" : ""
}

const pkg = `@tailcallhq/core-${tailcallPlatform}-${tailcallArch}${tailcallLibc}`

if (optionalDependencies[pkg]) {
  exec(`npm install ${pkg}@${optionalDependencies[pkg]} --no-save`, (error, stdout, stderr) => {
    if (error) {
      console.error(`Failed to install optional dependency: ${pkg}`, stderr)
    } else {
      console.log(`Successfully installed optional dependency: ${pkg}`, stdout)
    }
  })
} else {
  throw new Error(`Unsupported platform ${platform} arch ${architecture}`)
}
