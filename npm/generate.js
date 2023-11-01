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

const { os, cpu, version } = getArguments();

if (!os || !cpu || !version) {
  console.error('Usage: node <script.js> --os <os> --cpu <cpu> --version <version>');
  process.exit(1);
}

async function genPlatformPackage() {
  const name = `${cpu}-${os}`;
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
  fs.copyFile(
    resolve(__dirname, "../target", name, "release/tailcall"),
    resolve(filePath, "./tailcall")
  );
}

await genPlatformPackage();
