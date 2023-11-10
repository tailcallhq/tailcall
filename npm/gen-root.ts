import * as fs from "fs/promises"
import { resolve, dirname } from "path"
import * as yml from "yaml"
import { fileURLToPath } from "url"
import { parse } from "ts-command-line-args"
import { PackageJson as IPackageJSON } from "type-fest"

const __dirname = dirname(fileURLToPath(import.meta.url));

interface ICLI {
    version: string
}

const options = parse<ICLI>({
    version: { alias: "v", type: String },
})

async function getBuildDefinitions(): Promise<string[]> {
    const ciYMLPath = resolve(__dirname, "../.github/workflows/ci.yml")
    const ciYML = await fs.readFile(ciYMLPath, "utf8").then(yml.parse)
    return ciYML.jobs.release.strategy.matrix.build
}

async function genServerPackage(buildDefinitions: string[]) {
    const packageVersion = options.version || "0.1.0"

    console.log(`Generating package.json with version ${packageVersion}`)

    // Construct the optionalDependencies object with the provided version
    const optionalDependencies: Record<string, string> = {}

    for (const buildDef of buildDefinitions) {
        optionalDependencies[`@tailcallhq/core-${buildDef}`] = packageVersion
    }

    const packageJson = await fs.readFile(resolve(__dirname, "./package.json"), "utf8")
    const basePackage = JSON.parse(packageJson) as IPackageJSON
    const { description, license, repository, homepage, keywords } = basePackage

    const tailcallPackage: IPackageJSON = {
        description: description!,
        license: license!,
        repository: repository!,
        homepage: homepage!,
        keywords: keywords!,
        name: "@tailcallhq/tailcall",
        type: 'module',
        version: packageVersion,
        optionalDependencies,
        dependencies: {
            "detect-libc": "^2.0.2",
        },
        scripts: {
            preinstall: "node ./scripts/installOptionalDeps.js",
        },
    }

    // Define the directory path where the package.json should be created
    const directoryPath = resolve(__dirname, "@tailcallhq/tailcall")
    const scriptsPath = resolve(directoryPath, "./scripts")

    await fs.mkdir(scriptsPath, { recursive: true })
    await fs.mkdir(directoryPath, { recursive: true })

    const postInstallScript = await fs.readFile(resolve(__dirname, "./pre-install.js"), "utf8")

    const installScriptContent = `const version = "${packageVersion}";\n${postInstallScript}`

    await fs.writeFile(resolve(scriptsPath, "installOptionalDeps.js"), installScriptContent, "utf8")
    await fs.writeFile(resolve(directoryPath, "./package.json"), JSON.stringify(tailcallPackage, null, 2), "utf8")

    await fs.copyFile(resolve(__dirname, "../README.md"), resolve(directoryPath, "./README.md"))
}

const buildDefinitions = await getBuildDefinitions()
await genServerPackage(buildDefinitions)
