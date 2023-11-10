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
    scripts: {
      postinstall: "node ./scripts/installOptionalDeps.js",
      preinstall: "node ./scripts/preinstall.js",
    }
  };

  // Define the directory path where the package.json should be created
  const directoryPath = resolve(__dirname, "@tailcallhq/tailcall");
  const scriptsPath = resolve(directoryPath, "./scripts");

  await fs.mkdir(scriptsPath, { recursive: true });

  const installScriptContent = `
const exec = require('child_process').exec;
const optionalDependencies = ${JSON.stringify(optionalDependencies)};

Object.entries(optionalDependencies).forEach(([pkg, version]) => {
  exec(\`npm install \${pkg}@\${version} --no-save\`, (error, stdout, stderr) => {
    if (error) {
      console.error(\`Failed to install optional dependency: \${pkg}\`, stderr);
    } else {
      console.log(\`Successfully installed optional dependency: \${pkg}\`, stdout);
    }
  });
});
  `.trim();

  const preinstallScriptContent = `
const {platform, arch} = process;
const optionalDependencies = ${JSON.stringify(optionalDependencies)};
const getArchitecture = () => {
  if (arch === "x64") {
    return "x86_64"
  } else if (arch === "arm64") {
    return "[arm64|aarch64]"
  } else if (arch === "ia32") {
    return "i686"
  }
  return arch;
}
const getPlatform = () => {
  if (platform === 'win32') {
    return 'windows';
  } else if (platform === 'darwin') {
    return 'apple';
  }
  return platform;
}
const isSuppot = () => {
  const names = ['@tailcallhq/core', getPlatform(), getArchitecture()];
  return Object.keys(optionalDependencies).some(key => new RegExp(names.join('-')).test(key))
}
if (!isSuppot()) {
  throw new Error(\`Unsupported platform \${platform} arch \${arch}. Feedback: ${repository.url}\`);
}
  `.trim();

  // Ensure the directory exists
  await fs.mkdir(directoryPath, { recursive: true });

  await fs.writeFile(
    resolve(scriptsPath, "installOptionalDeps.js"),
    installScriptContent,
    "utf8"
  );

  await fs.writeFile(
    resolve(scriptsPath, "preinstall.js"),
    preinstallScriptContent,
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
