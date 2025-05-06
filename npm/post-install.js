// @ts-check
import {familySync, GLIBC, MUSL} from "detect-libc"
import {exec} from "child_process"
import util from "util"

const execa = util.promisify(exec)
const platform = process.platform
const arch = process.arch

const libcFamily = familySync()
let libc
if (platform === "win32") {
  libc = "-msvc"
} else {
  libc = libcFamily === GLIBC ? "-gnu" : libcFamily === MUSL ? "-musl" : ""
}

const pkg = `@tailcallhq/core-${platform}-${arch}${libc}`

// For Windows, check if MSVC is installed
async function checkMSVC() {
  if (platform === "win32") {
    try {
      await execa("where cl.exe")
      return true
    } catch (error) {
      return false
    }
  }
  return true  // Not Windows, no need for MSVC
}

async function install() {
  // Check for MSVC on Windows
  if (platform === "win32") {
    const hasMSVC = await checkMSVC()
    if (!hasMSVC) {
      console.error("\x1b[31mError: Microsoft Visual C++ Build Tools (MSVC) is required for Tailcall on Windows.\x1b[0m")
      console.error("Please install the Visual C++ Build Tools from: https://visualstudio.microsoft.com/visual-cpp-build-tools/")
      process.exit(1)
    }
  }

  try {
    // @ts-ignore
    const {stdout, stderr} = await execa(`npm install ${pkg}@${version} --no-save`)
    stderr ? console.log(stderr) : console.log(`Successfully installed optional dependency: ${pkg}`, stdout)
  } catch (error) {
    console.error(`\x1b[31mFailed to install optional dependency: ${pkg}\x1b[0m`)
    console.error(error.stderr || error.message || 'Unknown error')
    
    if (platform === "win32") {
      console.error("\nOn Windows, please ensure you have the following requirements:")
      console.error("1. Microsoft Visual C++ Build Tools installed")
      console.error("2. Node.js installed with the same architecture (x86 or x64) as your Windows")
    }
    
    // Kill the process with a non-zero exit code
    process.exit(1)
  }
}

install();
