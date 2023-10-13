import * as fs from "node:fs"
import {dirname, resolve} from "node:path"
import {fileURLToPath} from "node:url"

const APP_VERSION = process.env.APP_VERSION

const MANIFEST_PATH = resolve(fileURLToPath(import.meta.url), "../..", "package.json")

const rootManifest = JSON.parse(fs.readFileSync(MANIFEST_PATH).toString("utf-8"))

function generatePackages() {
  const __filename = fileURLToPath(import.meta.url)
  const __dirname = dirname(__filename)

  let target

  try {
    target = resolve(__dirname, "../../../dist")
  } catch {
    console.log("[Publish] Error, no build is available")
    process.exit(1)
  }

  const optionalPackages = {}

  try {
    console.log("[Publish] Read Built Files")
    const files = fs.readdirSync(target)

    for (const _file of files) {
      const packageName = _file.replace("tailcall-", "")
      const packageRoot = resolve(__dirname, "../..", packageName)

      console.log(`[Publish] Copy ${_file} to ${packageRoot} `)
      fs.copyFileSync(`${target}/${_file}`, `${packageRoot}/tailcall`)

      const packageFileName = `${packageRoot}/package.json`

      const packagefile = fs.readFileSync(packageFileName, {encoding: "utf-8"})

      const packageJSON = JSON.parse(packagefile)

      packageJSON["version"] = APP_VERSION || "latest"

      const updatedPackageJSON = JSON.stringify(packageJSON, null, 2)
      fs.writeFileSync(packageFileName, updatedPackageJSON)

      optionalPackages[packageJSON.name] = APP_VERSION || "latest"

      console.log("[Publish] Update package.json for sub-package")
    }

    rootManifest["version"] = APP_VERSION || "latest"

    rootManifest["optionalDependencies"] = optionalPackages

    fs.writeFileSync(MANIFEST_PATH, JSON.stringify(rootManifest, null, 2))

    console.log("[Publish] Updated root package.json")
  } catch (error) {
    console.log("[Publish] [Error] with reading Target files")
    process.exit(1)
  }
}

generatePackages()
