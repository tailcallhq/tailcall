import * as fs from "node:fs/promises"
import {dirname, resolve} from "node:path"
import {fileURLToPath} from "node:url"

const APP_VERSION = process.env.APP_VERSION

const MANIFEST_PATH = resolve(fileURLToPath(import.meta.url), "../package.json")

const rootManifest = JSON.parse(fs.readFileSync(MANIFEST_PATH).toString("utf-8"))

// Skip Running if no APP_VERSION is specified.
if (!APP_VERSION) {
  console.log("[Publish] No Version specified to publish CLI")
  process.exit(1)
}

async function generatePackages() {
  const __filename = fileURLToPath(import.meta.url)
  const __dirname = dirname(__filename)

  const target = resolve(__dirname, "../../../dist")
  const optionalPackages = {}

  try {
    console.log("[Publish] Read Built Files")
    const files = await fs.readdir(target)

    for (const _file of files) {
      const packageName = _file.replace("tailcall-", "")
      const packageRoot = resolve(__dirname, "../..", packageName)

      const packageFileName = `${packageRoot}/package.json`

      const packagefile = await fs.readFile(packageFileName, {encoding: "utf-8"})

      const packageJSON = JSON.parse(packagefile)

      packageJSON["version"] = APP_VERSION

      const updatedPackageJSON = JSON.stringify(packageJSON, null, 2)
      await fs.writeFile(packageFileName, updatedPackageJSON)

      optionalPackages[packageJSON.name] = APP_VERSION

      console.log("[Publish] Update package.json for sub-package")
    }

    rootManifest["version"] = APP_VERSION

    rootManifest["optionalDependencies"] = optionalPackages

    await fs.writeFile(MANIFEST_PATH, JSON.stringify(rootManifest, null, 2))

    console.log("[Publish] Updated root package.json")
  } catch (error) {
    console.error(error)
    process.exit(1)
  }
}

generatePackages()
