import * as fs from "fs/promises"
import {resolve, dirname} from "path"
import {fileURLToPath} from "url"
import {parse} from "ts-command-line-args"
import {PackageJson as IPackageJSON} from "type-fest"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

interface ICLI {
  target: string
  build: string
  version: string
  ext?: string
  libc?: string
}

const options = parse<ICLI>({
  target: {type: String},
  build: {type: String},
  version: {type: String},
  ext: {type: String, optional: true},
  libc: {type: String, optional: true},
})

async function genPlatformPackage() {
  const {target, build, version, libc, ext} = options
  const [os, cpu] = build.split("-")

  const packageJson = await fs.readFile(resolve(__dirname, "./package.json"), "utf8")
  const basePackage = JSON.parse(packageJson) as IPackageJSON
  const {description, license, repository, homepage, keywords} = basePackage

  const platformPackage: IPackageJSON = {
    description: description!,
    license: license!,
    repository: repository!,
    homepage: homepage!,
    keywords: keywords!,
    name: `@tailcallhq/core-${build}`,
    version,
    bin: {tailcall: ext ? `./bin/tailcall${ext}` : `./bin/tailcall`},
    os: [os],
    cpu: [cpu],
  }

  if (libc) platformPackage.libc = [libc]

  const packagePath = `@tailcallhq/core-${build}`
  const binPath = `${packagePath}/bin`

  const targetPath = ext ? `../target/${target}/release/tailcall${ext}` : `../target/${target}/release/tailcall`
  const tcPath = ext ? `${binPath}/tailcall${ext}` : `${binPath}/tailcall`
  const packageJsonPath = `${packagePath}/package.json`
  const readmePath = "../README.md"

  await fs.mkdir(binPath, {recursive: true})
  await fs.writeFile(packageJsonPath, JSON.stringify(platformPackage, null, 2), "utf8")

  await fs.copyFile(targetPath, tcPath)
  await fs.copyFile(readmePath, `${packagePath}/README.md`)
}

await genPlatformPackage()
