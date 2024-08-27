// @ts-check
import {familySync, GLIBC, MUSL} from "detect-libc"
import {exec} from "child_process"
import util from "util"
import get_matched_platform from "./utils.js"
import fs from "fs"
import axios from "axios"
import {resolve, dirname} from "path"
import {fileURLToPath} from "url"

const execa = util.promisify(exec)
const os = process.platform
const arch = process.arch
const libcFamily = familySync()

let libc = ""
if (os === "win32") {
  libc = "msvc"
} else {
  libc = libcFamily === GLIBC ? "gnu" : libcFamily === MUSL ? "musl" : ""
}

const matched_platform = get_matched_platform(os, arch, libc)
if (matched_platform != null) {
  const targetPlatform = matched_platform

  let targetPlatformExt = ""
  if (targetPlatform.get("ext") != undefined) {
    targetPlatformExt = targetPlatform.get("ext")
  }

  const pkg_download_base_url = "https://github.com/tailcallhq/tailcall/releases/download/"
  const specific_url = `v${version}/tailcall-${targetPlatform.get("target")}${targetPlatformExt}`
  const full_url = pkg_download_base_url + specific_url

  console.log(`Downloading Tailcall for ${targetPlatform.get("target")}${targetPlatformExt}  ...`)

  const output_path = `bin/tailcall-${targetPlatform.get("target")}${targetPlatformExt}`
  download_binary(full_url, output_path)
}

async function download_binary(full_url, output_path) {
  const file = fs.createWriteStream(output_path)
  axios({
    url: full_url,
    method: "GET",
    responseType: "stream",
  })
    .then((response) => {
      response.data.pipe(file)
      file.on("finish", async () => {
        const __dirname = dirname(fileURLToPath(import.meta.url))

        const directoryPath = resolve(__dirname, "../")
        const packageJsonString = fs.readFileSync(resolve(directoryPath, "./package.json"), "utf8")
        const packageJson = JSON.parse(packageJsonString)
        packageJson.bin = {tailcall: output_path}
        fs.writeFileSync(resolve(directoryPath, "./package.json"), JSON.stringify(packageJson, null, 2), "utf8")

        console.log("Tailcall binary downloaded successfully")
      })
    })
    .catch((error) => {
      console.error("Error downloading", error.message)
    })
}
