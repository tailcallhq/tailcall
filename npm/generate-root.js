import * as fs from "fs/promises";
import { resolve, dirname } from "path";
import * as yml from "yaml";
import { fileURLToPath } from 'url';
import { exec } from 'child_process';

// This snippet makes sure __dirname is defined in an ES module context
const __dirname = dirname(fileURLToPath(import.meta.url));

function getArguments() {
  const args = {};
  for (let i = 2; i < process.argv.length; i += 2) {
    const key = process.argv[i].replace('--', '');
    const value = process.argv[i + 1];
    args[key] = value;
  }
  return args;
}

const { version } = getArguments();

async function getBuildDefinitions() {
  const ciYMLPath = resolve(__dirname, "../.github/workflows/ci.yml");

  const ciYML = await fs.readFile(ciYMLPath, "utf8").then(yml.parse);
  return ciYML.jobs.release.strategy.matrix.build;
}

async function genServerPackage(buildDefinitions) {
  // Use the version passed from the CLI or default to "0.1.0"
  const packageVersion = version || "0.1.0";
  console.log(`Generating package.json with version ${packageVersion}`);

  // Construct the optionalDependencies object with the provided version
  const optionalDependencies = buildDefinitions.reduce((deps, buildDef) => {
    deps["@tailcallhq/core-" + buildDef] = packageVersion;
    return deps;
  }, {});

  const packageJson = await fs.readFile(resolve(__dirname, "./package.json"), "utf8");
  const basePackage = JSON.parse(packageJson);
  const { description, license, repository, homepage, keywords } = basePackage

  const tailcallPackage = {
    description,
    license,
    repository,
    homepage,
    keywords,
    name: "@tailcallhq/tailcall",
    version: packageVersion,
    optionalDependencies,
    dependencies: {
      'detect-libc': '^2.0.2'
    },
    scripts: {
      postinstall: "node ./scripts/installOptionalDeps.js"
    }
  };

  // Define the directory path where the package.json should be created
  const directoryPath = resolve(__dirname, "@tailcallhq/tailcall");
  const scriptsPath = resolve(directoryPath, "./scripts");

  await fs.mkdir(scriptsPath, { recursive: true });

  const postInstallScript = await fs.readFile(resolve(__dirname, "./post-install.js"), "utf8");

  const installScriptContent = `const optionalDependencies = ${JSON.stringify(optionalDependencies)};\n${postInstallScript}`;

  // Ensure the directory exists
  await fs.mkdir(directoryPath, { recursive: true });

  await fs.writeFile(
    resolve(scriptsPath, "installOptionalDeps.js"),
    installScriptContent,
    "utf8"
  );

  // Write the package.json file with pretty JSON formatting
  await fs.writeFile(
    resolve(directoryPath, "./package.json"),
    JSON.stringify(tailcallPackage, null, 2),
    "utf8"
  );

  await fs.copyFile(
    resolve(__dirname, "../README.md"),
    resolve(directoryPath, "./README.md")
  );

}

// Execute the script with the provided version argument from CLI
const buildDefinitions = await getBuildDefinitions();
await genServerPackage(buildDefinitions);
