import * as fs from "fs/promises"
import {resolve, dirname} from "path"
import * as yml from "yaml"
import {fileURLToPath} from "url"
import {parse} from "ts-command-line-args"
import {PackageJson as IPackageJSON} from "type-fest"

const __dirname = dirname(fileURLToPath(import.meta.url))

interface ICLI {
  version: string
  name: string
}

const options = parse<ICLI>({
  version: {alias: "v", type: String},
  name: {alias: "n", type: String},
})

async function getBuildDefinitions(): Promise<string[]> {
  const ciYMLPath = resolve(__dirname, "../.github/workflows/build_matrix.yml")
  const ciYML = await fs.readFile(ciYMLPath, "utf8").then(yml.parse)
  const steps = ciYML.jobs["setup-matrix"].steps

  for (const step of steps) {
    const matrix = step?.with?.matrix

    if (matrix) {
      // Parse yaml again since matrix is defined as string inside setup-matrix
      return yml.parse(matrix).build
    }
  }

  throw new Error("Cannot find matrix definition in workflow file")
}

async function genServerPackage(buildDefinitions: string[]) {
  const packageVersion = options.version || "0.1.0"
  const name = options.name || "@tailcallhq/tailcall"

  console.log(`Generating package.json with version ${packageVersion}`)

  // Construct the optionalDependencies object with the provided version
  const optionalDependencies: Record<string, string> = {}

  for (const buildDef of buildDefinitions) {
    optionalDependencies[`@tailcallhq/core-${buildDef}`] = packageVersion
  }

  const packageJson = await fs.readFile(resolve(__dirname, "./package.json"), "utf8")
  const basePackage = JSON.parse(packageJson) as IPackageJSON
  const {description, license, repository, homepage, keywords} = basePackage

  const tailcallPackage: IPackageJSON = {
    description: description!,
    license: license!,
    repository: repository!,
    homepage: homepage!,
    keywords: keywords!,
    name: name,
    type: "module",
    version: packageVersion,
    optionalDependencies,
    scarfSettings: {
      defaultOptIn: true,
      allowTopLevel: true,
    },
    dependencies: {
      "detect-libc": "^2.0.2",
      "@scarf/scarf": "^1.3.0",
    },
    scripts: {
      postinstall: "node ./scripts/post-install.js",
      preinstall: "node ./scripts/pre-install.js",
    },
  }

  // Define the directory path where the package.json should be created
  const directoryPath = resolve(__dirname, "@tailcallhq/tailcall")
  const scriptsPath = resolve(directoryPath, "./scripts")

  await fs.mkdir(scriptsPath, {recursive: true})
  await fs.mkdir(directoryPath, {recursive: true})

  const postInstallScript = await fs.readFile(resolve(__dirname, "./post-install.js"), "utf8")
  const preInstallScript = await fs.readFile(resolve(__dirname, "./pre-install.js"), "utf8")

  const postInstallScriptContent = `const version = "${packageVersion}";\n${postInstallScript}`
  const preInstallScriptContent = `const optionalDependencies = ${JSON.stringify(
    optionalDependencies,
  )};\n${preInstallScript}`

  await fs.writeFile(resolve(scriptsPath, "post-install.js"), postInstallScriptContent, "utf8")
  await fs.writeFile(resolve(scriptsPath, "pre-install.js"), preInstallScriptContent, "utf8")
  await fs.writeFile(resolve(directoryPath, "./package.json"), JSON.stringify(tailcallPackage, null, 2), "utf8")

  await fs.copyFile(resolve(__dirname, "../README.md"), resolve(directoryPath, "./README.md"))
}

const buildDefinitions = await getBuildDefinitions()
await genServerPackage(buildDefinitions)
