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

try {
  // @ts-ignore
  const {stdout, stderr} = await execa(`npm install ${pkg}@${version} --no-save`)
  stderr ? console.log(stderr) : console.log(`Successfully installed optional dependency: ${pkg}`, stdout)
} catch (error) {
  console.error(`Failed to install optional dependency: ${pkg}`, error.stderr)
}
