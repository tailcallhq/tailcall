import * as fs from "fs/promises";
import { resolve, dirname } from "path";
import * as yml from "yaml";

const __dirname = dirname(new URL(import.meta.url).pathname);

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
    deps["@tailcallhq/core-" + buildDef] = "*";
    return deps;
  }, {});

  const tailcallPackage = {
    name: "@tailcallhq/server",
    version: packageVersion,
    description: "Tailcall Server",
    optionalDependencies, // Now it's an object with versions set from CLI
  };

  // Define the directory path where the package.json should be created
  const directoryPath = resolve(__dirname, "@tailcallhq/server");

  // Ensure the directory exists
  await fs.mkdir(directoryPath, { recursive: true });

  // Write the package.json file with pretty JSON formatting
  await fs.writeFile(
    resolve(directoryPath, "./package.json"),
    JSON.stringify(tailcallPackage, null, 2),
    "utf8"
  );
}

// Execute the script with the provided version argument from CLI
const buildDefinitions = await getBuildDefinitions();
await genServerPackage(buildDefinitions);
