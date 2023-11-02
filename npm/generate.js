import * as fs from "fs/promises";
import { resolve, dirname } from "path";

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

const { target, version } = getArguments();

if (!target || !version) {
  console.error('Usage: node <script.js> --target <target> --version <version>');
  process.exit(1);
}

const targetRegex = /^([a-zA-Z0-9_]+)-([a-zA-Z0-9_]+)-([a-zA-Z0-9_\-]+)/;
const match = target.match(targetRegex);

if (!match) {
  console.error('Invalid target format. Expected format: <cpu>-<vendor>-<os>');
  process.exit(1);
}

const [, cpu, flavour ,os] = match;

async function genPlatformPackage() {
    let name;
    if (flavour) {
      name = `${cpu}-${flavour}-${os}`;
    } else {
      name = `${cpu}-${os}`;
    }
  const platformPackage = {
    name: `@tailcallhq/${name}`,
    version,
    description: `Tailcall ${name} Platform`,
    directories: { bin: "bin" },
    os: [os],
    cpu: [cpu],
  };

  const filePath = resolve(__dirname, `@tailcallhq/${name}/bin`);
  await fs.mkdir(filePath, { recursive: true });
  await fs.writeFile(
    resolve(filePath, "../package.json"),
    JSON.stringify(platformPackage, null, 2),
    "utf8"
  );
  await fs.copyFile(
    resolve(__dirname, "../target", name, "release/tailcall"),
    resolve(filePath, "./tailcall")
  );
}

await genPlatformPackage();
