import * as fs from "fs/promises"
import {resolve, dirname} from "path"
import * as yml from "yaml"

const __dirname = dirname(new URL(import.meta.url).pathname)

async function getBuildDefinitions() {
  const ciYMLPath = resolve(__dirname, "../.github/workflows/ci.yml")

  const ciYML = await fs.readFile(ciYMLPath, "utf8").then(yml.parse)
  return ciYML.jobs.release.strategy.matrix.build
}

const buildDefinitions = await getBuildDefinitions()

function getVersion() {
  return "0.1.0"
}

async function genServerPackage(buildDefinitions) {
  const tailcallPackage = {
    name: "@tailcallhq/server",
    version: getVersion(),
    description: "Tailcall Server",
    optionalDependencies: buildDefinitions.map((_) => "@tailcallhq/" + _),
  }

  const filePath = resolve(__dirname, "@tailcallhq/server")
  await fs.mkdir(filePath, {recursive: true})

  await fs.writeFile(resolve(filePath, "./package.json"), JSON.stringify(tailcallPackage, null, 2), "utf8")
}

async function genPlatformPackage({os, cpu}) {
  const name = `${cpu}-${os}`
  const platformPackage = {
    name: `@tailcallhq/${name}`,
    version: getVersion(),
    description: `Tailcall ${name} Platform`,
    directories: {bin: "bin"},
    os: [os],
    cpu: [cpu],
  }

  const filePath = resolve(__dirname, `@tailcallhq/${name}/bin`)
  await fs.mkdir(filePath, {recursive: true})
  await fs.writeFile(resolve(filePath, "../package.json"), JSON.stringify(platformPackage, null, 2), "utf8")
  fs.copyFile(resolve(__dirname, "../target", name, "release/tailcall"), resolve(filePath, "./tailcall"))
}

await genServerPackage(buildDefinitions)
await genPlatformPackage({os: "apple-darwin", cpu: "aarch64"})
