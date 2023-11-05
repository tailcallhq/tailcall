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

const { target, build, version } = getArguments();

if (!target || !build || !version) {
    console.error('Usage: node <script.js> --target <target> --build <build> --version <version>');
    process.exit(1);
}

const targetRegex = /^([a-zA-Z0-9_]+)-([a-zA-Z0-9_]+)-([a-zA-Z0-9_\-]+)/;
const match = target.match(targetRegex);

if (!match) {
    console.error('Invalid target format. Expected format: <cpu>-<vendor>-<os>');
    process.exit(1);
}

const [, processor] = build.split('-');

const [, cpu, flavour, os] = match;

async function genPlatformPackage() {
    let name;
    if (flavour) {
        name = `${cpu}-${flavour}-${os}`;
    } else {
        name = `${cpu}-${os}`;
    }

    const packageJson = await fs.readFile(resolve(__dirname, "./package.json"), "utf8");
    const basePackage = JSON.parse(packageJson);
    const { description, license, repository, homepage, keywords } = basePackage

    const platformPackage = {
        description,
        license,
        repository,
        homepage,
        keywords,
        name: `@tailcallhq/core-${build}`,
        version,
        directories: { bin: "bin" },
        os: [os],
        cpu: [processor],
    };

    const filePath = resolve(__dirname, `@tailcallhq/core-${build}/bin`);
    await fs.mkdir(filePath, { recursive: true });
    await fs.writeFile(
        resolve(filePath, "../package.json"),
        JSON.stringify(platformPackage, null, 2),
        "utf8"
    );

    // Copy the executable to the bin directory
    await fs.copyFile(
        resolve(__dirname, "../target", name, "release/tailcall"),
        resolve(filePath, "tc")
    );

    await fs.copyFile(
        resolve(__dirname, "../README.md"),
        resolve(filePath, "../README.md")
    );

}

await genPlatformPackage();
