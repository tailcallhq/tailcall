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
  const tailcallPackage = {
    name: "@tailcallhq/server",
    version: version || "0.1.0",
    description: "Tailcall Server",
    optionalDependencies: buildDefinitions.map((_) => "@tailcallhq/" + _),
  };

  const filePath = resolve(__dirname, "@tailcallhq/server");
  await fs.mkdir(filePath, { recursive: true });

  await fs.writeFile(
    resolve(filePath, "./package.json"),
    JSON.stringify(tailcallPackage, null, 2),
    "utf8"
  );
}

await getBuildDefinitions().then(genServerPackage);
