import fs from "fs"
import {dirname, resolve} from "path"
import {fileURLToPath} from "url"
import YML from "yaml"

const __dirname = dirname(fileURLToPath(import.meta.url))

export default function get_matched_platform(os, arch, libc) {
  const directoryPath = resolve(__dirname, "../")
  const file = fs.readFileSync(resolve(directoryPath, "./build-matrix.yaml"), "utf8")
  const build_matrix = YML.parse(file, {mapAsMap: true})

  let found = null
  build_matrix.get("include").forEach((platform) => {
    const split = platform.get("build").split("-")
    const platform_arch = split.at(1)
    const platform_os = split.at(0)
    const platform_libc = split.at(-1)
    if (platform_arch == arch && platform_os == os && platform_libc == libc) {
      found = platform
    }
  })
  return found
}
